use crate::constants::{
    NAMED_KEY_DICT_APPROVED_NAME, NAMED_KEY_DICT_HOLDERS_NAME, NAMED_KEY_DICT_METADATAS_NAME,
    NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME, NAMED_KEY_DICT_PROD_REQS,
    NAMED_KEY_DICT_PUBAPPROVED_NAME, NAMED_KEY_DICT_PUB_REQS, NAMED_KEY_DICT_REQ_OBJ,
    NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME, NAMED_KEY_DICT_TOTAL_SUPPLY,
};
use casper_contract::{contract_api::{storage}, unwrap_or_revert::UnwrapOrRevert};

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
    
    // let nft_metadata = NftMetadata::new("lol".to_string(), "asoidjaspoidj".to_string(), "aosodijasodij".to_string(), 1000, 1234);
    // let hash = nft_metadata.get_hash();
    // {
    //     let metadats_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_METADATAS_NAME);
    //     storage::dictionary_put(metadats_dict, 1u64.to_string().as_str(), nft_metadata);
    // }
    // {
    //     let hash_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME);
    //     storage::dictionary_put(hash_dict, &hash.as_string(), 1u64);
    // }
    // let nft_holder = NFTHolder::new(1000, 1);
    // {
    //     let holders_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_HOLDERS_NAME);
    //     storage::dictionary_put(holders_dict, 1u64.to_string().as_str(), nft_holder);
    // }
    // {
    //     let owners_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_OWNERS_NAME);
    //     let mut ll = U64list::new();
    //     ll.add(1u64);
    //     storage::dictionary_put(owners_dict, 1u64.to_string().as_str(), ll);
    // }

    // let request = PublishRequest::new(1, 10, get_caller(), get_caller());
    // {
    //     let req_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_REQ_OBJ);
    //     storage::dictionary_put(req_dict, 1u64.to_string().as_str(), request);

    //     let pub_reqs_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PUB_REQS);
    //     let mut ll = U64list::new();
    //     ll.add(1u64);
    //     storage::dictionary_put(pub_reqs_dict, get_caller().to_string().as_str(), ll);

    //     let prod_reqs_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PROD_REQS);
    //     let mut ll = U64list::new();
    //     ll.add(1u64);
    //     storage::dictionary_put(prod_reqs_dict, get_caller().to_string().as_str(), ll);
    // }
    // let approved = ApprovedNFT::new(1, 10, get_caller(), get_caller() , 1u64);
    // {
    //     let approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_APPROVED_NAME);
    //     storage::dictionary_put(approved_dict, 1u64.to_string().as_str(), approved);

    //     let pub_approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PUBAPPROVED_NAME);
    //     let mut ll = U64list::new();
    //     ll.add(1u64);
    //     storage::dictionary_put(pub_approved_dict, get_caller().to_string().as_str(), ll);

    //     let prod_approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PRODAPPROVED_NAME);
    //     let mut ll = U64list::new();
    //     ll.add(1u64);
    //     storage::dictionary_put(prod_approved_dict, get_caller().to_string().as_str(), ll);
    // }
    // // put 1 in total supply , holders_cnt , approved_cnt , pub_approved_cnt , prod_approved_cnt , pub_reqs_cnt , prod_reqs_cnt named_keys
    // {
    //     let total_supply_uref = runtime::get_key(NAMED_KEY_DICT_TOTAL_SUPPLY).unwrap_or_revert().into_uref().unwrap_or_revert();
    //     storage::write(total_supply_uref, 1u64);

    //     let holders_cnt_uref = runtime::get_key(NAMED_KEY_HOLDERSCNT).unwrap_or_revert().into_uref().unwrap_or_revert();
    //     storage::write(holders_cnt_uref, 1u64);

    //     let approved_cnt_uref = runtime::get_key(NAMED_KEY_APPROVED_CNT).unwrap_or_revert().into_uref().unwrap_or_revert();
    //     storage::write(approved_cnt_uref, 1u64);

    //     let prod_approved_cnt_uref = runtime::get_key(NAMED_KEY_APPROVED_CNT).unwrap_or_revert().into_uref().unwrap_or_revert();
    //     storage::write(prod_approved_cnt_uref, 1u64);

    //     let reqs_cnt_uref = runtime::get_key(NAMED_KEY_REQ_CNT).unwrap_or_revert().into_uref().unwrap_or_revert();
    //     storage::write(reqs_cnt_uref, 1u64);
    // }
    

}
