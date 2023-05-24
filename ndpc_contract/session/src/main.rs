#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::string::String;

use casper_contract::{
    contract_api::{runtime, system, account},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{Key, U512, ContractHash, RuntimeArgs, ApiError};


#[no_mangle]
pub extern "C" fn call() {
    let cnt : u64 = runtime::get_named_arg("cnt");
    let approved_id : u64 = runtime::get_named_arg("approved_id");
    let amount: U512 = runtime::get_named_arg("amount");
    let shipping_price: U512 = runtime::get_named_arg("shipping_price");
    let tax_price: U512 = runtime::get_named_arg("tax_price");
    let contract_hash_key : Key = runtime::get_named_arg("contract_hash");
    let ratio : String = runtime::get_named_arg("current_price_timestamp");
    let signature : String = runtime::get_named_arg("signature");
    
    let contract_hash_bytes = contract_hash_key.into_hash().unwrap_or_revert_with(ApiError::User(1));
    let contract_hash = ContractHash::new(contract_hash_bytes);
    let entry_point_name : &str = "buy";

    let new_purse = system::create_purse();
    system::transfer_from_purse_to_purse(account::get_main_purse(), new_purse, amount, None)
        .unwrap_or_revert_with(ApiError::User(2));
    let mut runtimeargs = RuntimeArgs::new();
    runtimeargs.insert("purse_addr", Key::URef(new_purse)).unwrap_or_revert_with(ApiError::User(3));
    runtimeargs.insert("amount", cnt).unwrap_or_revert_with(ApiError::User(4));
    runtimeargs.insert("approved_id", approved_id).unwrap_or_revert_with(ApiError::User(5));
    runtimeargs.insert("signature", signature).unwrap_or_revert_with(ApiError::User(6));
    runtimeargs.insert("current_price_timestamp", ratio).unwrap_or_revert_with(ApiError::User(7));
    runtimeargs.insert("shipping_price", shipping_price).unwrap_or_revert_with(ApiError::User(8));
    runtimeargs.insert("tax_price", tax_price).unwrap_or_revert_with(ApiError::User(9));
    runtime::call_contract::<()>(contract_hash, entry_point_name, runtimeargs);

}