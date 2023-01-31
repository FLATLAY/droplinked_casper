use alloc::{string::ToString, vec};
use casper_contract::contract_api::storage;
use casper_types::{EntryPoint, EntryPoints, contracts::{Parameters, NamedKeys}, Parameter, system::auction::ARG_AMOUNT, Group};
// constant named keys and runtime arguments
pub const NAMED_KEY_DICT_APPROVED_NAME: &str = "approved";
pub const NAMED_KEY_DICT_HOLDERS_NAME: &str = "holders";
pub const NAMED_KEY_DICT_OWNERS_NAME: &str = "owners";
pub const NAMED_KEY_DICT_PUBAPPROVED_NAME: &str = "publishers_approved";
pub const NAMED_KEY_DICT_PRODAPPROVED_NAME: &str = "producers_approved";
pub const NAMED_KEY_DICT_METADATAS_NAME: &str = "metadatas";
pub const NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME: &str = "token_id_by_hash";
pub const NAMED_KEY_TOKENSCNT : &str = "tokens_cnt";
pub const NAMED_KEY_HOLDERSCNT : &str = "holders_cnt";
pub const NAMED_KEY_APPROVED_CNT : &str = "approved_cnt";
pub const NAMED_KEY_REQ_CNT : &str = "request_cnt";
pub const NAMED_KEY_DICT_REQ_OBJ : &str = "request_objects";
pub const NAMED_KEY_DICT_PROD_REQS : &str = "producer_requests";
pub const NAMED_KEY_DICT_PUB_REQS : &str = "publiser_requests";
pub const NAMED_KEY_DICT_PUB_REJS : &str = "publisher_rejects";

pub const RUNTIME_ARG_METADATA : &str = "metadata";
pub const RUNTIME_ARG_AMOUNT : &str = "amount";
pub const RUNTIME_ARG_RECIPIENT : &str = "recipient";
pub const RUNTIME_ARG_HOLDER_ID : &str = "holder_id";
pub const RUNTIME_ARG_SPENDER : &str = "publisher-account";
pub const RUNTIME_ARG_APPROVED_ID : &str = "approved_id";
pub const RUNTIME_ARG_TOKEN_ID : &str = "token_id";
pub const RUNTIME_ARG_COMISSION : &str = "comission";
pub const RUNTIME_ARG_PRODUCER_ACCOUNT_HASH : &str = "producer-account";
pub const RUNTIME_ARG_REQUEST_ID : &str = "request_id";
pub const RUNTIME_ARG_PRICE : &str = "price";

pub const ENTRYPOINT_MINT : &str = "mint";
pub const ENTRYPOINT_APPROVE : &str = "approve";
pub const ENTRYPOINT_DISAPPROVE : &str = "disapprove";
pub const ENTRYPOINT_BUY : &str = "buy";
pub const ENTRYPOINT_PUBLISH_REQUEST : &str = "publish_request";
pub const ENTRYPOINT_CANCEL_REQUEST : &str = "cancel_request";
pub const ENTRYPOINT_GET_TOKENS : &str = "get_tokens";
pub const ENTRYPOINT_GET_TOKEN : &str = "get_tokens";
pub const ENTRYPOINT_INIT : &str = "init";

pub const GROUP_CONSTRUCTOR : &str = "constructor";

pub const PACKAGE_HASH_NAME : &str = "droplink_package_hash";
pub const CONTRACT_NAME : &str = "droplinked_contract";

pub fn get_entrypoints() -> EntryPoints{
    let mut result =EntryPoints::new();
    let mint_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_METADATA.to_string(), casper_types::CLType::String),
        Parameter::new(RUNTIME_ARG_AMOUNT.to_string(), casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_RECIPIENT.to_string(), casper_types::CLType::Key),
        Parameter::new(RUNTIME_ARG_PRICE.to_string(), casper_types::CLType::U256)
    ];
    let approve_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_REQUEST_ID, casper_types::CLType::U64)
    ]; 
    let disapprove_paramters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_AMOUNT, casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_APPROVED_ID, casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_SPENDER, casper_types::CLType::Key)
    ];    
    let buy_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_AMOUNT, casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_APPROVED_ID, casper_types::CLType::U64)
    ];
    let publish_request_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_PRODUCER_ACCOUNT_HASH, casper_types::CLType::Key),    
        Parameter::new(RUNTIME_ARG_AMOUNT, casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_HOLDER_ID, casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_COMISSION, casper_types::CLType::U8)
    ];
    let cancel_request_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_REQUEST_ID, casper_types::CLType::U64)
    ];
    let get_token_parameters : Parameters = vec![Parameter::new(RUNTIME_ARG_TOKEN_ID, casper_types::CLType::U64)];

    //EntryPoints declaration here
    let entry_point_mint = EntryPoint::new(ENTRYPOINT_MINT, mint_parameters  ,casper_types::CLType::U64,casper_types::EntryPointAccess::Public,casper_types::EntryPointType::Contract);
    let entry_point_approve = EntryPoint::new(ENTRYPOINT_APPROVE, approve_parameters , casper_types::CLType::U64,casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_disapprove = EntryPoint::new(ENTRYPOINT_DISAPPROVE , disapprove_paramters , casper_types::CLType::Unit , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_buy = EntryPoint::new(ENTRYPOINT_BUY , buy_parameters , casper_types::CLType::Unit , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_get_tokens = EntryPoint::new(ENTRYPOINT_GET_TOKENS , Parameters::new() , casper_types::CLType::String , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_get_token = EntryPoint::new(ENTRYPOINT_GET_TOKEN , get_token_parameters , casper_types::CLType::String , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_init = EntryPoint::new(ENTRYPOINT_INIT , Parameters::new() , casper_types::CLType::Unit , casper_types::EntryPointAccess::Groups(vec![Group::new(GROUP_CONSTRUCTOR)]) , casper_types::EntryPointType::Contract);
    let entry_point_publish_request = EntryPoint::new(ENTRYPOINT_PUBLISH_REQUEST , publish_request_parameters , casper_types::CLType::U64 , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_cancel_request = EntryPoint::new(ENTRYPOINT_CANCEL_REQUEST , cancel_request_parameters , casper_types::CLType::Unit , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);

    //add all created entrypoints here
    result.add_entry_point(entry_point_mint);
    result.add_entry_point(entry_point_approve);
    result.add_entry_point(entry_point_disapprove);
    result.add_entry_point(entry_point_buy);
    result.add_entry_point(entry_point_get_tokens);
    result.add_entry_point(entry_point_get_token);
    result.add_entry_point(entry_point_init);
    result.add_entry_point(entry_point_publish_request);
    result.add_entry_point(entry_point_cancel_request);
    result
}

pub fn get_named_keys() -> alloc::collections::BTreeMap<alloc::string::String, casper_types::Key>{
    let mut named_keys : NamedKeys = NamedKeys::new();
    named_keys.insert(NAMED_KEY_APPROVED_CNT.to_string(), storage::new_uref(0u64).into());
    named_keys.insert(NAMED_KEY_HOLDERSCNT.to_string(), storage::new_uref(0u64).into());
    named_keys.insert(NAMED_KEY_TOKENSCNT.to_string(), storage::new_uref(0u64).into());
    named_keys.insert(NAMED_KEY_REQ_CNT.to_string(), storage::new_uref(0u64).into());
    named_keys
}