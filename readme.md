# Droplinked contract

## Build and deploy
### Prerequisites
You need to have [`CLI Client`](https://docs.casperlabs.io/developers/prerequisites/#the-casper-command-line-client), `rust` (1.60.0-nightly or higher), and `make` installed in your system. 
### Build
Run :
```
make build-contract
```
in the _ndpc_contract_ folder, in order to build the WASM file, to deploy it into casper chain.

The result of this build would be located in ndpc_contract/deploy/contract.wasm file.

**If you do not have `make` installed, you could run this command directly:**

```
cd contract && cargo build --release --target wasm32-unknown-unknown
wasm-strip contract/target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true
cp contract/target/wasm32-unknown-unknown/release/contract.wasm deploy/contract.wasm
```


### Deploy
In deploy part, we should send this WASM file, to casper nodes, to deploy them on the chain,
Run :
```
casper-client put-deploy -n http://<CSPR_RPC_HERE>:7777 --chain-name <CHAINNAME> --payment-amount 190420060000 -k <PATH_TO_SECRET_KEY> --session-path deploy/contract.wasm --session-arg "timestamp:u64='<CURRENTTIMESTAMP>'" --session-arg "ratio_verifier:string='<PUBLICKEY_OF_RATIO_VERIFIER>'"
```
where:
- `CSPR_RPC` is the ip address of a casper rpc node (for testnet or mainnet) which could be found [here](https://testnet.cspr.live/tools/peers) and [here](https://cspr.live/tools/peers) for testnet or mainnet nodes,
- `CHAINNAME` should be `casper-test` for testnet, and `casper` for mainnet.
- `PATH_TO_SECRET_KEY`, which could be accessed by downloading your private-key from `casper-signer`
- `CURRENTTIMESTAMP` which should be set to the current UNIXEPOCH time, it is used for security issues on contract
- `PUBLICKEY_OF_RATIO_VERIFIER`, should be set to the public key of the party (or person), who signs the CSPR/USDT ratio for `buy` entrypoint

### Unit tests
**The Contract was developed and tested on casper-testnet, so this repository does not include unit tests for this contract**

---

## Introduction
On the droplinked protocol, we are registering products on chain and to enable 3rd party publishers to leverage these registered products and sell them across any marketplace, dapp or native site in order to earn commission. We are complimenting this with headless tooling for NFT Gated store fronts on droplinked.com and other valued added NFT solutions. This particular repository contains the customized contract for the Casper Network.

**droplinked.com needs to interact with the casper-signer in order to sign transactions. Interaction with the casper-signer is nessesary for login, minting, buying, publishing and all other actions that require a signature.**

droplinkeds' contract implements base functionalities of ethereum's ERC-1155 standard. This contract implements SFT tokens (Semi-Fungible Token), which have both uniqueness and value. For example, a producer wants to mint 1M NFTs of the same product (each product has an nft which describes who owns the item); by minting 1M NFT's in a standard such as an ERC-721 (CEP47) is not cost effective (storing 1M ID's and owner address will cost a lot of gas); so instead of minting them one by one, we mint a base token (which contains the ID), and hold that id alongside the number of tokens that a particular account owns. 

This way, we only store a single token ID (which represents the product), and a single number (which represents how many of these token ID's a person owns) for each particular account. 

On droplinked, a publisher can send a publish request to the producer with a particular pre-defined commission amount. The producer can accept or reject requests and if a request is accepted, the publisher is then given the abilkity to publish the product to share with consumers and earn their entitled settlement portion.

There exists a [python util file](https://github.com/FLATLAY/droplinked_casper/blob/b089e7c3bc9c04304fa5eb5984902297ec939c85/ndpc_contract/util.py) which interacts with the contract (for testing purposes). It uses 3 accounts (their keys are located in the [Keys Directory](https://github.com/FLATLAY/droplinked_casper/tree/b089e7c3bc9c04304fa5eb5984902297ec939c85/ndpc_contract/keys)).

---

## Structure of the contract
Here we explain each structure used within the contract and how they are used:

1. [NFTHolder](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L33-L37) : this struct holds the token ID and its amount for a specific account. remaining_amount is the amount left which is not published for a publisher.
2. [NFTMetadata](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L26-L31) : this struct holds the metadata of a token. It has a name, a URI(it can be IPFS hash), and a checksum (the hash of the file uploaded off-chain), and a price (in USD). We will add functionality to buy method to buy a token with a constant USD price that levarage CSPR token in future.
3. [PublishRequest](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L19-L25) : this struct holds the request of a publisher to a producer to publish a token. It has a holder_id, amount, a publisher address, a producer address, and commission. this struct will be saved in a dictionary which maps a request_id to a PublishRequest.
4. [ApprovedNFT](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/ndpc_types.rs#L39-L46) : this struct holds the data of the approved tokens (for publishers), it has a holder_id, amount, owner and publisher account address, the token_id, and the amount of commission. After approving a PublishRequest by a producer, it will be saved in a dictionary which maps every approved_id to this object.

---

## Methods (EntryPoints)
Here we explain each method within the contract and how they are used:

1. [**Mint**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L46-L110) : gets (`metadata` , `price` , `amount` , `reciver_key`) and mints the `amount` of tokens to `reciver_key`'s account. It first stores the metadata in a `NFTMetadata` struct and saves it in `metadas` dict (which maps a token_id to its `NFTMetadata`). if the `metadata` is already minted, it will use its existing `token_id`. then it will create a `NFTHolder` struct and save it in `holders` dict (which maps a holder_id to a list of `NFTHolder` structs). If the `reciver_key` already owns this token, it will add the `amount` to its `NFTHolder` struct, otherwise it will create a new `NFTHolder` struct and add it to the list.
2. [**publish_request**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L470-L534) : gets (`producer_account_hash`, `holder_id`, `amount`, `comission`) and creates a `PublishRequest` struct and saves it in `publish_requests` dict (which maps a request_id to a `PublishRequest` struct). Then puts the `request_id` in `producer_requests` dict (which maps a producer account hash to a list of request_ids), also puts the `request_id` in `publisher_requests` dict (which maps a publisher account hash to a list of request_ids). A producer can accept or reject a request and a publisher can cancel any request.
3. [**approve**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L113-L219) : gets (`request_id`) and approves it, and creates an `ApprovedNFT` struct and saves it in `approved_nfts` dict (which maps a approved_id to an `ApprovedNFT` struct). then puts the `approved_id` in `producer_approved` dict (which maps a producer account hash to a list of approved_ids), also puts the `approved_id` in `publisher_approved` dict (which maps a publisher account hash to a list of approved_ids). A producer can disapprove an approved request at any time post an timestamp.
4. [**disapprove**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L222-L284) : gets (`approved_id`, `amount`, `publisher_address`) and disapproves the `approved_id`. If the `amount` is equal to the `amount` of the `ApprovedNFT` struct, it will remove the `approved_id` from `producer_approved` and `publisher_approved` dicts. Otherwise, it will decrease the `amount` of the `ApprovedNFT` struct.
5. [**buy**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L286-L415) : gets (`approved_id` and `amount`) and a `purse` (which the session code will pass to the contract) and if the CSPR tokens in the `purse` are sufficient based on the requirement, it will transfer the commission amount to the publisher and the remainder to the producer minus any royalties. Then it will decrease the `amount` of the `ApprovedNFT` struct. if the `amount` is equal to the `amount` of the `ApprovedNFT` struct, it will remove the `approved_id` activate `producer_approved` and `publisher_approved` dicts. Then it creates a `NFTHolder` struct for the buyer and saves it in `holders` dict. [TODO] : If the buyer already owns this token, it will add the `amount` to its `NFTHolder` struct, otherwise it will create a new `NFTHolder` struct and add it to the list.
6. [**cancel_request**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L536-L565) : gets (`request_id`) and removes the `request_id` from `producer_requests` and `publisher_requests` dicts.
7. [**init**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/main.rs#L567-L580) : Initializes the dictionaries needed for the contract. It is called only once when the contract is deployed by the installer and it's not callable by any other user (or by the installer after the installation of the contract).
8. [**Getter Functions**](https://github.com/FLATLAY/droplinked_casper/blob/8378af28ebeda4559ae76044d41ff9cdcc770227/ndpc_contract/contract/src/constants.rs#L39-L40) : These functions are used for interacting with the contract from one contract to another or by session call.

## Groups
- Constructor : this group is used for initializing the contract.
- [TODO] Producer : producers can mint tokens and approve requests.
- [TODO] Publisher : publishers can publish approved requests and send publish requests.

---

## Storage Model

![storageModel](https://user-images.githubusercontent.com/20683538/226114657-869ba832-f870-4571-9692-ff25347f54c3.png)

## Deployment

This contract is deployed on Testnet (casper-test) successfully, here is the contract hash: [7e818936550e216dd01e83fba521c726a573336afd9953148b97aec01e8c3d68](https://testnet.cspr.live/contract/7e818936550e216dd01e83fba521c726a573336afd9953148b97aec01e8c3d68)


# Project Feautures
## NFT Gating system
Producers can set a set of rules in order to sell their tokens. They can limit the buyers to accounts which have bought several other tokens by the producer (gating), or they can provide tiered discounts.

These rules (ruleset) are deployed on droplinked.com before the customer purchases the token.

## NFT Storefront
droplinked.com provides a storefront in wich the producers can upload their NFTs and set their prices and rulesets, while customers can explore the NFTs and buy them. These NFT's represent both digital and physical goods.
