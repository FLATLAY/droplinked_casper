use core::ops::{Mul, Sub, Div};

use alloc::{string::{String, ToString}, vec::Vec};
use casper_contract::{contract_api::{runtime::{self, revert, get_caller, get_blocktime}, 
    system::{get_purse_balance, transfer_from_purse_to_account}, storage}, 
    unwrap_or_revert::UnwrapOrRevert};
use casper_types::{Key, ApiError, account::AccountHash, U512, BlockTime};
use crate::{ndpc_utils::{get_ratio_verifier, verify_signature, get_latest_timestamp, set_latest_timestamp, self, get_nft_metadata}, 
    constants::{
            RUNTIME_ARG_CURRENT_PRICE_TIMESTAMP, RUNTIME_ARG_SIGNATURE, RUNTIME_ARG_APPROVED_ID, 
            RUNTIME_ARG_AMOUNT, NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_APPROVED_NAME, 
            NAMED_KEY_DICT_PUBAPPROVED_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME, 
            NAMED_KEY_DICT_HOLDERS_NAME, NAMED_KEY_DICT_METADATAS_NAME, NAMED_KEY_HOLDERSCNT
        }, Error, ndpc_types::{AsStrized, ApprovedNFT, self, U64list}, event::{DropLinkedEvent, emit}};

#[no_mangle]
pub extern "C" fn buy(){
    let ratio_verifier = get_ratio_verifier();
    let mp = runtime::get_named_arg::<String>(RUNTIME_ARG_CURRENT_PRICE_TIMESTAMP);
    let sig = runtime::get_named_arg(RUNTIME_ARG_SIGNATURE);

    let approved_id : u64 = runtime::get_named_arg(RUNTIME_ARG_APPROVED_ID);
    let amount : u64 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    //get purse from runtime args
    let purse = {
        let purse_key : Key = runtime::get_named_arg("purse_addr");
        purse_key.into_uref().unwrap_or_revert()
    };

    if !verify_signature(ratio_verifier, sig, mp.clone()){
        revert(ApiError::from(Error::InvalidSignature));
    }
    
    let m_price = mp.split(',').collect::<Vec<&str>>();
    let price_rat = m_price[0].parse::<u64>().unwrap();
    let provided_block_height = m_price[1].parse::<u64>().unwrap();
    //let latest_timestamp = get_latest_timestamp();
    let latest_block_time = get_blocktime();
    if provided_block_height <= latest_block_time.try_into().unwrap(){
        revert(ApiError::from(Error::InvalidTimestamp));
    }
    //set_latest_timestamp(current_timestamp);

    //define storages we need to work with
    let owners_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_OWNERS_NAME);
    let approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_APPROVED_NAME);
    let publishers_approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PUBAPPROVED_NAME);
    let producers_approved_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PRODAPPROVED_NAME);
    let holders_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_HOLDERS_NAME);
    let metadata_dict = ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_METADATAS_NAME);
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
    let amount_to_pay = price.mul(amount*price_rat);
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
    let holders_cnt_uref = ndpc_utils::get_named_key_by_name(NAMED_KEY_HOLDERSCNT);
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
    let caller_tokens_opt = storage::dictionary_get::<U64list>(owners_dict, &caller_string).unwrap_or_revert();
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
    emit(DropLinkedEvent::Buy { amount, approved_id, buyer: get_caller()});
}