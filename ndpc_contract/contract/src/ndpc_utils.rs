use core::ops::{Add, Div, Mul, Sub};

use alloc::string::ToString;
use alloc::{borrow::ToOwned, string::String};
use casper_contract::contract_api::runtime::get_key;
use casper_contract::{
    contract_api::{
        runtime::{get_call_stack, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::U512;
use casper_types::{
    system::CallStackElement, ApiError, ContractPackageHash, PublicKey, URef,
};
use ed25519_dalek::{ed25519::signature::Signature, Verifier};

use crate::constants::RUNTIME_FEE;
use crate::ndpc_types::{ApprovedNFT, NFTHolder, PublishRequest, NftMetadata};
use crate::{
    constants::NAMED_KEY_RATIO_VERIFIER,
    ndpc_types::{self, U64list},
    Error,
};

/// A shortcut function to get a dictionary by its name, and return its URef value
/// 
/// # Gets : 
/// `dict_name` : `&str`
/// # Returns : 
/// `URef` of the needed dictionary 
pub fn get_named_key_by_name(dict_name: &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert()
}

/// A getter function, which gets the ratio verifier account from the contract state, and return it as a PublicKey
pub fn get_ratio_verifier() -> casper_types::PublicKey {
    let ratio_verifier = casper_contract::contract_api::runtime::get_key(NAMED_KEY_RATIO_VERIFIER)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();
    storage::read::<casper_types::PublicKey>(ratio_verifier)
        .unwrap_or_revert()
        .unwrap_or_revert()
}

/// Verify a signature of a message, which is signed by the given publicKey
/// 
/// Note : It only supports Ed25519 publicKeys, as the verifier for the Secp256k1 keys, would larger up the contract and it would not be practical
pub fn verify_signature(public_key: PublicKey, signature: String, message: String) -> bool {
    let mut owned_string = "Casper Message:\n".to_owned();
    owned_string.push_str(&message);
    match public_key {
        casper_types::PublicKey::Ed25519(x) => {
            let sig = ed25519_dalek::Signature::from_bytes(
                base16::decode(signature.as_str()).unwrap().as_slice(),
            )
            .unwrap();
            x.verify(owned_string.as_bytes(), &sig).is_ok()
        }
        casper_types::PublicKey::Secp256k1(_x) => {
            revert(Error::TransferFailed);
        }
        _ => {
            panic!("Invalid ratio verifier type");
        }
    }
}

/// A shortcut function which returns the holders_cnt as a u64
pub fn get_holders_cnt(holders_cnt_uref: casper_types::URef) -> u64 {
    storage::read(holders_cnt_uref)
        .unwrap_or_revert()
        .unwrap_or_revert()
}

/// A shortcut function which returns the holder_ids which a account owns, as a Option<U64list>
pub fn get_holder_ids(owners_dict_uref: casper_types::URef, owner: &str) -> Option<U64list> {
    storage::dictionary_get(owners_dict_uref, owner).unwrap_or_revert()
}

/// Returns the contractPackage hash
/// 
/// Uses the callstack, to get the last caller, convert it to storedContract, and return its contractPackage hash
pub fn contract_package_hash() -> ContractPackageHash {
    let call_stacks = get_call_stack();
    let last_entry = call_stacks.last().unwrap_or_revert();
    let package_hash: Option<ContractPackageHash> = match last_entry {
        CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } => Some(*contract_package_hash),
        _ => None,
    };
    package_hash.unwrap_or_revert()
}

/// Gets `token_id` and returns the NFTMetadata object related to it
pub fn get_nft_metadata(token_id: String, metadatas_dict: URef) -> ndpc_types::NftMetadata {
    storage::dictionary_get::<NftMetadata>(metadatas_dict, token_id.as_str()).unwrap_or_revert_with(ApiError::from(Error
        ::MetadataDoesentExist)).unwrap_or_revert_with(ApiError::from(Error
    ::MetadataDoesentExist))
}

/// Gets `holder_id` and returns the NFTHolder object that has the holder_id in the holders_dict
pub fn get_holder_by_id(holders_dict: URef, holder_id: u64) -> NFTHolder {
    storage::dictionary_get::<ndpc_types::NFTHolder>(holders_dict, holder_id.to_string().as_str())
        .unwrap_or_revert_with(ApiError::from(Error::HolderDoesentExist))
        .unwrap_or_revert_with(ApiError::from(Error::HolderDoesentExist))
}

/// Gets `request_id`, and returns the PublisRequest related to it, gets the publishRequest from the `request_objects` dict
pub fn get_request_by_id(requests_dict: URef, request_id: u64) -> PublishRequest {
    storage::dictionary_get::<PublishRequest>(requests_dict, request_id.to_string().as_str())
        .unwrap_or_revert_with(ApiError::from(Error::RequestDoesntExist))
        .unwrap_or_revert_with(ApiError::from(Error::RequestDoesntExist))
}

/// Gets a `approved_id` and returns the ApprovedNFT related to that, gets the ApprovedNFT from the `approved` dict
pub fn get_approved_holder_by_id(approved_dict: URef, approved_id: u64) -> ApprovedNFT {
    storage::dictionary_get::<ApprovedNFT>(approved_dict, approved_id.to_string().as_str())
        .unwrap_or_revert_with(ApiError::from(Error::ApprovedHolderDoesentExist))
        .unwrap_or_revert_with(ApiError::from(Error::ApprovedHolderDoesentExist))
}

//-----------------------------------
/// Simply returns the ratioVerifier of the contract
/// 
/// This account would get the fee% of the payments
pub fn get_droplinked_account() -> PublicKey {
    get_ratio_verifier()
}

pub struct PaymentDetails {
    pub droplinked: U512,
    pub recipient: U512,
}
/// Calculates the amount of motes that need to be transfered to producer, and droplinked account
pub fn calculate_payment(
    product_price: U512,
    shipping_price: U512,
    tax_price: U512,
    fee: u64,
) -> PaymentDetails {
    let droplinked_part = product_price.mul(fee).div(10000u64);
    let recipient_part = shipping_price
        .add(product_price.sub(droplinked_part))
        .add(shipping_price)
        .add(tax_price);
    PaymentDetails {
        droplinked: droplinked_part,
        recipient: recipient_part,
    }
}
/// A shortcut function, which returns the fee that was set in the contract state during installation process
pub(crate) fn get_fee() -> u64 {
    let fee_uref = get_key(RUNTIME_FEE)
        .unwrap_or_revert_with(Error::KeyNotFound)
        .into_uref()
        .unwrap_or_revert_with(Error::KeyNotUref);
    storage::read::<u64>(fee_uref)
        .unwrap_or_revert_with(Error::FeeNotFound)
        .unwrap_or_revert_with(Error::FeeNotFound)
}
