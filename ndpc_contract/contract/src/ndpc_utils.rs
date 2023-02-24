use alloc::string::{self, String, ToString};
use casper_contract::{unwrap_or_revert::UnwrapOrRevert, contract_api::{runtime, storage}};
use casper_types::{bytesrepr::ToBytes, PublicKey};
use ed25519::SignatureEncoding;
use ed25519_dalek::Verifier;

use secp256k1::{Secp256k1, Message, hashes::{sha256, hex}};

use crate::constants::{NAMED_KEY_LATEST_TIMESTAMP, NAMED_KEY_RATIO_VERIFIER};

pub fn get_named_key_by_name(dict_name : &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name).unwrap_or_revert().into_uref().unwrap_or_revert()
}

//TODO : need to be checked
pub fn secp256k1_verify(public_key : String, signature : String, message : String) -> bool{
    let secp = Secp256k1::new();
    let public_key_bytes = base16::decode(public_key.as_str()).unwrap();
    let public_key_bytes_arr = public_key_bytes.as_slice();
    let public_key = secp256k1::PublicKey::from_slice(public_key_bytes_arr).unwrap();
    let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    let signature_bytes = base16::decode(signature.as_str()).unwrap();
    let sb_arr = signature_bytes.as_slice();
    let sig = secp256k1::ecdsa::Signature::from_der(sb_arr).unwrap();
    secp.verify_ecdsa(&message, &sig, &public_key).is_ok()
}


//TODO : need to be checked
pub fn ed25519_verify(public_key : ed25519_dalek::PublicKey, signature : String, message : String) -> bool{
    let sig = ed25519::Signature::from_slice(&signature.to_bytes().unwrap()).unwrap();
    let sig = ed25519_dalek::Signature::from(sig.to_bytes());
    public_key.verify(message.as_bytes(), &sig).is_ok()
}

pub fn get_ratio_verifier() -> casper_types::PublicKey{
    let ratio_verifier = casper_contract::contract_api::runtime::get_key(NAMED_KEY_RATIO_VERIFIER).unwrap_or_revert().into_uref().unwrap_or_revert();
    storage::read::<casper_types::PublicKey>(ratio_verifier).unwrap_or_revert().unwrap_or_revert()
}

pub fn verify_signature(public_key : PublicKey, signature : String, message : String) -> bool{
    match (public_key){
        casper_types::PublicKey::Ed25519(x)=>{
            ed25519_verify(x, signature, message)
        }
        casper_types::PublicKey::Secp256k1(x)=>{
            secp256k1_verify(base16::encode_lower(&x.to_bytes()), signature, message)
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
pub fn set_latest_timestamp(timestamp : u64){
    let timestamp = runtime::get_key(NAMED_KEY_LATEST_TIMESTAMP).unwrap_or_revert().into_uref().unwrap_or_revert();
    storage::write(timestamp, timestamp);
}