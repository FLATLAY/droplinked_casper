use alloc::{string::ToString, vec};
use casper_contract::contract_api::storage;
use casper_types::{EntryPoint, EntryPoints, contracts::{Parameters, NamedKeys}, Parameter, system::auction::ARG_AMOUNT, Group};

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
pub const NAMED_KEY_DICT_TOTAL_SUPPLY : &str = "total_supply";


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

pub const CONTRACTPACKAGEHASH : &str = "droplink_package_hash";

pub fn get_entrypoints() -> EntryPoints{
    let mut result =EntryPoints::new();

    let mint_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_METADATA.to_string(), casper_types::CLType::String),
        Parameter::new(RUNTIME_ARG_AMOUNT.to_string(), casper_types::CLType::U64),
        Parameter::new(RUNTIME_ARG_RECIPIENT.to_string(), casper_types::CLType::Key),
        Parameter::new("price".to_string(), casper_types::CLType::U256)
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
    let get_total_supply_parameters : Parameters = vec![
        Parameter::new(RUNTIME_ARG_TOKEN_ID, casper_types::CLType::U64)
    ];
    let get_token_parameters : Parameters = vec![Parameter::new(RUNTIME_ARG_TOKEN_ID, casper_types::CLType::U64)];

    //EntryPoints declaration here
    //TODO: Access point should be groups not public 
    let entry_point_mint = EntryPoint::new("mint", mint_parameters  ,casper_types::CLType::U64,casper_types::EntryPointAccess::Public,casper_types::EntryPointType::Contract);
    let entry_point_approve = EntryPoint::new("approve", approve_parameters , casper_types::CLType::U64,casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_disapprove = EntryPoint::new("disapprove" , disapprove_paramters , casper_types::CLType::Unit , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_buy = EntryPoint::new("buy" , buy_parameters , casper_types::CLType::Unit , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_get_tokens = EntryPoint::new("get_tokens" , Parameters::new() , casper_types::CLType::String , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_get_token = EntryPoint::new("get_token" , get_token_parameters , casper_types::CLType::String , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_init = EntryPoint::new("init" , Parameters::new() , casper_types::CLType::Unit , casper_types::EntryPointAccess::Groups(vec![Group::new("constructor")]) , casper_types::EntryPointType::Contract);
    let entry_point_publish_request = EntryPoint::new("publish_request" , publish_request_parameters , casper_types::CLType::U64 , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_cancel_request = EntryPoint::new("cancel_request" , cancel_request_parameters , casper_types::CLType::Unit , casper_types::EntryPointAccess::Public , casper_types::EntryPointType::Contract);
    let entry_point_get_total_supply = EntryPoint::new("get_total_supply", get_total_supply_parameters, casper_types::CLType::U64, casper_types::EntryPointAccess::Public, casper_types::EntryPointType::Contract);

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
    result.add_entry_point(entry_point_get_total_supply);
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