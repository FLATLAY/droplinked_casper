use alloc::string::{self, String};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::PublicKey;
use secp256k1::Secp256k1;

pub fn get_named_key_by_name(dict_name : &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name).unwrap_or_revert().into_uref().unwrap_or_revert()
}

pub fn secp256k1_verify(public_key : String, signature : String, message : String) -> bool{
    let secp = Secp256k1::new();
    let public_key_bytes = base16::decode(public_key.as_str()).unwrap();
    let kk = public_key_bytes.as_slice();
    let public_key = PublicKey::from_slice(kk).unwrap();
    let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    let signature_bytes = base16::decode(signature.as_str()).unwrap();
    let mm = signature_bytes.as_slice();
    let sig = secp256k1::ecdsa::Signature::from_der(mm).unwrap();
    secp.verify_ecdsa(&message, &sig, &public_key).is_ok()
}