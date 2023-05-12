use core::ops::{Add, Div, Mul, Sub};

use crate::{
    constants::{
        NAMED_KEY_DICT_APPROVED_NAME, NAMED_KEY_DICT_HOLDERS_NAME, NAMED_KEY_DICT_METADATAS_NAME,
        NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME,
        NAMED_KEY_DICT_PUBAPPROVED_NAME, NAMED_KEY_HOLDERSCNT, RUNTIME_ARG_AMOUNT,
        RUNTIME_ARG_APPROVED_ID, RUNTIME_ARG_CURRENT_PRICE_TIMESTAMP, RUNTIME_ARG_PURSE_ADDR,
        RUNTIME_ARG_RECIPIENT, RUNTIME_ARG_SHIPPING_PRICE, RUNTIME_ARG_SIGNATURE,
        RUNTIME_ARG_TAX_PRICE, RUNTIME_PRODUCT_PRICE,
    },
    event::{emit, DropLinkedEvent},
    ndpc_types::{self, AsStrized, U64list},
    ndpc_utils::{
        self, calculate_payment, get_approved_holder_by_id, get_droplinked_account,
        get_nft_metadata, get_ratio_verifier, verify_signature,
    },
    Error,
};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{
        runtime::{self, get_blocktime, get_caller, get_named_arg, revert},
        storage,
        system::{
            get_purse_balance, transfer_from_purse_to_account, transfer_from_purse_to_public_key,
        },
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, ApiError, AsymmetricType, Key, PublicKey, URef, U512};

fn get_buy_storage() -> (
    casper_types::URef,
    casper_types::URef,
    casper_types::URef,
    casper_types::URef,
    casper_types::URef,
    casper_types::URef,
) {
    (
        ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_OWNERS_NAME),
        ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_APPROVED_NAME),
        ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PUBAPPROVED_NAME),
        ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_PRODAPPROVED_NAME),
        ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_HOLDERS_NAME),
        ndpc_utils::get_named_key_by_name(NAMED_KEY_DICT_METADATAS_NAME),
    )
}

fn get_buy_runtime_args() -> (
    alloc::string::String,
    alloc::string::String,
    u64,
    u64,
    U512,
    U512,
    URef,
) {
    (
        runtime::get_named_arg::<String>(RUNTIME_ARG_CURRENT_PRICE_TIMESTAMP),
        runtime::get_named_arg::<String>(RUNTIME_ARG_SIGNATURE),
        runtime::get_named_arg::<u64>(RUNTIME_ARG_APPROVED_ID),
        runtime::get_named_arg::<u64>(RUNTIME_ARG_AMOUNT),
        runtime::get_named_arg::<U512>(RUNTIME_ARG_SHIPPING_PRICE),
        runtime::get_named_arg::<U512>(RUNTIME_ARG_TAX_PRICE),
        {
            let purse_key: Key = runtime::get_named_arg(RUNTIME_ARG_PURSE_ADDR);
            purse_key.into_uref().unwrap_or_revert()
        },
    )
}

#[no_mangle]
pub extern "C" fn buy() {
    let ratio_verifier = get_ratio_verifier();
    let (mp, sig, approved_id, amount, shipping_price, tax_price, purse) = get_buy_runtime_args();
    if !verify_signature(ratio_verifier, sig, mp.clone()) {
        revert(ApiError::from(Error::InvalidSignature));
    }
    let m_price = mp.split(',').collect::<Vec<&str>>();
    let price_rat = m_price[0].parse::<u64>().unwrap();
    let provided_block_height = m_price[1].parse::<u64>().unwrap();
    let latest_block_time: u64 = get_blocktime().into();
    // We will let only 2 blocks time for the transaction to happen !
    if provided_block_height <= latest_block_time + 66000u64 {
        revert(ApiError::from(Error::InvalidTimestamp));
    }
    //define storages we need to work with
    let (
        owners_dict,
        approved_dict,
        publishers_approved_dict,
        producers_approved_dict,
        holders_dict,
        metadata_dict,
    ) = get_buy_storage();

    let purse_balance = get_purse_balance(purse).unwrap_or_revert();
    let caller_string = get_caller().as_string();
    let mut approved_holder = get_approved_holder_by_id(approved_dict, approved_id);
    let token_id = approved_holder.token_id;
    let token_metadata = get_nft_metadata(token_id.to_string(), metadata_dict);

    let producer_hash: AccountHash = approved_holder.owneraccount;
    let publisher_hash: AccountHash = approved_holder.publisheraccount;
    let producer_string: String = producer_hash.as_string();
    let publisher_string: String = publisher_hash.as_string();

    if amount > approved_holder.amount {
        runtime::revert(ApiError::from(Error::NotEnoughAmount));
    }

    //first get the metadata from the token_id(from the metadatas dict)
    let product_price: U512 =
        U512::from_dec_str(token_metadata.price.to_string().as_str()).unwrap_or_default();
    let tax_and_shipping_price = tax_price.add(shipping_price).mul(price_rat * amount); // is amount here needed ?!

    let amount_to_pay = product_price.mul(amount * price_rat);

    let droplinked_share: U512 = amount_to_pay.div(100u64);

    let _amount_remained = amount_to_pay.sub(droplinked_share);

    let publisher_percent: U512 = token_metadata.comission.into();
    let producer_percent: U512 = U512::from(100u64).sub(publisher_percent);
    let producer_part = amount_to_pay
        .mul(producer_percent)
        .div(100u64)
        .add(tax_and_shipping_price);
    let publisher_part = amount_to_pay.sub(producer_part);

    if purse_balance < amount_to_pay {
        //not enough balance
        runtime::revert(ApiError::from(Error::NotEnoughBalance));
    }
    //transfer to producer
    let result_prod = transfer_from_purse_to_account(purse, producer_hash, producer_part, None);
    if result_prod.is_err() {
        //transfer failed
        runtime::revert(ApiError::from(Error::TransferFailed));
    }
    //transfer to publisher
    let result_pub = transfer_from_purse_to_account(purse, publisher_hash, publisher_part, None);
    if result_pub.is_err() {
        //transfer failed
        runtime::revert(ApiError::from(Error::TransferFailed));
    }
    //update approved holder and holder amounts
    approved_holder.amount -= amount;
    //update holder using approved_holder.holder_id
    let holder_opt = storage::dictionary_get::<ndpc_types::NFTHolder>(
        holders_dict,
        approved_holder.holder_id.to_string().as_str(),
    )
    .unwrap_or_revert();
    if holder_opt.is_none() {
        //the holder does not exist
        runtime::revert(ApiError::from(Error::HolderDoesentExist));
    }
    let mut holder: ndpc_types::NFTHolder = holder_opt.unwrap_or_revert();
    holder.amount -= amount;

    storage::dictionary_put(
        holders_dict,
        approved_holder.holder_id.to_string().as_str(),
        holder,
    );
    //if approved holder amount is 0, remove it from publisher and producer's approved lists
    if approved_holder.amount == 0 {
        //remove from publisher's approved list
        let mut publisher_approved_list =
            storage::dictionary_get::<U64list>(publishers_approved_dict, publisher_string.as_str())
                .unwrap_or_revert()
                .unwrap_or_revert_with(ApiError::from(Error::ApprovedListDoesentExist));
        publisher_approved_list.remove(approved_id);
        storage::dictionary_put(
            publishers_approved_dict,
            publisher_string.as_str(),
            publisher_approved_list,
        );
        //remove from producer's approved list

        let mut producer_approved_list =
            storage::dictionary_get::<U64list>(producers_approved_dict, producer_string.as_str())
                .unwrap_or_revert()
                .unwrap_or_revert_with(ApiError::from(Error::ApprovedListDoesentExist));
        producer_approved_list.remove(approved_id);
        storage::dictionary_put(
            producers_approved_dict,
            producer_string.as_str(),
            producer_approved_list,
        );
    }
    let token_id = approved_holder.token_id;
    //update approved holder
    storage::dictionary_put(
        approved_dict,
        approved_id.to_string().as_str(),
        approved_holder,
    );
    //creates new nftholder and adds it to the holders dict and gets holder_id from it and adds it to callers list(if list didn't exist, create it)
    let new_holder = ndpc_types::NFTHolder::new(amount, token_id);
    //get new holder id
    let holders_cnt_uref = ndpc_utils::get_named_key_by_name(NAMED_KEY_HOLDERSCNT);
    let mut holders_cnt: u64 = storage::read(holders_cnt_uref)
        .unwrap_or_revert()
        .unwrap_or_revert();
    holders_cnt += 1;
    storage::write(holders_cnt_uref, holders_cnt);
    let new_holder_id = holders_cnt;
    //add new holder to holders dict
    storage::dictionary_put(holders_dict, new_holder_id.to_string().as_str(), new_holder);
    //add new holder id to callers list
    let caller_list_opt =
        storage::dictionary_get::<U64list>(holders_dict, &caller_string).unwrap_or_revert();
    if caller_list_opt.is_none() {
        //the caller list does not exist
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(holders_dict, &caller_string, new_list);
    } else {
        let mut caller_list = caller_list_opt.unwrap_or_revert();
        caller_list.add(new_holder_id);
        storage::dictionary_put(holders_dict, &caller_string, caller_list);
    }
    //get caller's tokens from owners_dict
    let caller_tokens_opt =
        storage::dictionary_get::<U64list>(owners_dict, &caller_string).unwrap_or_revert();
    if caller_tokens_opt.is_none() {
        //the caller tokens list does not exist
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(owners_dict, &caller_string, new_list);
    } else {
        let mut caller_tokens = caller_tokens_opt.unwrap_or_revert();
        //add holder_id to caller's tokens
        caller_tokens.add(new_holder_id);
        //update caller's tokens
        storage::dictionary_put(owners_dict, &caller_string, caller_tokens);
    }
    emit(DropLinkedEvent::Buy {
        amount,
        approved_id,
        buyer: get_caller(),
    });
}

#[no_mangle]
pub extern "C" fn direct_pay() {
    let product_price: U512 = get_named_arg(RUNTIME_PRODUCT_PRICE);
    let product_shipping: U512 = get_named_arg(RUNTIME_ARG_SHIPPING_PRICE);
    let product_tax: U512 = get_named_arg(RUNTIME_ARG_TAX_PRICE);
    let recipient_key_hex: String = get_named_arg(RUNTIME_ARG_RECIPIENT);

    let recipient: PublicKey = PublicKey::from_hex(recipient_key_hex.clone()).unwrap();
    let purse = {
        let purse_key: Key = runtime::get_named_arg(RUNTIME_ARG_PURSE_ADDR);
        purse_key
            .into_uref()
            .unwrap_or_revert_with(Error::PuseIsNotValid)
    };
    let fee = ndpc_utils::get_fee();
    let purse_balance = get_purse_balance(purse).unwrap_or_revert_with(Error::GetBalance);
    if purse_balance < product_price.add(product_shipping).add(product_tax) {
        runtime::revert(Error::InsufficientFunds);
    }
    let payment_details = calculate_payment(product_price, product_shipping, product_tax, fee);
    let droplinked_share = payment_details.droplinked;
    let recipient_part = purse_balance.sub(droplinked_share);
    transfer_from_purse_to_public_key(purse, get_droplinked_account(), droplinked_share, None)
        .unwrap_or_revert_with(Error::TransferFailed);
    transfer_from_purse_to_public_key(purse, recipient, recipient_part, None)
        .unwrap_or_revert_with(Error::TransferFailed);

    emit(DropLinkedEvent::Payment {
        recipient: recipient_key_hex,
        amounts: vec![product_price, product_shipping, product_tax],
    });
}
