use core::ops::{Add, Div, Mul, Sub};

use alloc::string::ToString;
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use casper_contract::contract_api::runtime::get_key;
use casper_contract::{
    contract_api::{
        runtime::{self, get_call_stack, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::U512;
use casper_types::{
    system::CallStackElement, ApiError, ContractPackageHash, PublicKey, URef, U256,
};
use ed25519_dalek::{ed25519::signature::Signature, Verifier};

use crate::constants::RUNTIME_FEE;
use crate::ndpc_types::{ApprovedNFT, NFTHolder, PublishRequest};
use crate::{
    constants::NAMED_KEY_RATIO_VERIFIER,
    ndpc_types::{self, U64list},
    Error,
};

pub fn get_named_key_by_name(dict_name: &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert()
}

//TODO : need to be checked
// pub fn secp256k1_verify(public_key : [u8;33], signature : String, message : String) -> bool{
//     let secp = Secp256k1::new();
//     let public_key = secp256k1::PublicKey::from_slice(&public_key).unwrap();
//     let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
//     let signature_bytes = base16::decode(signature.as_str()).unwrap();
//     let sb_arr = signature_bytes.as_slice();
//     let sig = secp256k1::ecdsa::Signature::from_der(sb_arr).unwrap();
//     secp.verify_ecdsa(&message, &sig, &public_key).is_ok()
// }

pub fn get_ratio_verifier() -> casper_types::PublicKey {
    let ratio_verifier = casper_contract::contract_api::runtime::get_key(NAMED_KEY_RATIO_VERIFIER)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();
    storage::read::<casper_types::PublicKey>(ratio_verifier)
        .unwrap_or_revert()
        .unwrap_or_revert()
}

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

pub fn get_holders_cnt(holders_cnt_uref: casper_types::URef) -> u64 {
    storage::read(holders_cnt_uref)
        .unwrap_or_revert()
        .unwrap_or_revert()
}

pub fn get_holder_ids(owners_dict_uref: casper_types::URef, owner: &str) -> Option<U64list> {
    storage::dictionary_get(owners_dict_uref, owner).unwrap_or_revert()
}

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

pub(crate) fn get_nft_metadata(token_id: String, metadatas_dict: URef) -> ndpc_types::NftMetadata {
    let metadata_opt =
        storage::dictionary_get::<String>(metadatas_dict, token_id.as_str()).unwrap_or_revert();
    if metadata_opt.is_none() {
        //the metadata does not exist
        runtime::revert(ApiError::from(Error::MetadataDoesentExist));
    }
    let metadata_string = metadata_opt.unwrap_or_revert();
    //split by , => [name , token_uri , checksum , price]
    let metadata_split = metadata_string.split(',').collect::<Vec<&str>>();
    let name = metadata_split[0].to_string();
    let token_uri = metadata_split[1].to_string();
    let checksum = metadata_split[2].to_string();
    let price = U256::from_dec_str(metadata_split[3]).unwrap();
    let comission = metadata_split[4].to_string().parse().unwrap();
    ndpc_types::NftMetadata::new(name, token_uri, checksum, price, comission)
}

pub fn get_holder_by_id(holders_dict: URef, holder_id: u64) -> NFTHolder {
    storage::dictionary_get::<ndpc_types::NFTHolder>(holders_dict, holder_id.to_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::HolderDoesentExist))
}

pub fn get_request_by_id(requests_dict: URef, request_id: u64) -> PublishRequest {
    storage::dictionary_get::<PublishRequest>(requests_dict, request_id.to_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert()
}

pub fn get_approved_holder_by_id(approved_dict: URef, approved_id: u64) -> ApprovedNFT {
    storage::dictionary_get::<ApprovedNFT>(approved_dict, approved_id.to_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::ApprovedHolderDoesentExist))
}

//-----------------------------------

pub fn get_droplinked_account() -> PublicKey {
    get_ratio_verifier()
}
pub struct PaymentDetails {
    pub droplinked: U512,
    pub recipient: U512,
}

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

pub(crate) fn get_fee() -> u64 {
    let fee_uref = get_key(RUNTIME_FEE)
        .unwrap_or_revert_with(Error::KeyNotFound)
        .into_uref()
        .unwrap_or_revert_with(Error::KeyNotUref);
    storage::read::<u64>(fee_uref)
        .unwrap_or_revert_with(Error::FeeNotFound)
        .unwrap_or_revert_with(Error::FeeNotFound)
}
