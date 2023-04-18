use casper_types::{account::AccountHash};
pub enum DropLinkedEvent{
    Mint {
        recipient : AccountHash,
        token_id : u64,
        holder_id : u64,
        amount : u64
    },
    PublishOffer{
        holder_id : u64,
        amount : u64,
        commision : u8,
        producer : AccountHash,
        offer_id : u64
    },
    PublishRequest{
        owner : AccountHash,
        publisher : AccountHash,
        amount : u64,
        holder_id : u64,
        request_id : u64,
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