use alloc::string::{String, ToString};
use casper_contract::{contract_api::{runtime::{self, get_named_arg, revert}, storage}, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{U256, Key, ApiError, CLValue, account::AccountHash, URef};
use crate::{
    constants::{self, RUNTIME_ARG_AMOUNT, RUNTIME_ARG_METADATA, RUNTIME_ARG_RECIPIENT, 
        NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME, NAMED_KEY_DICT_METADATAS_NAME, NAMED_KEY_DICT_HOLDERS_NAME, 
        NAMED_KEY_HOLDERSCNT, NAMED_KEY_DICT_OWNERS_NAME, NAMED_KEY_TOKENSCNT}, 
        Error, ndpc_types::{AsStrized, self, NFTHolder, NftMetadata}, 
        ndpc_utils::{self, get_named_key_by_name, get_holders_cnt, get_holder_ids}, 
        event::{DropLinkedEvent, emit}};

fn get_mint_runtime_args() -> (String, U256, u64, String,AccountHash) {
    let reciver_acc = get_named_arg::<Key>(RUNTIME_ARG_RECIPIENT).into_account().unwrap_or_revert_with(ApiError::from(Error::NotAccountHash));
    let reciver : String = reciver_acc.as_string();
    (get_named_arg(RUNTIME_ARG_METADATA),
    get_named_arg("price"),
    get_named_arg(RUNTIME_ARG_AMOUNT),
    reciver,reciver_acc)
}
fn get_mint_dicts() -> (URef, URef, URef, URef, URef, URef) {
    (get_named_key_by_name(NAMED_KEY_DICT_TOKEN_ID_BY_HASH_NAME),
    get_named_key_by_name(NAMED_KEY_DICT_METADATAS_NAME),
    get_named_key_by_name(NAMED_KEY_DICT_HOLDERS_NAME),
    get_named_key_by_name(NAMED_KEY_HOLDERSCNT),
    get_named_key_by_name(NAMED_KEY_DICT_OWNERS_NAME),
    get_named_key_by_name(NAMED_KEY_TOKENSCNT))
}

fn generate_metata(metadata : String, price : U256) -> (NftMetadata, String) {
    let generated_metadata_res = NftMetadata::from_json(metadata,price);
    let generated_metadata = generated_metadata_res.unwrap_or_revert_with(Error::MintMetadataNotValid);
    let metadata_hash = generated_metadata.get_hash().as_string();
    (generated_metadata,metadata_hash)
}

fn get_new_token_id(tokenid_by_hash_uref : URef, metadata_hash : String, tokens_cnt_uref : URef) -> u64 {
    let mut _token_id_final : u64 = 0u64;
    let _token_id : u64 = 0u64;
    match storage::dictionary_get(tokenid_by_hash_uref, &metadata_hash).unwrap_or_revert(){
        Some(_token_id) => {
            _token_id_final = _token_id;
        },
        None => {
            let tokens_cnt:u64 = storage::read(tokens_cnt_uref).unwrap_or_revert().unwrap_or_revert();
            _token_id_final = tokens_cnt + 1u64;
            storage::write(tokens_cnt_uref,_token_id_final);
            storage::dictionary_put(tokenid_by_hash_uref, &metadata_hash, _token_id_final);
        }
    }
    _token_id_final
}

fn add_nft_holder(holders_cnt_uref : URef, owners_dict_uref : URef, holder_by_id_uref : URef , reciver : String, _token_id : u64, amount : u64)-> u64 {
    let nft_holder = NFTHolder::new(amount, amount, _token_id);
    let holders_cnt : u64 = get_holders_cnt(holders_cnt_uref);
    let mut holder_id_final : u64 = 0;
    let owner_holder_ids = get_holder_ids(owners_dict_uref, &reciver);
    if owner_holder_ids.is_none(){
        let mut new_list = ndpc_types::U64list::new();
        new_list.list.push(holders_cnt+ 1u64);
        let holderid : u64 = holders_cnt+ 1u64;
        holder_id_final = holderid;
        storage::write(holders_cnt_uref, holderid);
        storage::dictionary_put(holder_by_id_uref, holderid.to_string().as_str(), nft_holder);    
        storage::dictionary_put(owners_dict_uref, reciver.as_str(), new_list);
    }
    else{
        let mut owner_holder_ids : ndpc_types::U64list = owner_holder_ids.unwrap_or_revert();
        let mut existed = false;
        for holder_id in owner_holder_ids.list.iter(){
            let holder = storage::dictionary_get(holder_by_id_uref, holder_id.to_string().as_str()).unwrap_or_revert();
            if holder.is_none(){
                revert(ApiError::from(Error::MintHolderNotFound));
            }
            let mut holder : NFTHolder = holder.unwrap_or_revert();
            if holder.token_id == _token_id{
                holder.amount += amount;
                holder.remaining_amount += amount;
                storage::dictionary_put(holder_by_id_uref, holder_id.to_string().as_str(), holder);
                existed = true;
                break;
            }
        }
        if !existed {
            let holderid : u64 = holders_cnt+ 1u64;
            holder_id_final = holderid;
            storage::write(holders_cnt_uref, holderid);
            storage::dictionary_put(holder_by_id_uref, holderid.to_string().as_str(), nft_holder);    
            owner_holder_ids.list.push(holderid);
            storage::dictionary_put(owners_dict_uref, reciver.as_str(), owner_holder_ids);
        }
    }
    holder_id_final
}

#[no_mangle]
pub extern "C" fn mint(){
    // get the runtime args
    let (metadata,price,amount,reciver,reciver_acc) = get_mint_runtime_args();

    //generate the metadata
    let (generated_metadata,metadata_hash) = generate_metata(metadata,price);
    
    //get the needed dictionaries
    let (tokenid_by_hash_uref,
            metadata_by_id_uref,
            holder_by_id_uref,
            holders_cnt_uref,
            owners_dict_uref,
            tokens_cnt_uref) = get_mint_dicts();
    
    //get the token id
    let token_id = get_new_token_id(tokenid_by_hash_uref, metadata_hash, tokens_cnt_uref);
    
    //add the token_id generated (or retrieved) to the metadatas dictioanary (with the actual metadata)
    storage::dictionary_put(metadata_by_id_uref, token_id.to_string().as_str(), generated_metadata.to_string());
    
    //Create an NFTHolder object and add it
    let holder_id = add_nft_holder(holders_cnt_uref, owners_dict_uref, holder_by_id_uref, reciver, token_id, amount);

    //update the total supply dict by adding the amount of tokens minted to that token_id
    let total_supply_uref = ndpc_utils::get_named_key_by_name(constants::NAMED_KEY_DICT_TOTAL_SUPPLY);
    let total_supply = storage::dictionary_get(total_supply_uref, token_id.to_string().as_str()).unwrap_or_revert();
    if total_supply.is_none(){
        storage::dictionary_put(total_supply_uref, token_id.to_string().as_str(), amount);
    }
    else{
        let mut total_supply : u64 = total_supply.unwrap_or_revert();
        total_supply += amount;
        storage::dictionary_put(total_supply_uref, token_id.to_string().as_str(), total_supply);
    }

    // return the token_id
    let ret_val = CLValue::from_t(token_id).unwrap_or_revert();
    
    emit(DropLinkedEvent::Mint { recipient: reciver_acc, token_id, holder_id, amount});
    runtime::ret(ret_val);
}