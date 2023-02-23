use alloc::string::{self, String};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::bytesrepr::ToBytes;
use ed25519::SignatureEncoding;
use ed25519_dalek::Verifier;

use secp256k1::{Secp256k1, PublicKey, Message, hashes::{sha256, hex}};

pub fn get_named_key_by_name(dict_name : &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name).unwrap_or_revert().into_uref().unwrap_or_revert()
}

//TODO : need to be checked
pub fn secp256k1_verify(public_key : String, signature : String, message : String) -> bool{
    let secp = Secp256k1::new();
    let public_key_bytes = base16::decode(public_key.as_str()).unwrap();
    let public_key_bytes_arr = public_key_bytes.as_slice();
    let public_key = PublicKey::from_slice(public_key_bytes_arr).unwrap();
    let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    let signature_bytes = base16::decode(signature.as_str()).unwrap();
    let sb_arr = signature_bytes.as_slice();
    let sig = secp256k1::ecdsa::Signature::from_der(sb_arr).unwrap();
    secp.verify_ecdsa(&message, &sig, &public_key).is_ok()
}


//TODO : need to be checked
pub fn ed25519_verify(public_key : String, signature : String, message : String) -> bool{
    let p_key = ed25519_dalek::PublicKey::from_bytes(base16::decode(public_key.as_str()).unwrap().as_slice()).unwrap();
    let sig = ed25519::Signature::from_slice(&signature.to_bytes().unwrap()).unwrap();
    let sig = ed25519_dalek::Signature::from(sig.to_bytes());
    p_key.verify(message.as_bytes(), &sig).is_ok()
}