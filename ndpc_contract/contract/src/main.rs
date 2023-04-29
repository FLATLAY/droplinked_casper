#![no_std]
#![no_main] 
pub mod ndpc_types;
mod ndpc_utils;
mod constants;
pub mod event;
pub mod mint;
pub mod affiliate;
pub mod payment;
pub mod init;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::{string::{String, ToString}, collections::BTreeSet};
use casper_contract::{contract_api::{runtime, storage}, unwrap_or_revert::UnwrapOrRevert};
use constants::{get_entrypoints, get_named_keys};
use casper_types::{RuntimeArgs, ApiError, URef, ContractPackageHash, PublicKey, AsymmetricType};

#[repr(u16)]
enum Error {
    NotAccountHash = 0,
    MintMetadataNotValid = 1,
    NoTokensFound = 2,
    NotOwnerOfHolderId = 3,
    NotEnoughTokens = 4,
    ApprovedHolderDoesentExist = 5,
    NotEnoughAmount = 6,
    MetadataDoesentExist = 7,
    NotEnoughBalance = 8,
    TransferFailed = 9,
    HolderDoesentExist = 10,
    ApprovedListDoesentExist = 11,
    EmptyOwnerShipList = 12,
    PublisherHasNoApprovedHolders = 13,
    ProducerHasNoApprovedHolders = 15,
    EmptyRequestCnt = 17,
    AccessDenied = 18,
    EmptyU64List = 19,
    MintHolderNotFound = 21,
    InvalidSignature = 23,
    InvalidTimestamp = 24,
    OfferDoesentExist = 25,
    EmptyOfferCnt = 26,
}
impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

fn install_contract(){
    let time_stamp = runtime::get_named_arg::<u64>("timestamp");
    let ratio_verifier_hex = runtime::get_named_arg::<String>("ratio_verifier");
    let ratio_verifier = PublicKey::from_hex(ratio_verifier_hex).unwrap();
    let entry_points = get_entrypoints();
    let named_keys = get_named_keys(time_stamp, ratio_verifier);
    let (contract_hash , _contract_version) = storage::new_contract(entry_points, Some(named_keys) , Some(constants::CONTRACTPACKAGEHASH.to_string()), None);
    let package_hash = ContractPackageHash::new(runtime::get_key(constants::CONTRACTPACKAGEHASH).unwrap_or_revert().into_hash().unwrap_or_revert());
    let constructor_access: URef = storage::create_contract_user_group(package_hash, "constructor", 1, Default::default()).unwrap_or_revert().pop().unwrap_or_revert();
    let _: () = runtime::call_contract(contract_hash, "init", RuntimeArgs::new());
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs).unwrap_or_revert();
    runtime::put_key("droplink_contract", contract_hash.into());
}

#[no_mangle]
pub extern "C" fn call() {
    install_contract();
}