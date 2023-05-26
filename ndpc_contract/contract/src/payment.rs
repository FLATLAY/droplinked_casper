use core::ops::{Add, Sub};

use crate::{
    constants::{
        NAMED_KEY_DICT_APPROVED_NAME, NAMED_KEY_DICT_HOLDERS_NAME, NAMED_KEY_DICT_METADATAS_NAME,
        NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_DICT_PRODAPPROVED_NAME,
        NAMED_KEY_DICT_PUBAPPROVED_NAME, RUNTIME_ARG_AMOUNT,
        RUNTIME_ARG_APPROVED_ID, RUNTIME_ARG_CURRENT_PRICE_TIMESTAMP, RUNTIME_ARG_PURSE_ADDR,
        RUNTIME_ARG_RECIPIENT, RUNTIME_ARG_SHIPPING_PRICE, RUNTIME_ARG_SIGNATURE,
        RUNTIME_ARG_TAX_PRICE, RUNTIME_PRODUCT_PRICE, NAMED_KEY_HOLDERSCNT,
    },
    event::{emit, DropLinkedEvent},
    ndpc_types::{self, AsStrized, U64list},
    ndpc_utils::{
        self, calculate_payment, get_approved_holder_by_id, get_droplinked_account,
        get_nft_metadata, get_ratio_verifier, verify_signature, get_fee,
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

/// Buy Entrypoint's needed dicts
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

/// Buy entrypoint's runtime args
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

/// Buy entrypoint of the droplinked contract
/// 
/// Gets the ratio verifier, gets the incoming purse, splits its tokens to the producer, publisher and droplinked based on the fee and comission and shipping and tax, and ratio of casper/usd
/// Verifies the signature of the droplinked account on the ratio, and checks the time provided to it (to prevent time based ratio attacks)
/// Transfers the calculated amounts to corresponding accounts, and transfers the NFT
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
        _owners_dict,
        approved_dict,
        _publishers_approved_dict,
        _producers_approved_dict,
        _holders_dict,
        metadata_dict,
    ) = get_buy_storage();
    
    let caller_string = get_caller().as_string();
    
    
    
    let mut _approved_holder: ndpc_types::ApprovedNFT = get_approved_holder_by_id(approved_dict, approved_id); //1)
    let token_id = _approved_holder.token_id;

    
    
    let token_metadata = get_nft_metadata(token_id.to_string(), metadata_dict); //2)
    
    let producer_hash: AccountHash = _approved_holder.owneraccount;
    let publisher_hash: AccountHash = _approved_holder.publisheraccount;
    let producer_string: String = producer_hash.as_string();
    let publisher_string: String = publisher_hash.as_string();

    
    if amount > _approved_holder.amount {
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

    let mut approved_holder = get_approved_holder_by_id(approved_dict, approved_id);

    approved_holder.amount -= amount;

    let mut holder: ndpc_types::NFTHolder = storage::dictionary_get::<ndpc_types::NFTHolder>(
        _holders_dict,
        approved_holder.holder_id.to_string().as_str(),
    )
    .unwrap_or_revert_with(Error::HolderDoesentExist)
    .unwrap_or_revert_with(Error::HolderDoesentExist);
    holder.amount -= amount;
    storage::dictionary_put(
        _holders_dict,
        approved_holder.holder_id.to_string().as_str(),
        holder,
    );

    if approved_holder.amount == 0 {
        let mut publisher_approved_list =
            storage::dictionary_get::<U64list>(_publishers_approved_dict, publisher_string.as_str())
                .unwrap_or_revert()
                .unwrap_or_revert_with(ApiError::from(Error::_ApprovedListDoesentExist));
        publisher_approved_list.remove(approved_id);
        storage::dictionary_put(
            _publishers_approved_dict,
            publisher_string.as_str(),
            publisher_approved_list,
        );
        let mut producer_approved_list =
            storage::dictionary_get::<U64list>(_producers_approved_dict, producer_string.as_str())
                .unwrap_or_revert()
                .unwrap_or_revert_with(ApiError::from(Error::_ApprovedListDoesentExist));
        producer_approved_list.remove(approved_id);
        storage::dictionary_put(
            _producers_approved_dict,
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
    storage::dictionary_put(_holders_dict, new_holder_id.to_string().as_str(), new_holder);
    let caller_list_opt =
        storage::dictionary_get::<U64list>(_holders_dict, &caller_string).unwrap_or_revert();
    if caller_list_opt.is_none() {
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(_holders_dict, &caller_string, new_list);
    } else {
        let mut caller_list = caller_list_opt.unwrap_or_revert();
        caller_list.add(new_holder_id);
        storage::dictionary_put(_holders_dict, &caller_string, caller_list);
    }
    let caller_tokens_opt =
        storage::dictionary_get::<U64list>(_owners_dict, &caller_string).unwrap_or_revert();
    if caller_tokens_opt.is_none() {
        let mut new_list = U64list::new();
        new_list.add(new_holder_id);
        storage::dictionary_put(_owners_dict, &caller_string, new_list);
    } else {
        let mut caller_tokens = caller_tokens_opt.unwrap_or_revert();
        caller_tokens.add(new_holder_id);
        storage::dictionary_put(_owners_dict, &caller_string, caller_tokens);
    }
    
}

/// Direct buy is used to proxy the casper transfers through droplinked's contract, to transfer droplinked's share to its account, and transfer the rest of it to the producer
/// 
/// fee% of the product price should go to droplinked's account, and the rest of it (tax price + shipping price + rest of the product price) to the producer's account
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
