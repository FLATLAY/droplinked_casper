#![no_std]
#![no_main]
mod ndpc_types;
mod ndpc_utils;
mod constants;
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;
use core::ops::{Mul, Div, Sub};
use alloc::{string::{String, ToString}, collections::BTreeSet, vec::Vec};
use casper_contract::{contract_api::{runtime::{self, get_caller}, storage, system::{get_purse_balance, transfer_from_purse_to_account}}, unwrap_or_revert::UnwrapOrRevert};
use constants::{get_entrypoints, get_named_keys, ENTRYPOINT_INIT};
use ndpc_types::{NFTHolder, ApprovedNFT, U64list,AsStrized, PublishRequest};
use casper_types::{RuntimeArgs, U256, Key, account::AccountHash, ApiError, URef, U512, ContractPackageHash, CLValue};
/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
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
    ProducerHasNoApprovedHolders = 14,
    EmptyRequestCnt = 15,
    AccessDenied = 16,
    EmptyU64List = 17,
}
impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[no_mangle]
pub extern "C" fn mint(){
    //Get runtime arguments from the caller
    let metadata : String = runtime::get_named_arg(constants::RUNTIME_ARG_METADATA);
    let price : U256 = runtime::get_named_arg(constants::RUNTIME_ARG_PRICE);
    let amount : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_AMOUNT);
    let reciver_key : Key = runtime::get_named_arg(constants::RUNTIME_ARG_RECIPIENT);
    let reciver_acc = reciver_key.into_account().unwrap_or_revert_with(ApiError::from(Error::NotAccountHash));
    let reciver : String = reciver_acc.as_string();
    //create the metadata from it's string representation and calculate the hash
    let generated_metadata_res = ndpc_types::NftMetadata::from_json(metadata,price);
    if generated_metadata_res.is_err(){
        runtime::revert(ApiError::from(Error::MintMetadataNotValid));
    }
    let generated_metadata = generated_metadata_res.unwrap();
    let metadata_hash = generated_metadata.get_hash().as_string();
    
    //dictionaries and urefs here
    let token_id_by_hash_dict_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME);
    let metadata_by_token_id_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_METADATAS_NAME);
    let nft_holder_by_id_dict_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_HOLDERS_NAME);
    let holders_cnt_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_HOLDERSCNT);
    let owners_dict_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_OWNERS_NAME);
    let tokens_cnt_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_TOKENSCNT);

    let mut _token_id_final : u64 = 0u64;
    let _token_id : u64 = 0u64;
    match storage::dictionary_get(token_id_by_hash_dict_uref, &metadata_hash).unwrap_or_revert(){
        // if the token id is not found in the dictionary, then we create a new one
        None => {
            let tokens_cnt:u64 = storage::read(tokens_cnt_uref).unwrap_or_revert().unwrap_or_revert();
            _token_id_final = tokens_cnt + 1u64;
            storage::write(tokens_cnt_uref,_token_id_final);
            storage::dictionary_put(token_id_by_hash_dict_uref, &metadata_hash, _token_id_final);
        },
        // if the token id is found in the dictionary, then we use it
        Some(_token_id) => {
            _token_id_final = _token_id;
        }
    }
    //add the token_id generated (or retrieved) to the metadatas dictioanary (with the actual metadata)
    storage::dictionary_put(metadata_by_token_id_uref, _token_id_final.to_string().as_str(), generated_metadata.to_string());
    //Create an NFTHolder object for the reciver
    let nft_holder = NFTHolder::new(amount, amount, _token_id_final);
    
    let holders_cnt : u64 = storage::read(holders_cnt_uref).unwrap_or_revert().unwrap_or_revert();
    let holder_id : u64 = holders_cnt+ 1u64;
    storage::write(holders_cnt_uref, holder_id);
    storage::dictionary_put(nft_holder_by_id_dict_uref, holder_id.to_string().as_str(), nft_holder);    
    
    let owner_holder_ids = storage::dictionary_get(owners_dict_uref, reciver.as_str()).unwrap_or_revert();
    //create the list if it did not exist
    if owner_holder_ids.is_none(){
        let mut new_list = ndpc_types::U64list::new();
        new_list.list.push(holder_id);
        storage::dictionary_put(owners_dict_uref, reciver.as_str(), new_list);
    }
    else{
        let mut owner_holder_ids : ndpc_types::U64list = owner_holder_ids.unwrap_or_revert();
        owner_holder_ids.list.push(holder_id);
        storage::dictionary_put(owners_dict_uref, reciver.as_str(), owner_holder_ids);
    }
    // return the token_id
    let ret = CLValue::from_t(_token_id_final).unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn approve(){
    // check if the approved_id does not exist in the list of approved_ids of publisher and producer    
    let requests_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_REQ_OBJ);
    let prod_reqs_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PROD_REQS);
    let pub_reqs_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PUB_REQS);
    
    //define the runtime arguments needed for this entrypoint
    let request_id : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_REQUEST_ID);
    //get the request object from the dictionary
    let request_obj_string = storage::dictionary_get::<String>(requests_dict, request_id.to_string().as_str()).unwrap_or_revert().unwrap_or_revert();
    let request_obj = PublishRequest::from_string(request_obj_string);

    let amount : u64 = request_obj.amount;
    let holder_id : u64 = request_obj.holder_id;

    let spender_key : Key = Key::Account(request_obj.publisher);
    let spender_acc : AccountHash = spender_key.into_account().unwrap_or_revert_with(ApiError::from(Error::NotAccountHash));
    let spender : String = spender_acc.as_string();
    
    //define storages we need to work with
    let owners_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_OWNERS_NAME);
    let holders_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_HOLDERS_NAME);
    let publishers_approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PUBAPPROVED_NAME);
    let producers_approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PRODAPPROVED_NAME);
    let approved_cnt_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_APPROVED_CNT);
    let approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_APPROVED_NAME);

    let caller_account = runtime::get_caller();
    let caller : String = caller_account.as_string();
    //let caller : String = sender;
    let caller_holder_ids = storage::dictionary_get::<U64list>(owners_dict, caller.as_str())
        .unwrap_or_revert().unwrap_or_revert_with(ApiError::from(Error::NoTokensFound));
    let mut found : bool = false;
    for caller_holder_id in caller_holder_ids.list{
        if caller_holder_id == holder_id{
            found = true;
            break;
        }
    }
    if !found{
        //the caller does not own the token with the given holder_id
        runtime::revert(ApiError::from(Error::NotOwnerOfHolderId));
    }

    let mut holder : NFTHolder = storage::dictionary_get(holders_dict, holder_id.to_string().as_str()).unwrap_or_revert().unwrap_or_revert();
    if holder.remaining_amount < amount{
        //the caller does not own enough tokens
        runtime::revert(ApiError::from(Error::NotEnoughTokens));
    }
    
    //update the remaining amount of the holder
    holder.remaining_amount -= amount;
    //create the approved holder
    let approved_holder = ApprovedNFT::new(holder_id, amount , caller_account, spender_acc, holder.token_id, request_obj.percentage);
    
    storage::dictionary_put(holders_dict, holder_id.to_string().as_str(), holder); //copy i g

    //get approved_cnt, increment it and save it 
    let approved_cnt : u64 = storage::read(approved_cnt_uref).unwrap_or_revert().unwrap_or_revert();
    let new_approved_cnt = approved_cnt + 1;
    storage::write(approved_cnt_uref, new_approved_cnt);

    let approved_id = new_approved_cnt;
    //save the approved holder
    storage::dictionary_put(approved_dict, approved_id.to_string().as_str(), approved_holder);

    //add the approved holder to the publishers approved dictionary
    let publisher_approved_holders_opt = storage::dictionary_get(publishers_approved_dict, &spender).unwrap_or_revert();
    if publisher_approved_holders_opt.is_none(){
        let mut new_list = ndpc_types::U64list::new();
        new_list.list.push(approved_id);
        storage::dictionary_put(publishers_approved_dict, &spender, new_list);
    }
    else{
        let mut publisher_approved_holders : ndpc_types::U64list = publisher_approved_holders_opt.unwrap_or_revert();
        publisher_approved_holders.list.push(approved_id);
        storage::dictionary_put(publishers_approved_dict, &spender, publisher_approved_holders);
    }
    
    //add the approved holder to the producers approved dictionary
    let producer_approved_holders_opt = storage::dictionary_get(producers_approved_dict, &caller).unwrap_or_revert();
    if producer_approved_holders_opt.is_none(){
        let mut new_list = ndpc_types::U64list::new();
        new_list.list.push(approved_id);
        storage::dictionary_put(producers_approved_dict, &caller, new_list);
    }
    else{
        let mut producer_approved_holders : ndpc_types::U64list = producer_approved_holders_opt.unwrap_or_revert();
        producer_approved_holders.list.push(approved_id);
        storage::dictionary_put(producers_approved_dict, &caller, producer_approved_holders);
    }
    
    //remove the request from the publishers requests dictionary and the producers requests dictionary
    let publisher_requests_opt = storage::dictionary_get::<U64list>(pub_reqs_dict, &spender).unwrap_or_revert();
    let mut publisher_requests : U64list = publisher_requests_opt.unwrap_or_revert();
    publisher_requests.remove(request_id);
    storage::dictionary_put(pub_reqs_dict, &spender, publisher_requests);

    let producer_requests_opt = storage::dictionary_get::<U64list>(prod_reqs_dict, &caller).unwrap_or_revert();
    let mut producer_requests : U64list = producer_requests_opt.unwrap_or_revert();
    producer_requests.remove(request_id);
    storage::dictionary_put(prod_reqs_dict, &caller, producer_requests);

    //return the approved_id
    let ret = CLValue::from_t(approved_id).unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn disapprove(){
    //check if the caller is the owner of the token
    //define the runtime arguments needed for this entrypoint
    let amount : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_AMOUNT);
    let approved_id : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_APPROVED_ID);
    let spender_key : Key = runtime::get_named_arg(constants::RUNTIME_ARG_SPENDER); //spender is the publisher
    let spender_acc = spender_key.into_account().unwrap_or_revert_with(ApiError::from(Error::NotAccountHash));
    let spender : String = spender_acc.as_string();
    //define storages we need to work with
    let approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_APPROVED_NAME);
    let publishers_approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PUBAPPROVED_NAME);
    let producers_approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PRODAPPROVED_NAME);
    let holders_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_HOLDERS_NAME);

    //from the approved_id, get the approvednft
    let mut approved_holder = storage::dictionary_get::<ApprovedNFT>(approved_dict, approved_id.to_string().as_str())
        .unwrap_or_revert().unwrap_or_revert_with(ApiError::from(Error::ApprovedHolderDoesentExist));
    //check if the caller is the owner of the token
    let caller = runtime::get_caller();
    if caller != approved_holder.owneraccount{
        //the caller is not the owner of the token
        runtime::revert(ApiError::from(Error::NotOwnerOfHolderId));
    }
    let caller_string = caller.as_string();

    //if amount was not enough, revert
    if approved_holder.amount < amount{
        runtime::revert(ApiError::from(Error::NotEnoughAmount));
    }
    //else, approvednft's amount -= amount
    approved_holder.amount -= amount;

    if approved_holder.amount == 0 {
        {
            //remove the approvednft from the u64list of publisher
            let mut publisher_approved_holders = storage::dictionary_get::<ndpc_types::U64list>(publishers_approved_dict, &spender)
                .unwrap_or_revert()
                .unwrap_or_revert_with(ApiError::from(Error::PublisherHasNoApprovedHolders));
            publisher_approved_holders.remove(approved_id);
            storage::dictionary_put(publishers_approved_dict, &spender, publisher_approved_holders);
        }
        {
            //remove the approvednft from the u64list of producer
            let mut producer_approved_holders = storage::dictionary_get::<ndpc_types::U64list>(producers_approved_dict, caller_string.as_str())
                .unwrap_or_revert()
                .unwrap_or_revert_with(ApiError::from(Error::ProducerHasNoApprovedHolders));
            producer_approved_holders.remove(approved_id);
            storage::dictionary_put(producers_approved_dict, caller_string.as_str(), producer_approved_holders);
        }
    }

    let holder_id = approved_holder.holder_id;
    
    //put back approved_holder in the dictionary
    storage::dictionary_put(approved_dict, approved_id.to_string().as_str(), approved_holder);

    //from the approved holder, get the holder_id and then the nftholder
    let mut holder = storage::dictionary_get::<NFTHolder>(holders_dict, holder_id.to_string().as_str()).unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::HolderDoesentExist));
    holder.remaining_amount += amount;
    //put back holder to the dictionary
    storage::dictionary_put(holders_dict, holder_id.to_string().as_str(), holder);
}

#[no_mangle]
pub extern "C" fn buy(){
    let approved_id : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_APPROVED_ID);
    let amount : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_AMOUNT);
    //get purse from runtime args
    let purse = {
        let purse_key : Key = runtime::get_named_arg("purse_addr");
        purse_key.into_uref().unwrap_or_revert()
    };
    //define storages we need to work with
    let owners_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_OWNERS_NAME);
    let approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_APPROVED_NAME);
    let publishers_approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PUBAPPROVED_NAME);
    let producers_approved_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PRODAPPROVED_NAME);
    let holders_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_HOLDERS_NAME);
    let metadata_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_METADATAS_NAME);
    let purse_balance = get_purse_balance(purse).unwrap_or_revert();
    let caller_string = get_caller().as_string();

    let mut approved_holder = storage::dictionary_get::<ApprovedNFT>(approved_dict, approved_id.to_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::ApprovedHolderDoesentExist));
    
    let producer_hash : AccountHash = approved_holder.owneraccount;
    let publisher_hash : AccountHash = approved_holder.publisheraccount;
    let producer_string : String = producer_hash.as_string();
    let publisher_string : String = publisher_hash.as_string();

    //check if amount <= approvednft's amount
    if amount > approved_holder.amount{
        //amount is not enough
        runtime::revert(ApiError::from(Error::NotEnoughAmount));
    }
    //first get the metadata from the token_id(from the metadatas dict)
    let token_id = approved_holder.token_id;
    let metadata = get_nft_metadata(token_id.to_string(), metadata_dict);
    let price : U512 = U512::from_dec_str(metadata.price.to_string().as_str()).unwrap_or_default();
    let amount_to_pay = price.mul(amount);
    // transfers the amount of money to the owner
    let publisher_percent : U512 = approved_holder.percentage.into();
    let producer_percent : U512 = U512::from(100u64).sub(publisher_percent);
    let one_hundred : U512 = 100u64.into();
    let producer_part = amount_to_pay.mul(producer_percent).div(one_hundred);
    let publisher_part = amount_to_pay.sub(producer_part);

    if purse_balance < amount_to_pay{
        //not enough balance
        runtime::revert(ApiError::from(Error::NotEnoughBalance));
    }
    //transfer to producer
    let result_prod = transfer_from_purse_to_account(purse, producer_hash, producer_part, None);
    if result_prod.is_err(){
        //transfer failed
        runtime::revert(ApiError::from(Error::TransferFailed));
    }
    //transfer to publisher
    let result_pub = transfer_from_purse_to_account(purse, publisher_hash, publisher_part, None);
    if result_pub.is_err(){
        //transfer failed
        runtime::revert(ApiError::from(Error::TransferFailed));
    }
    //update approved holder and holder amounts
    approved_holder.amount -= amount;
    //update holder using approved_holder.holder_id
    let holder_opt = storage::dictionary_get::<ndpc_types::NFTHolder>(holders_dict, approved_holder.holder_id.to_string().as_str()).unwrap_or_revert();
    if holder_opt.is_none(){
        //the holder does not exist
        runtime::revert(ApiError::from(Error::HolderDoesentExist));
    }
    let mut holder : ndpc_types::NFTHolder = holder_opt.unwrap_or_revert();
    holder.amount -= amount;

    storage::dictionary_put(holders_dict, approved_holder.holder_id.to_string().as_str(), holder);
    //if approved holder amount is 0, remove it from publisher and producer's approved lists
    if approved_holder.amount == 0{
        //remove from publisher's approved list
        let mut publisher_approved_list = storage::dictionary_get::<U64list>(publishers_approved_dict, publisher_string.as_str())
            .unwrap_or_revert()
            .unwrap_or_revert_with(ApiError::from(Error::ApprovedListDoesentExist));
        publisher_approved_list.remove(approved_id);
        storage::dictionary_put(publishers_approved_dict, publisher_string.as_str(), publisher_approved_list);
        //remove from producer's approved list
        
        let mut producer_approved_list = storage::dictionary_get::<U64list>(producers_approved_dict, producer_string.as_str())
            .unwrap_or_revert()
            .unwrap_or_revert_with(ApiError::from(Error::ApprovedListDoesentExist));
        producer_approved_list.remove(approved_id);
        storage::dictionary_put(producers_approved_dict, producer_string.as_str(), producer_approved_list);
    }
    let token_id = approved_holder.token_id;
    //update approved holder
    storage::dictionary_put(approved_dict, approved_id.to_string().as_str(), approved_holder);
    //creates new nftholder and adds it to the holders dict and gets holder_id from it and adds it to callers list(if list didn't exist, create it)
    let new_holder = ndpc_types::NFTHolder::new(amount, amount, token_id);
    //get new holder id
    let holders_cnt_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_HOLDERSCNT);
    let mut holders_cnt : u64 = storage::read(holders_cnt_uref).unwrap_or_revert().unwrap_or_revert();
    holders_cnt += 1;
    storage::write(holders_cnt_uref, holders_cnt);
    let new_holder_id = holders_cnt;
    //add new holder to holders dict
    storage::dictionary_put(holders_dict, new_holder_id.to_string().as_str(), new_holder);
    //add new holder id to callers list
    let caller_list_opt = storage::dictionary_get::<U64list>(holders_dict, &caller_string).unwrap_or_revert();
    if caller_list_opt.is_none(){
        //the caller list does not exist
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(holders_dict, &caller_string, new_list);
    }else{
        let mut caller_list = caller_list_opt.unwrap_or_revert();
        caller_list.add(new_holder_id);
        storage::dictionary_put(holders_dict, &caller_string, caller_list);
    }
    //get caller's tokens from owners_dict
    let mut caller_tokens_opt = storage::dictionary_get::<U64list>(owners_dict, &caller_string).unwrap_or_revert();
    if caller_tokens_opt.is_none(){
        //the caller tokens list does not exist
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(owners_dict, &caller_string, new_list);
    }
    else{
        let mut caller_tokens = caller_tokens_opt.unwrap_or_revert();
        //add holder_id to caller's tokens
        caller_tokens.add(new_holder_id);
        //update caller's tokens
        storage::dictionary_put(owners_dict, &caller_string, caller_tokens);
    }
}

fn get_nft_metadata(token_id : String , metadatas_dict : URef) -> ndpc_types::NftMetadata{
    let metadata_opt = storage::dictionary_get::<String>(metadatas_dict, token_id.as_str()).unwrap_or_revert();
    if metadata_opt.is_none(){
        //the metadata does not exist
        runtime::revert(ApiError::from(Error::MetadataDoesentExist));
    }
    let metadata_string = metadata_opt.unwrap_or_revert();
    //split by , => [name , token_uri , checksum , price]
    let metadata_split = metadata_string.split(",").collect::<Vec<&str>>();
    let name = metadata_split[0].to_string();
    let token_uri = metadata_split[1].to_string();
    let checksum = metadata_split[2].to_string();
    let price = U256::from_dec_str(metadata_split[3]).unwrap();
    let metadata = ndpc_types::NftMetadata::new(name, token_uri, checksum, price);
    return metadata;
}

#[no_mangle]
pub extern "C" fn get_tokens(){
    let owner_hash = get_caller();
    let owner = owner_hash.as_string();
    let owners_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_OWNERS_NAME);
    let holders_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_HOLDERS_NAME);
    let metadatas_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_METADATAS_NAME);
    let owners_list = storage::dictionary_get::<U64list>(owners_dict, owner.as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::EmptyOwnerShipList));
        let mut description = String::from("");
    for holder_id in owners_list.list{
        description+= "[";
        let holder = storage::dictionary_get::<ndpc_types::NFTHolder>(holders_dict, holder_id.to_string().as_str())
            .unwrap_or_revert()
            .unwrap_or_revert_with(ApiError::from(Error::HolderDoesentExist));
        description+= holder.to_string().as_str();
        description+= " , ";
        let token = get_nft_metadata(holder.token_id.to_string(), metadatas_dict);
        description+= token.to_json().as_str();
        description+= "]";
    }
    let ret = CLValue::from_t(description).unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn get_token(){
    let token_id = runtime::get_named_arg::<u64>(constants::RUNTIME_ARG_TOKEN_ID);
    let metadatas_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_METADATAS_NAME);
    let token = get_nft_metadata(token_id.to_string(), metadatas_dict);
    let ret = CLValue::from_t(token.to_json()).unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn publish_request(){
    //storages we need to work with
    let holders_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_HOLDERS_NAME);
    let owners_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_OWNERS_NAME);
    let requests_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_REQ_OBJ);
    let prod_reqs_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PROD_REQS);
    let pub_reqs_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PUB_REQS);
    //runtime args
    let producer_account_hash = runtime::get_named_arg::<Key>(constants::RUNTIME_ARG_PRODUCER_ACCOUNT_HASH).into_account().unwrap_or_revert();
    let holder_id = runtime::get_named_arg::<u64>(constants::RUNTIME_ARG_HOLDER_ID);
    let amount = runtime::get_named_arg::<u64>(constants::RUNTIME_ARG_AMOUNT);
    let comission = runtime::get_named_arg::<u8>(constants::RUNTIME_ARG_COMISSION);
    let caller = get_caller().as_string();
    let producer_string = producer_account_hash.as_string();
    //get holder by id
    let holder = storage::dictionary_get::<ndpc_types::NFTHolder>(holders_dict, holder_id.to_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::HolderDoesentExist));
    //if holder.amount < amount  revert
    if holder.amount < amount{
        runtime::revert(ApiError::from(Error::NotEnoughAmount));
    }
    //check if holder_id exists in owners_dict (producer as the key)
    let prod_list = storage::dictionary_get::<U64list>(owners_dict, producer_string.as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::EmptyOwnerShipList));    
    let mut is_owner = false;
    for id in prod_list.list{
        if id == holder_id{
            is_owner = true;
            break;
        }
    }
    if !is_owner{
        runtime::revert(ApiError::from(Error::NotOwnerOfHolderId));
    }
    //create publish request
    let publish_request = ndpc_types::PublishRequest::new(holder_id, amount, comission,producer_account_hash,get_caller());
    let tokens_cnt_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_REQ_CNT);
    let request_cnt = storage::read::<u64>(tokens_cnt_uref).unwrap_or_revert().unwrap_or_revert_with(ApiError::from(Error::EmptyRequestCnt));
    let request_id = request_cnt + 1;
    storage::write(tokens_cnt_uref, request_id);
    storage::dictionary_put(requests_dict, request_id.to_string().as_str(),publish_request.to_string());
    //add request to producer requests
    let prod_reqs_opt = storage::dictionary_get::<U64list>(prod_reqs_dict, producer_string.as_str())
        .unwrap_or_revert();
    let mut prod_reqs = match prod_reqs_opt{
        Some(reqs) => reqs,
        None => U64list::new(),
    };
    prod_reqs.list.push(request_id);
    storage::dictionary_put(prod_reqs_dict, producer_string.as_str(), prod_reqs);
    //add request to publisher requests
    let pub_reqs_opt = storage::dictionary_get::<U64list>(pub_reqs_dict, caller.as_str())
        .unwrap_or_revert();
    let mut pub_reqs = match pub_reqs_opt{
        Some(reqs) => reqs,
        None => U64list::new(),
    };
    pub_reqs.list.push(request_id);
    storage::dictionary_put(pub_reqs_dict, caller.as_str(), pub_reqs);

    let ret = CLValue::from_t(request_id).unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn cancel_request(){
    //storages we need to work with
    let requests_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_REQ_OBJ);
    let prod_reqs_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PROD_REQS);
    let pub_reqs_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_PUB_REQS);
    //runtime args
    let request_id : u64 = runtime::get_named_arg(constants::RUNTIME_ARG_REQUEST_ID);
    let caller : String = get_caller().as_string();

    //get request from requests_dict using request_id
    let req_string : String = storage::dictionary_get(requests_dict, request_id.to_string().as_str()).unwrap_or_revert().unwrap_or_revert();
    let request_obj = ndpc_types::PublishRequest::from_string(req_string);
    //check if request's publisher is the caller
    if request_obj.publisher != get_caller(){
        runtime::revert(ApiError::from(Error::AccessDenied));
    }
    //remove the request_id from the publisher's requests and from the producer's requests
    let mut pub_reqs = storage::dictionary_get::<U64list>(pub_reqs_dict, request_obj.publisher.as_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::EmptyU64List));
    let mut prod_reqs = storage::dictionary_get::<U64list>(prod_reqs_dict, request_obj.producer.as_string().as_str())
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::from(Error::EmptyU64List));
    
    pub_reqs.remove(request_id);
    prod_reqs.remove(request_id);
    storage::dictionary_put(pub_reqs_dict, caller.as_str(), pub_reqs);
    storage::dictionary_put(prod_reqs_dict, request_obj.producer.as_string().as_str(), prod_reqs);
}

#[no_mangle]
pub extern "C" fn init(){
    storage::new_dictionary(constants::NAMED_KEY_DICT_APPROVED_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_HOLDERS_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_METADATAS_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_OWNERS_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_PRODAPPROVED_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_PUBAPPROVED_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_REQ_OBJ).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_PROD_REQS).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_PUB_REQS).unwrap_or_revert();
    storage::new_dictionary(constants::NAMED_KEY_DICT_PUB_REJS).unwrap_or_revert();
}
fn install_contract(){
    // Install the contract
    // Get the entry points and named keys
    let entry_points = get_entrypoints();
    let named_keys = get_named_keys();
    // Create the contract
    let (contract_hash , _contract_version) = storage::new_contract(entry_points, Some(named_keys) , Some(constants::PACKAGE_HASH_NAME.to_string()), None);
    // Create the constructor access uref
    let package_hash = ContractPackageHash::new(runtime::get_key(constants::PACKAGE_HASH_NAME).unwrap_or_revert().into_hash().unwrap_or_revert());
    let constructor_access: URef = storage::create_contract_user_group(package_hash, constants::GROUP_CONSTRUCTOR, 1, Default::default()).unwrap_or_revert().pop().unwrap_or_revert();
    // Call the contract's init function to initialize the contract's dictionaries
    let _: () = runtime::call_contract(contract_hash, ENTRYPOINT_INIT, RuntimeArgs::new());
    // Remove the constructor access uref
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, constants::GROUP_CONSTRUCTOR, urefs).unwrap_or_revert();
    // Put the contract hash in the installer's named keys
    runtime::put_key(constants::CONTRACT_NAME, contract_hash.into());
}

#[no_mangle]
pub extern "C" fn call() {
    install_contract();
}
