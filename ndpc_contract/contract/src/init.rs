use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use crate::constants::{
    NAMED_KEY_DICT_APPROVED_NAME, NAMED_KEY_DICT_HOLDERS_NAME, NAMED_KEY_DICT_METADATAS_NAME, 
    NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME, NAMED_KEY_DICT_PUBAPPROVED_NAME, 
    NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME, NAMED_KEY_DICT_REQ_OBJ, NAMED_KEY_DICT_PROD_REQS, 
    NAMED_KEY_DICT_PUB_REQS, NAMED_KEY_DICT_PUB_REJS, NAMED_KEY_DICT_TOTAL_SUPPLY};

#[no_mangle]
pub extern "C" fn init(){
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
    storage::new_dictionary(NAMED_KEY_DICT_PUB_REJS).unwrap_or_revert();
    storage::new_dictionary(NAMED_KEY_DICT_TOTAL_SUPPLY).unwrap_or_revert();
}