#![no_std]
#![no_main]
pub mod affiliate;
mod constants;
pub mod event;
pub mod init;
pub mod mint;
pub mod ndpc_types;
mod ndpc_utils;
pub mod payment;
#[allow(unused_imports)]
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, AsymmetricType, ContractPackageHash, PublicKey, RuntimeArgs, URef};
use constants::{get_entrypoints, get_named_keys, NAMED_KEY_RATIO_VERIFIER};


/// All the different User error types for the contract
#[repr(u16)]
enum Error {
    NotAccountHash = 0,
    MintMetadataNotValid = 1,
    PuseIsNotValid = 2,
    NotOwnerOfHolderId = 3,
    ApprovedHolderDoesentExist = 4,
    NotEnoughAmount = 5,
    MetadataDoesentExist = 6,
    _NotEnoughBalance = 7,
    TransferFailed = 8,
    HolderDoesentExist = 9,
    _ApprovedListDoesentExist = 10,
    EmptyOwnerShipList = 11,
    PublisherHasNoApprovedHolders = 12,
    ProducerHasNoApprovedHolders = 13,
    EmptyRequestCnt = 14,
    AccessDenied = 15,
    EmptyU64List = 16,
    MintHolderNotFound = 17,
    InvalidSignature = 18,
    InvalidTimestamp = 19,
    GetBalance = 20,
    InsufficientFunds = 21,
    KeyNotFound = 22,
    FeeNotFound = 23,
    KeyNotUref = 24,
    RequestDoesntExist = 25,
}
impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

/// Installs the droplinked_contract with the given session args and calls the init function of the contract
/// 
/// It'll get fee and ratio_verifier publicKey (the account which verifies the ratio of cspr/usd) as input, install droplinked's contract on testnet, and call it's init entrypoint
/// It stores the contract hash and contract-package hash into the deployer's NAMEDKEYS, Creates a constructor group, which are the only group who can call the init function, adds the deployer to them,
/// Calls the init function and then removes the deployer from that group so that the init method would never be called again!
fn install_contract() {
    let fee: u64 = runtime::get_named_arg(constants::RUNTIME_FEE);
    let ratio_verifier_hex = runtime::get_named_arg::<String>(NAMED_KEY_RATIO_VERIFIER);
    let ratio_verifier = PublicKey::from_hex(ratio_verifier_hex).unwrap();
    let entry_points = get_entrypoints();
    let named_keys = get_named_keys(ratio_verifier, fee);
    let (contract_hash, _contract_version) = storage::new_locked_contract(
        entry_points,
        Some(named_keys),
        Some(constants::CONTRACTPACKAGEHASH.to_string()),
        None,
    );
    let package_hash = ContractPackageHash::new(
        runtime::get_key(constants::CONTRACTPACKAGEHASH)
            .unwrap_or_revert()
            .into_hash()
            .unwrap_or_revert(),
    );
    let constructor_access: URef =
        storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
            .unwrap_or_revert()
            .pop()
            .unwrap_or_revert();
    let _: () = runtime::call_contract(contract_hash, "init", RuntimeArgs::new());
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
        .unwrap_or_revert();
    runtime::put_key("droplinked_contract", contract_hash.into());
}

/// This method gets executed when the session code is deployed to the network, It will simply call the install_contract function to install the contract
#[no_mangle]
pub extern "C" fn call() {
    install_contract();
}
