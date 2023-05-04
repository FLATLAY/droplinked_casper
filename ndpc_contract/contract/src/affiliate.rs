use alloc::string::{String, ToString};
use casper_contract::{contract_api::{runtime::{self, get_caller, revert}, storage}, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{Key, ApiError, CLValue, account::AccountHash};

use crate::{constants::{RUNTIME_ARG_AMOUNT, NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_REQ_OBJ,
NAMED_KEY_DICT_PROD_REQS, NAMED_KEY_DICT_PUB_REQS, RUNTIME_ARG_REQUEST_ID,
NAMED_KEY_DICT_PUBAPPROVED_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME, 
NAMED_KEY_APPROVED_CNT, NAMED_KEY_DICT_APPROVED_NAME,
RUNTIME_ARG_APPROVED_ID, RUNTIME_ARG_SPENDER, 
NAMED_KEY_REQ_CNT, NAMED_KEY_DICT_HOLDERS_NAME, RUNTIME_ARG_HOLDER_ID, RUNTIME_ARG_COMISSION, self},Error, ndpc_types::{AsStrized, self, NFTHolder, PublishRequest, U64list, ApprovedNFT, NftMetadata}, event::{DropLinkedEvent, emit}, ndpc_utils::{self, get_holder_ids}};

#[no_mangle]
pub extern "C" fn approve(){
    //TODO: Critical : Check for double occurance of approves

    //!!! ---- important TODO: check if the caller account is in the group of producers ----!!!
    //!!! ---- important TODO: check if spender account is in publishers list ----!!!
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
    let metadatas_dict = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_METADATAS_NAME);

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
    if holder.remaining_amount < amount || holder.amount < amount{
        //the caller does not own enough tokens
        runtime::revert(ApiError::from(Error::NotEnoughTokens));
    }
    
    //update the remaining amount of the holder
    holder.remaining_amount -= amount;
    //create the approved holder

    let nft_metadata : NftMetadata = storage::dictionary_get(metadatas_dict, holder.token_id.to_string().as_str()).unwrap_or_revert().unwrap_or_revert();
    let approved_holder = ApprovedNFT::new(holder_id, amount , caller_account, spender_acc, holder.token_id, nft_metadata.comission);
    
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
    emit(DropLinkedEvent::ApprovedPublish { request_id, approved_id });
    runtime::ret(ret);
}
#[no_mangle]
pub extern "C" fn disapprove(){
    
    //check if the caller is the owner of the token
    //define the runtime arguments needed for this entrypoint
    let amount : u64 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    let approved_id : u64 = runtime::get_named_arg(RUNTIME_ARG_APPROVED_ID);
    let spender_key : Key = runtime::get_named_arg(RUNTIME_ARG_SPENDER); //spender is the publisher
    let spender_acc = spender_key.into_account().unwrap_or_revert_with(ApiError::from(Error::NotAccountHash));
    let spender : String = spender_acc.as_string();
    //define storages we need to work with
    let approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_APPROVED_NAME);
    let publishers_approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PUBAPPROVED_NAME);
    let producers_approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PRODAPPROVED_NAME);
    let holders_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_HOLDERS_NAME);

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
    emit(DropLinkedEvent::DisapprovedPublish {  approved_id });
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
    let publish_request = ndpc_types::PublishRequest::new(holder_id, amount,producer_account_hash,get_caller());
    let tokens_cnt_uref = ndpc_utils::get_named_key_by_name(NAMED_KEY_REQ_CNT);
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
    emit(DropLinkedEvent::PublishRequest { owner: producer_account_hash, publisher: get_caller(), amount, holder_id, request_id });
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn cancel_request(){
    //storages we need to work with
    let requests_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_REQ_OBJ);
    let prod_reqs_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PROD_REQS);
    let pub_reqs_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PUB_REQS);
    //runtime args
    let request_id : u64 = runtime::get_named_arg(RUNTIME_ARG_REQUEST_ID);
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
    emit(DropLinkedEvent::CancelRequest { request_id });
}
