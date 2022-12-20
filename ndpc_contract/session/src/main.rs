#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{string::String, vec};

use casper_contract::{
    contract_api::{runtime, storage, system, account},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, Key, URef, U512, TransferredTo, EntryPoints, CLValue, ContractHash, runtime_args, RuntimeArgs};

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
enum Error {
    KeyAlreadyExists = 0,
    KeyMismatch = 1,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[no_mangle]
pub extern "C" fn call() {
    let cnt : u64 = runtime::get_named_arg("cnt");
    let approved_id : u64 = runtime::get_named_arg("approved_id");
    let amount: U512 = runtime::get_named_arg("amount");
    let contract_hash_key : Key = runtime::get_named_arg("contract_hash");
    let contract_hash_bytes = contract_hash_key.into_hash().unwrap_or_revert();
    let contract_hash = ContractHash::new(contract_hash_bytes);
    let entry_point_name : &str = "buy";
    // This creates a new empty purse that the caller will use just this one time.
    let new_purse = system::create_purse();
    system::transfer_from_purse_to_purse(account::get_main_purse(), new_purse, amount, None)
        .unwrap_or_revert();
    let mut runtimeargs = RuntimeArgs::new();
    runtimeargs.insert("purse_addr", Key::URef(new_purse));
    runtimeargs.insert("amount", cnt);
    runtimeargs.insert("approved_id", approved_id);
    runtime::call_contract::<()>(contract_hash, entry_point_name, runtimeargs);
}