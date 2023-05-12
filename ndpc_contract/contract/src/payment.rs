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
    let signed_data = mp.split(',').collect::<Vec<&str>>();
    let price_ratio: u64 = signed_data[0].parse::<u64>().unwrap();
    let provided_timestamp = signed_data[1].parse::<u64>().unwrap();

    let latest_block_time: u64 = get_blocktime().into();
    // We will let only 2 blocks time for the transaction to happen !
    if provided_timestamp <= latest_block_time + 66000u64 {
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

    let product_price: U512 = U512::from_dec_str(token_metadata.price.to_string().as_str())
        .unwrap_or_default()
        .mul(price_ratio)
        .mul(amount)
        .div(100u64);

    let purse_balance = get_purse_balance(purse).unwrap_or_revert();
    let total_amount_to_transfer = shipping_price.add(tax_price).add(product_price);

    if purse_balance < total_amount_to_transfer {
        runtime::revert(ApiError::from(Error::NotEnoughBalance));
    }

    let droplinked_share: U512 = product_price.div(100u64); // 1% of the product price goes to droplinked account as a fee, load the fee amount from named keys instead of hardcoding it
    let producer_share: U512 = total_amount_to_transfer
        .sub(droplinked_share)
        .mul(U512::from(100u64).sub(token_metadata.comission))
        .div(100u64);
    let publisher_share: U512 = total_amount_to_transfer
        .sub(droplinked_share)
        .sub(producer_share);

    //transfer to producer
    transfer_from_purse_to_account(purse, producer_hash, producer_share, None)
        .unwrap_or_revert_with(Error::TransferFailed);
    //transfer to publisher
    transfer_from_purse_to_account(purse, publisher_hash, publisher_share, None)
        .unwrap_or_revert_with(Error::TransferFailed);
    //transfer to droplinked
    transfer_from_purse_to_public_key(purse, get_ratio_verifier(), droplinked_share, None)
        .unwrap_or_revert_with(Error::TransferFailed);

    approved_holder.amount -= amount;

    let mut holder: ndpc_types::NFTHolder = storage::dictionary_get::<ndpc_types::NFTHolder>(
        holders_dict,
        approved_holder.holder_id.to_string().as_str(),
    )
    .unwrap_or_revert_with(Error::HolderDoesentExist)
    .unwrap_or_revert_with(Error::HolderDoesentExist);
    holder.amount -= amount;
    storage::dictionary_put(
        holders_dict,
        approved_holder.holder_id.to_string().as_str(),
        holder,
    );

    if approved_holder.amount == 0 {
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
    storage::dictionary_put(
        approved_dict,
        approved_id.to_string().as_str(),
        approved_holder,
    );
    let new_holder = ndpc_types::NFTHolder::new(amount, token_id);
    let holders_cnt_uref = ndpc_utils::get_named_key_by_name(NAMED_KEY_HOLDERSCNT);
    let mut holders_cnt: u64 = storage::read(holders_cnt_uref)
        .unwrap_or_revert()
        .unwrap_or_revert();
    holders_cnt += 1;
    storage::write(holders_cnt_uref, holders_cnt);
    let new_holder_id = holders_cnt;
    storage::dictionary_put(holders_dict, new_holder_id.to_string().as_str(), new_holder);
    let caller_list_opt =
        storage::dictionary_get::<U64list>(holders_dict, &caller_string).unwrap_or_revert();
    if caller_list_opt.is_none() {
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(holders_dict, &caller_string, new_list);
    } else {
        let mut caller_list = caller_list_opt.unwrap_or_revert();
        caller_list.add(new_holder_id);
        storage::dictionary_put(holders_dict, &caller_string, caller_list);
    }
    let caller_tokens_opt =
        storage::dictionary_get::<U64list>(owners_dict, &caller_string).unwrap_or_revert();
    if caller_tokens_opt.is_none() {
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(owners_dict, &caller_string, new_list);
    } else {
        let mut caller_tokens = caller_tokens_opt.unwrap_or_revert();
        caller_tokens.add(new_holder_id);
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
