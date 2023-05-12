use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use casper_contract::contract_api::storage;
use casper_types::{account::AccountHash, URef, U512};

use crate::{constants, ndpc_utils::contract_package_hash};
pub enum DropLinkedEvent {
    Mint {
        recipient: AccountHash,
        token_id: u64,
        holder_id: u64,
        amount: u64,
        comission: u64,
        price: u64,
    },
    PublishRequest {
        owner: AccountHash,
        publisher: AccountHash,
        amount: u64,
        holder_id: u64,
        request_id: u64,
    },
    ApprovedPublish {
        request_id: u64,
        approved_id: u64,
    },
    DisapprovedPublish {
        approved_id: u64,
    },
    CancelRequest {
        request_id: u64,
    },
    Buy {
        amount: u64,
        approved_id: u64,
        buyer: AccountHash,
    },
    Payment {
        recipient: String,
        amounts: Vec<U512>,
    },
}

pub fn emit(event: DropLinkedEvent) {
    let mut events = Vec::new();
    let package = contract_package_hash();
    match event {
        DropLinkedEvent::Mint {
            recipient,
            token_id,
            holder_id,
            amount,
            comission,
            price,
        } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_mint".to_string());
            param.insert("recipient", recipient.to_string());
            param.insert("token_id", token_id.to_string());
            param.insert("holder_id", holder_id.to_string());
            param.insert("amount", amount.to_string());
            param.insert("comission", comission.to_string());
            param.insert("price", price.to_string());
            events.push(param);
        }
        DropLinkedEvent::PublishRequest {
            owner,
            publisher,
            amount,
            holder_id,
            request_id,
        } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_publish_request".to_string());
            param.insert("owner", owner.to_string());
            param.insert("publisher", publisher.to_string());
            param.insert("amount", amount.to_string());
            param.insert("holder_id", holder_id.to_string());
            param.insert("request_id", request_id.to_string());
            events.push(param);
        }
        DropLinkedEvent::DisapprovedPublish { approved_id } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_disapproved_publish".to_string());
            param.insert("approved_id", approved_id.to_string());
            events.push(param);
        }
        DropLinkedEvent::CancelRequest { request_id } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_cancel_request".to_string());
            param.insert("request_id", request_id.to_string());
            events.push(param);
        }
        DropLinkedEvent::ApprovedPublish {
            request_id,
            approved_id,
        } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_approved_publish".to_string());
            param.insert("request_id", request_id.to_string());
            param.insert("approved_id", approved_id.to_string());
            events.push(param);
        }
        DropLinkedEvent::Buy {
            amount,
            approved_id,
            buyer,
        } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_buy".to_string());
            param.insert("amount", amount.to_string());
            param.insert("approved_id", approved_id.to_string());
            param.insert("buyer", buyer.to_string());
            events.push(param);
        }
        DropLinkedEvent::Payment { recipient, amounts } => {
            let mut param = alloc::collections::BTreeMap::new();
            param.insert(constants::CONTRACTPACKAGEHASH, package.to_string());
            param.insert("event_type", "droplinked_direct_pay".to_string());
            param.insert(
                "amounts",
                format!("{},{},{}", amounts[0], amounts[1], amounts[2]),
            );
            param.insert("recipient", recipient);
            events.push(param);
        }
    }
    for param in events {
        let _: URef = storage::new_uref(param);
    }
}
