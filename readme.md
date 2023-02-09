# Droplinked Contract
## Introduction
In Droplinked Project, we are registering the products on chain and enabling publishers to use register the products to sell it in their native site and gain commission, plus we are doing NFT Gated store front (Droplinked.com) and other valued added NFT solutions. this repository contains the Droplinked casper Contract.

**Droplinked.com needs to interact with the casper-signer in order to sign the transactions. Interaction with the casper-signer is nessesary for login, minting, buying, publishing and all other actions that require a signature.**

Droplinked contract implements base functionalities of etherium's erc-1155 standard. This contract implements SFT tokens (Semi Fungible Token), which have both uniquness and value. for example a producer, wants to mint 1M NFTs of a same product (each product has its nft which describes who owns this product); mintin 1M NFT's in standards like ERC-721 (CEP47) is not cost effective (storing 1M ID's and owner address will cost a lot of gas); so instead of minting them one by one, we mint a base token (which has a ID), and hold that id alongside with the number of tokens of this kind that a account owns. 

In this way, we would only store a single token ID (which represents the product), and a single number (which represents how many of this token ID a person owns) for each account. 

In Droplinked, a publisher can send a publish request to the producer with specifi amount of commission. The producer can accept or reject the request. If the request is accepted, the publisher can publish the product to a consumer and earn its part. 

There exists a [python util file](https://github.com/FLATLAY/droplinked_casper/blob/b089e7c3bc9c04304fa5eb5984902297ec939c85/ndpc_contract/util.py) which interacts with the contract (for testing purpose). It uses 3 accounts (their keys are located in [Keys Directory](https://github.com/FLATLAY/droplinked_casper/tree/b089e7c3bc9c04304fa5eb5984902297ec939c85/ndpc_contract/keys)).
  
## Structure of the contract
Here we explain each struct used in the contract, and how they are used.

1. [NFTHolder](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L33-L37) : this struct holds the token ID and its amount for a specific account. remaining_amount is the amount left which is not published for a publisher.
2. [NFTMetadata](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L26-L31) : this struct holds the metadata of a token. It has a name, a URI(it can be IPFS hash), and a checksum (the hash of the file uploaded off-chain), and a price (is USD). we will add functionality to buy method to buy a token with a constant USD price, with CSPR tokens in future.
3. [PublishRequest](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L19-L25) : this struct holds the request of a publisher to a producer to publish a token. It has a holder_id, amount, a publisher address, a producer address, and commission. this struct will be saved in a dictionary which maps a request_id to a PublishRequest.
4. [ApprovedNFT](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L39-L46) : this struct holds the data of the approved tokens (for publishers), it has a holder_id, amount, owner and publisher account address, the token_id, and the amount of commission. After approving a PublishRequest by a producer, it will be saved in a dictionary which maps a approved_id to this object.

## Methods (EntryPoints)
Here we explain each method of the contract, and how they are used.

1. [**Mint**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L46-L110) : gets (`metadata` , `price` , `amount` , `reciver_key`) and mints the `amount` of tokens to `reciver_key`'s account. It first stores the metadata in a `NFTMetadata` struct and saves it in `metadas` dict (which maps a token_id to its `NFTMetadata`). if the `metadata` is already minted, it will use its existing `token_id`. then it will create a `NFTHolder` struct and save it in `holders` dict (which maps a holder_id to a list of `NFTHolder` structs). if the `reciver_key` already owns this token, it will add the `amount` to its `NFTHolder` struct, otherwise it will create a new `NFTHolder` struct and add it to the list.
2. [**publish_request**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L470-L534) : gets (`producer_account_hash`, `holder_id`, `amount`, `comission`) and creates a `PublishRequest` struct and saves it in `publish_requests` dict (which maps a request_id to a `PublishRequest` struct). then puts the `request_id` in `producer_requests` dict (which maps a producer account hash to a list of request_ids), also puts the `request_id` in `publisher_requests` dict (which maps a publisher account hash to a list of request_ids). A producer can accept or reject a request, and a publisher can cancel a request.
3. [**approve**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L113-L219) : gets (`request_id`) and approves it, and creates an `ApprovedNFT` struct and saves it in `approved_nfts` dict (which maps a approved_id to an `ApprovedNFT` struct). then puts the `approved_id` in `producer_approved` dict (which maps a producer account hash to a list of approved_ids), also puts the `approved_id` in `publisher_approved` dict (which maps a publisher account hash to a list of approved_ids). A producer disapprove the approved request in any time.
4. [**disapprove**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L222-L284) : gets (`approved_id`, `amount`, `publisher_address`) and disapproves the `approved_id`. if the `amount` is equal to the `amount` of the `ApprovedNFT` struct, it will remove the `approved_id` from `producer_approved` and `publisher_approved` dicts. otherwise, it will decrease the `amount` of the `ApprovedNFT` struct.
5. [**buy**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L286-L415) : gets (`approved_id` and `amount`) and a `purse` (which the session code will pass to the contract) and if the CSPR tokens in the `purse` where enough, it will transfer the commission amount to the publisher, and the rest to the producer. then it will decrease the `amount` of the `ApprovedNFT` struct. if the `amount` is equal to the `amount` of the `ApprovedNFT` struct, it will remove the `approved_id` from `producer_approved` and `publisher_approved` dicts. then it creates a `NFTHolder` struct for the buyer and saves it in `holders` dict. [TODO] : If the buyer already owns this token, it will add the `amount` to its `NFTHolder` struct, otherwise it will create a new `NFTHolder` struct and add it to the list.
6. [**cancel_request**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L536-L565) : gets (`request_id`) and removes the `request_id` from `producer_requests` and `publisher_requests` dicts.
7. [**init**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L567-L580) : Initializes the dictionaries needed for the contract. It is called only once when the contract is deployed by the installer, and it's not callable by any other user (or by the installer after the installation of the contract).
8. [**Getter Functions**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/constants.rs#L39-L40) : these functions are used for interacting with the contract from another contract or a session call.

## Groups
- Constructor : this group is used for initializing the contract.
- [TODO] Producer : producers can mint tokens and approve requests.
- [TODO] Publisher : publishers can publish approved requests and send publish requests.

## Storage Model

![storageModel](https://raw.githubusercontent.com/FLATLAY/droplinked_casper/main/ndpc_contract/Storage.jpg)

## Deployment

This contract is deployed on Testnet (casper-test) successfully, here is the contract hash : [a5f32ce82d104d80662d9a0c1ed9028ff31b041165ea16f263e2fc7c1965d6a8](https://testnet.cspr.live/contract/a5f32ce82d104d80662d9a0c1ed9028ff31b041165ea16f263e2fc7c1965d6a8)


# Project Feautures
## NFT Gating system
Producers can set a set of rules in order to sell their tokens. they can limit the buyers to accounts which have bought several other tokens by the producer (Gating), or they can give several discounts.

These rules (ruleset) are checked in Droplinked.com before the customer purchases the token.

## NFT FrontStore
Droplinked.com provides a frontstore, in wich the producers can upload their NFTs and set their prices and rulesets, and customers can explore the NFTs and buy them.
