use casper_types::{Key, account::AccountHash};
use crate::ndpc_types;
use ndpc_types::{ApprovedNFT,MetadataHash,NFTHolder,NftMetadata,PublishRequest};

pub enum DropLinkedEvent{
    Mint {
        recipient : AccountHash,
        token_id : u64,
        holder_id : u64,
        amount : u64
    },
    PublishRequest{
        owner : AccountHash,
        publisher : AccountHash,
        amount : u64,
        holder_id : u64,
        request_id : u64
    },
    ApprovedPublish {
        request_id : u64,
        approved_id : u64
    },
    DisapprovedPublish {
        approved_id : u64
    },
    CancelRequest{
        request_id : u64
    },
    Buy{
        amount : u64,
        approved_id : u64,
        buyer : AccountHash
    }    
}