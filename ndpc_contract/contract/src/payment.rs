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
        get_holder_by_id, get_nft_metadata, get_ratio_verifier, verify_signature, get_fee,
    },
    Error,
};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec, format,
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
use casper_types::{account::AccountHash, ApiError, AsymmetricType, Key, PublicKey, U512};

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
) {
    (
        runtime::get_named_arg::<String>(RUNTIME_ARG_CURRENT_PRICE_TIMESTAMP),
        runtime::get_named_arg::<String>(RUNTIME_ARG_SIGNATURE),
        runtime::get_named_arg::<u64>(RUNTIME_ARG_APPROVED_ID),
        runtime::get_named_arg::<u64>(RUNTIME_ARG_AMOUNT),
        runtime::get_named_arg::<U512>(RUNTIME_ARG_SHIPPING_PRICE),
        runtime::get_named_arg::<U512>(RUNTIME_ARG_TAX_PRICE)
    )
}

#[no_mangle]
pub extern "C" fn buy() {
    let ratio_verifier = get_ratio_verifier();
    
    let (mp, sig, approved_id, amount, shipping_price, tax_price) = get_buy_runtime_args();
    
    let purse = {
        let purse_key: Key = runtime::get_named_arg(RUNTIME_ARG_PURSE_ADDR);
        purse_key.into_uref().unwrap_or_revert()
    };


    if !verify_signature(ratio_verifier, sig, mp.clone()) {
        revert(ApiError::from(Error::InvalidSignature));
    }

    let signed_data = mp.split(',').collect::<Vec<&str>>();
    
    let price_ratio: u64 = signed_data[0].parse::<u64>().unwrap();
    let provided_timestamp = signed_data[1].parse::<u64>().unwrap();
    
    let latest_block_time: u64 = u64::from(get_blocktime()); //1)

    if latest_block_time > provided_timestamp + 130000u64 {
        revert(ApiError::from(Error::InvalidTimestamp));
    }

    let (
        owners_dict,
        approved_dict,
        publishers_approved_dict,
        producers_approved_dict,
        holders_dict,
        metadata_dict,
    ) = get_buy_storage();
    
    let caller_string = get_caller().as_string();
    
    
    
    let mut approved_holder: ndpc_types::ApprovedNFT = get_approved_holder_by_id(approved_dict, approved_id); //1)
    let token_id = approved_holder.token_id;

    
    
    let token_metadata = get_nft_metadata(token_id.to_string(), metadata_dict); //2)
    
    let producer_hash: AccountHash = approved_holder.owneraccount;
    let publisher_hash: AccountHash = approved_holder.publisheraccount;
    let producer_string: String = producer_hash.as_string();
    let publisher_string: String = publisher_hash.as_string();

    
    if amount > approved_holder.amount {
        runtime::revert(ApiError::from(Error::NotEnoughAmount));
    }
    
    // Do a function call to transfer function 
    // EIFUH
    
    let fee = get_fee();
    let product_price = (token_metadata.price * price_ratio * amount)/100u64;
    let total_amount = product_price + shipping_price.as_u64()+ tax_price.as_u64();
    let droplinked_share = (product_price * fee) / 10000u64 ;
    let producer_share = ((product_price - droplinked_share) * ((10000u64 - token_metadata.comission)))/10000u64 + shipping_price.as_u64() + tax_price.as_u64();
    let publisher_share = total_amount - producer_share - droplinked_share;

    let log = format!(
        "Buyer: {}, Producer: {}, Publisher: {}, Amount: {}, Price: {}, Shipping: {}, Tax: {}, Droplinked Share: {}, Producer Share: {}, Publisher Share: {}",
        caller_string,
        producer_string,
        publisher_string,
        amount,
        product_price,
        shipping_price,
        tax_price,
        droplinked_share,
        producer_share,
        publisher_share
    );

    runtime::put_key("_log", storage::new_uref(log).into());
    runtime::put_key("_purse", purse.into());

    //transfer to producer
    transfer_from_purse_to_account(purse, producer_hash, U512::from(producer_share), None)
        .unwrap_or_revert_with(Error::TransferFailed);
    //transfer to publisher
    transfer_from_purse_to_account(purse, publisher_hash, U512::from(publisher_share), None)
        .unwrap_or_revert_with(Error::TransferFailed);
    //transfer to droplinked
    transfer_from_purse_to_public_key(purse, get_ratio_verifier(), U512::from(droplinked_share), None)
        .unwrap_or_revert_with(Error::TransferFailed);
    
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
