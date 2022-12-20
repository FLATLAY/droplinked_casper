use casper_contract::unwrap_or_revert::UnwrapOrRevert;

pub fn get_named_key_by_name(dict_name : &str) -> casper_types::URef {
    casper_contract::contract_api::runtime::get_key(dict_name).unwrap_or_revert().into_uref().unwrap_or_revert()
}