use alloc::{string::String, borrow::ToOwned};
use casper_contract::{unwrap_or_revert::UnwrapOrRevert, contract_api::{runtime::{self, revert}, storage}};
use casper_types::PublicKey;
use ed25519_dalek::{Verifier, ed25519::signature::Signature};

use crate::{constants::{NAMED_KEY_LATEST_TIMESTAMP, NAMED_KEY_RATIO_VERIFIER}, Error};

pub fn get_named_key_by_name(dict_name : &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name).unwrap_or_revert().into_uref().unwrap_or_revert()
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

pub fn get_ratio_verifier() -> casper_types::PublicKey{
    let ratio_verifier = casper_contract::contract_api::runtime::get_key(NAMED_KEY_RATIO_VERIFIER).unwrap_or_revert().into_uref().unwrap_or_revert();
    storage::read::<casper_types::PublicKey>(ratio_verifier).unwrap_or_revert().unwrap_or_revert()
}

pub fn verify_signature(public_key : PublicKey, signature : String, message : String) -> bool{
    let mut owned_string = "Casper Message:\n".to_owned();
    owned_string.push_str(&message);
    match public_key{
        casper_types::PublicKey::Ed25519(x)=>{
            let sig = ed25519_dalek::Signature::from_bytes(base16::decode(signature.as_str()).unwrap().as_slice()).unwrap();
            x.verify(owned_string.as_bytes(), &sig).is_ok()
        }
        casper_types::PublicKey::Secp256k1(_x)=>{
            revert(Error::TransferFailed);
        }
        _ => {
            panic!("Invalid ratio verifier type");
        }
    }
}

pub fn get_latest_timestamp() -> u64{
    let timestamp = runtime::get_key(NAMED_KEY_LATEST_TIMESTAMP).unwrap_or_revert().into_uref().unwrap_or_revert();
    storage::read::<u64>(timestamp).unwrap_or_revert().unwrap_or_revert()
}
pub fn set_latest_timestamp(timestamps : u64){
    let timestamp = runtime::get_key(NAMED_KEY_LATEST_TIMESTAMP).unwrap_or_revert().into_uref().unwrap_or_revert();
    storage::write(timestamp, timestamps);
}
