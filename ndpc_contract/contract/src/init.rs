use crate::constants::{
    NAMED_KEY_DICT_APPROVED_NAME, NAMED_KEY_DICT_HOLDERS_NAME, NAMED_KEY_DICT_METADATAS_NAME,
    NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME, NAMED_KEY_DICT_PROD_REQS,
    NAMED_KEY_DICT_PUBAPPROVED_NAME, NAMED_KEY_DICT_PUB_REQS, NAMED_KEY_DICT_REQ_OBJ,
    NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME, NAMED_KEY_DICT_TOTAL_SUPPLY,
};
use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};

/// init entrypoint, the first called method of the contract
/// 
/// It would be called after the contract is deployed, by the deployer of the contract, and is only callable once and only once, It will
/// initialize dictionaries that the contract need to work with, and put them into the namedkeys of the contract. 
#[no_mangle]
pub extern "C" fn init() {
    storage::new_dictionary(NAMED_KEY_DICT_APPROVED_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_HOLDERS_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_METADATAS_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_OWNERS_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_PRODAPPROVED_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_PUBAPPROVED_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_REQ_OBJ).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_PROD_REQS).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_PUB_REQS).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_TOTAL_SUPPLY).unwrap_or_revert();
}
