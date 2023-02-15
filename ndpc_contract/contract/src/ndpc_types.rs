//  __________________________________________________________________________________________________
// |    Author: 	k3rn3lpanic
// |    Description: 	Defines the types used in the contract and implements some traits for converting
// |    them into byte arrays, or into JSON strings (and vice versa).
// |__________________________________________________________________________________________________

extern crate alloc;
use alloc::{string::{String, ToString}, vec::Vec, borrow::ToOwned, boxed::Box, format};
use casper_contract::contract_api::{runtime::blake2b, storage};
use casper_types::{account::AccountHash, bytesrepr::{FromBytes, ToBytes, Error}, CLTyped, U256};
const METADATA_HASH_LENGTH: usize = 32;
pub struct MetadataHash(pub [u8; METADATA_HASH_LENGTH]);
impl AsStrized for MetadataHash{
    fn as_string(&self) -> String{
        base16::encode_lower(&self.0)
    }
}
// This struct is used to store publish requests
pub struct PublishRequest{ 
    pub holder_id : u64,
    pub amount : u64,
    pub percentage : u8,
    pub producer : AccountHash,
    pub publisher : AccountHash,
}
pub struct NftMetadata{
    pub name: String,
    pub token_uri: String,
    pub checksum: String,
    pub price : U256
}
// A amount and a token_id identifies a NFT
pub struct NFTHolder {
    pub remaining_amount : u64,
    pub amount : u64,
    pub token_id : u64
}

// this struct is used to store the approved NFTs (approved to publish)
pub struct ApprovedNFT {
    pub holder_id : u64,
    pub amount : u64,
    pub owneraccount : AccountHash,
    pub publisheraccount : AccountHash,
    pub token_id : u64,
    pub percentage : u8
} //size : 32 + 32 + 8 + 8 + 8 + 1 = 89 bytes

// a simple wrapper for a list of u64 (used to store multiple lists of u64 in the contract)
pub struct U64list{
    pub list : Vec<u64>
}

impl ToBytes for NftMetadata{
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = alloc::vec::Vec::new();
        let mut nft_metadata_string = format!("{},{},{},{}", self.name, self.token_uri, self.checksum, self.price);
        result.append(&mut nft_metadata_string.to_bytes()?);
        Ok(result)
    }
    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
    fn serialized_length(&self) -> usize{
        self.name.serialized_length() + self.token_uri.serialized_length() + self.checksum.serialized_length() + self.price.serialized_length()
    }
}

impl CLTyped for NftMetadata{
    fn cl_type() -> casper_types::CLType{
        casper_types::CLType::Any
    }
}

impl FromBytes for NftMetadata{
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (name, rem) = FromBytes::from_bytes(bytes)?;
        let (token_uri, rem) = FromBytes::from_bytes(rem)?;
        let (checksum, rem) = FromBytes::from_bytes(rem)?;
        let (price, rem) = FromBytes::from_bytes(rem)?;
        Ok((NftMetadata{name, token_uri, checksum , price}, rem))
    }
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}

impl NftMetadata{
    pub fn get_hash(&self) -> MetadataHash{
        return MetadataHash(blake2b((self.name.as_str().to_owned()+self.token_uri.as_str()+self.checksum.as_str()).as_bytes()));
    }
    pub fn new(name : String , token_uri : String , checksum : String, price : U256) -> Self{
        NftMetadata {name , token_uri , checksum, price}
    }
    pub fn to_json(&self) -> String{
        format!("{{\"name\":\"{}\",\"token_uri\":\"{}\",\"checksum\":\"{}\",\"price\":\"{}\"}}",self.name,self.token_uri,self.checksum,self.price)
    }
    pub fn to_string(&self) -> String{
        format!("{},{},{},{}",self.name,self.token_uri,self.checksum,self.price)
    }
    pub fn from_json(json : String , price : U256) -> Result<Self, Error>{
        let split = json.split('\"');
        //TODO: use another functionality to get the name, token_uri and checksum from the json (this one depends on the index of the split)
        let mut name = String::new();
        let mut token_uri = String::new();
        let mut checksum = String::new();
        for (i,s) in split.enumerate(){
            if i == 3{
                name = s.to_owned();
            }
            if i == 7{
                token_uri = s.to_owned();
            }
            if i == 11{
                checksum = s.to_owned();
            }
        }
        Ok(NftMetadata::new(name,token_uri,checksum,price))
    }
}

//Implement ToBytes, FromBytes and CLTyped for NFTHolder to be able to store it in the contract's storage and retrieve it
impl ToBytes for NFTHolder{
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::new();
        result.append(&mut self.remaining_amount.to_bytes()?);
        result.append(&mut self.amount.to_bytes()?);
        result.append(&mut self.token_id.to_bytes()?);
        Ok(result)
    }
    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
    fn serialized_length(&self) -> usize{
        self.remaining_amount.serialized_length() + self.amount.serialized_length() + self.token_id.serialized_length()
    }
}

impl FromBytes for NFTHolder{
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (remaining_amount, rem) = FromBytes::from_bytes(bytes)?;
        let (amount, rem) = FromBytes::from_bytes(rem)?;
        let (token_id, rem) = FromBytes::from_bytes(rem)?;
        Ok((NFTHolder{remaining_amount, amount, token_id}, rem))
    }
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}

impl CLTyped for NFTHolder{
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::ByteArray(6u32)
    }
}

impl NFTHolder{
    pub fn new(remaining_amount : u64 , amount : u64 , token_id : u64) -> Self{
        NFTHolder {remaining_amount, amount,token_id}
    }
    pub fn to_string(&self) -> String{
        format!("{{\"remaining_amount\":\"{}\",\"amount\":\"{}\",\"token_id\":\"{}\"}}",self.remaining_amount,self.amount,self.token_id)
    }
}

impl ToBytes for ApprovedNFT{
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::new();
        result.append(&mut self.holder_id.to_bytes()?);
        result.append(&mut self.amount.to_bytes()?);
        result.append(&mut self.owneraccount.to_bytes()?);
        result.append(&mut self.publisheraccount.to_bytes()?);
        result.append(&mut self.token_id.to_bytes()?);
        result.append(&mut self.percentage.to_bytes()?);
        Ok(result)
    }
    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
    fn serialized_length(&self) -> usize{
        self.holder_id.serialized_length() + self.amount.serialized_length() + self.owneraccount.serialized_length() + self.publisheraccount.serialized_length() + self.token_id.serialized_length() + self.percentage.serialized_length()
    }
}

impl FromBytes for ApprovedNFT{
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (holder_id, rem) = FromBytes::from_bytes(bytes)?;
        let (amount, rem) = FromBytes::from_bytes(rem)?;
        let (owneraccount, rem) = FromBytes::from_bytes(rem)?;
        let (publisheraccount, rem) = FromBytes::from_bytes(rem)?;
        let (token_id, rem) = FromBytes::from_bytes(rem)?;
        let (percentage, rem) = FromBytes::from_bytes(rem)?;
        Ok((ApprovedNFT{holder_id, amount, owneraccount, publisheraccount, token_id, percentage}, rem))
    }
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}
impl CLTyped for ApprovedNFT{
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::ByteArray(89u32)
    }
}

impl ApprovedNFT{
    pub fn new(holder_id : u64 , amount : u64 , owneraccount : AccountHash , publisheraccount : AccountHash, token_id : u64, percentage : u8) -> Self{
        ApprovedNFT {holder_id, amount, owneraccount, publisheraccount, token_id , percentage}
    }
    pub fn to_string(&self) -> String{
        format!("{{\"holder_id\":\"{}\",\"amount\":\"{}\",\"owneraccount\":\"{}\",\"publisheraccount\":\"{}\",\"token_id\":\"{}\",\"percentage\":\"{}\"}}",self.holder_id,self.amount,self.owneraccount,self.publisheraccount,self.token_id,self.percentage)
    }
}
impl ToBytes for U64list{
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::new();
        result.append(&mut self.list.to_bytes()?);
        Ok(result)
    }
    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
    fn serialized_length(&self) -> usize{
        self.list.serialized_length()
    }
}
impl FromBytes for U64list{
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (list, rem) = FromBytes::from_bytes(bytes)?;
        Ok((U64list{list}, rem))
    }
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}
impl CLTyped for U64list{
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::List(Box::new(casper_types::CLType::U64))
    }
}
impl U64list{
    pub fn new() -> Self{
        U64list { list : Vec::new() } 
    }
    pub fn remove(&mut self, value : u64) -> u64{
        self.list.retain(|&x| x != value);
        value
    }
    pub fn add(&mut self, value : u64){
        self.list.push(value);
    }
}

impl ToBytes for PublishRequest{
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::new();
        result.append(&mut self.holder_id.to_bytes()?);
        result.append(&mut self.amount.to_bytes()?);
        result.append(&mut self.percentage.to_bytes()?);
        result.append(&mut self.producer.to_bytes()?);
        result.append(&mut self.publisher.to_bytes()?);
        Ok(result)
    }
    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
    fn serialized_length(&self) -> usize{
        self.holder_id.serialized_length() + self.amount.serialized_length() + self.percentage.serialized_length() + self.producer.serialized_length() + self.publisher.serialized_length()
    }
}
impl FromBytes for PublishRequest{
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (holder_id, rem) = FromBytes::from_bytes(bytes)?;
        let (amount, rem) = FromBytes::from_bytes(rem)?;
        let (percentage, rem) = FromBytes::from_bytes(rem)?;
        let (producer, rem) = FromBytes::from_bytes(rem)?;
        let (publisher, rem) = FromBytes::from_bytes(rem)?;
        Ok((PublishRequest{holder_id, amount, percentage, producer, publisher}, rem))
    }
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}
impl CLTyped for PublishRequest{
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::ByteArray(41u32)
    }
}

impl PublishRequest{
    pub fn new(holder_id : u64, amount : u64, percentage : u8, producer : AccountHash, publisher : AccountHash) -> Self{
        PublishRequest {holder_id, amount, percentage, producer, publisher}
    }
    pub fn to_string(&self) -> String{
        format!("{},{},{},{},{}",self.holder_id,self.amount,self.percentage,self.producer,self.publisher)
    }
    pub fn from_string(string : String) -> Self{
        let mut split = string.split(",");
        let holder_id = split.next().unwrap().parse::<u64>().unwrap();
        let amount = split.next().unwrap().parse::<u64>().unwrap();
        let percentage = split.next().unwrap().parse::<u8>().unwrap();
        let producer = AccountHash::from_string(split.next().unwrap().to_string());
        let publisher = AccountHash::from_string(split.next().unwrap().to_string());
        PublishRequest {holder_id, amount, percentage, producer, publisher}
    }
}

pub trait from_stringize{
    fn from_string(string : String) -> Self;
}
impl from_stringize for AccountHash{
    fn from_string(string : String) -> Self{
        AccountHash::from_formatted_str(format!("account-hash-{}",string).as_str()).unwrap()
    }
}
pub trait AsStrized{
    fn as_string(&self) -> String;
}
impl AsStrized for AccountHash{
    fn as_string(&self) -> String{
        base16::encode_lower(&self.0)
    }
}