

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fmt::Display;
    use std::path::PathBuf;
    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
        DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT, WasmTestBuilder,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
    use casper_types::CLTyped;
    use casper_types::{
        account::AccountHash, runtime_args, Key, Motes, PublicKey, RuntimeArgs, SecretKey, U512, ContractHash, Contract,
    };
    use casper_types::bytesrepr::{ToBytes, FromBytes};
    // Defining Objects needed to be used with testing contract : 
    const METADATA_HASH_LENGTH: usize = 32;

    pub trait AsStrized {
        fn as_string(&self) -> String;
    }
    pub struct MetadataHash(pub [u8; METADATA_HASH_LENGTH]);
    impl AsStrized for MetadataHash {
        fn as_string(&self) -> String {
            base16::encode_lower(&self.0)
        }
    }
    pub struct U64list{
        pub list : BTreeSet<u64>
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
    pub struct ApprovedNFT {
        pub holder_id: u64,
        pub amount: u64,
        pub owneraccount: AccountHash,
        pub publisheraccount: AccountHash,
        pub token_id: u64,
    }
    impl ToBytes for ApprovedNFT {
        fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
            let mut result = Vec::new();
            result.append(&mut self.holder_id.to_bytes()?);
            result.append(&mut self.amount.to_bytes()?);
            result.append(&mut self.owneraccount.to_bytes()?);
            result.append(&mut self.publisheraccount.to_bytes()?);
            result.append(&mut self.token_id.to_bytes()?);
            Ok(result)
        }
        fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
        where
            Self: Sized,
        {
            self.to_bytes()
        }
        fn serialized_length(&self) -> usize {
            self.holder_id.serialized_length()
                + self.amount.serialized_length()
                + self.owneraccount.serialized_length()
                + self.publisheraccount.serialized_length()
                + self.token_id.serialized_length()
        }
    }
    
    impl FromBytes for ApprovedNFT {
        fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
            let (holder_id, rem) = FromBytes::from_bytes(bytes)?;
            let (amount, rem) = FromBytes::from_bytes(rem)?;
            let (owneraccount, rem) = FromBytes::from_bytes(rem)?;
            let (publisheraccount, rem) = FromBytes::from_bytes(rem)?;
            let (token_id, rem) = FromBytes::from_bytes(rem)?;
            Ok((
                ApprovedNFT {
                    holder_id,
                    amount,
                    owneraccount,
                    publisheraccount,
                    token_id,
                },
                rem,
            ))
        }
        fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
            Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
        }
    }
    impl CLTyped for ApprovedNFT {
        fn cl_type() -> casper_types::CLType {
            casper_types::CLType::Any
        }
    }
    impl Display for ApprovedNFT{
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{{\"holder_id\":\"{}\",\"amount\":\"{}\",\"owneraccount\":\"{}\",\"publisheraccount\":\"{}\",\"token_id\":\"{}\"}}",self.holder_id,self.amount,self.owneraccount,self.publisheraccount,self.token_id)
        }
    }   

    pub struct NftMetadata {
    pub name: String,
    pub token_uri: String,
    pub checksum: String,
    pub price: u64,
    pub comission: u64,
    }
    impl ToBytes for NftMetadata {
        fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
            let mut result = Vec::new();
            result.append(&mut self.name.to_bytes()?);
            result.append(&mut self.token_uri.to_bytes()?);
            result.append(&mut self.checksum.to_bytes()?);
            result.append(&mut self.price.to_bytes()?);
            result.append(&mut self.comission.to_bytes()?);
            Ok(result)
        }
        fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
        where
            Self: Sized,
        {
            self.to_bytes()
        }
        fn serialized_length(&self) -> usize {
            self.name.serialized_length()
                + self.token_uri.serialized_length()
                + self.checksum.serialized_length()
                + self.price.serialized_length()
                + self.comission.serialized_length()
        }
    }
    
    impl CLTyped for NftMetadata {
        fn cl_type() -> casper_types::CLType {
            casper_types::CLType::Any
        }
    }
    
    impl FromBytes for NftMetadata {
        fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
            let (name, rem) = FromBytes::from_bytes(bytes)?;
            let (token_uri, rem) = FromBytes::from_bytes(rem)?;
            let (checksum, rem) = FromBytes::from_bytes(rem)?;
            let (price, rem) = FromBytes::from_bytes(rem)?;
            let (comission, rem) = FromBytes::from_bytes(rem)?;
            Ok((
                NftMetadata {
                    name,
                    token_uri,
                    checksum,
                    price,
                    comission,
                },
                rem,
            ))
        }
        fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
            Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
        }
    }
    
    impl Display for NftMetadata {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "{},{},{},{},{}",
                self.name, self.token_uri, self.checksum, self.price, self.comission
            )
        }
    }
    

    // --------------------------------------------------------------------------------------------------------------
    
    fn install_contract() -> (WasmTestBuilder<InMemoryGlobalState> ,ContractHash, Contract){

        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);

        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE*10)),
            None,
        );  
        let account_prod = GenesisAccount::account(
            public_key_producer,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );  
        let account_pub = GenesisAccount::account(
            public_key_publisher,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None
        );
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = runtime_args! {
            "ratio_verifier" => "0144f5adf499591351807bc83490314262bd6846beee80a16269a83c9901ecec8a".to_string(),
            "fee" => 100u64
        };
        let deploy_item = DeployItemBuilder::new()
        .with_empty_payment_bytes(runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT
        })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();
        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        genesis_config.ee_config_mut().push_account(account_pub);
        
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_success();
        println!("Deployed The contract!");
        // ---------------------------------------
        
        // Get the contract hash from NAMED_KEYS of the deployer account
        let contract_hash = builder
            .get_expected_account(account_addr)
            .named_keys()
            .get("droplinked_contract")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
        println!("Contract hash = {}" , contract_hash);
        let contract : Contract = builder.get_contract(contract_hash).unwrap();
        (builder, contract_hash , contract)
    }

    #[test]
    fn publish_request_with_error(){
        // This publish request should result in error, because it's doing a request on a holder_id that does not exist!
        // Create Accounts needed for this test
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
        
        let (mut builder, contract_hash , _contract) = install_contract();
        
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 2;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_failure()
            .commit();
    }

    #[test]
    fn install_contract_test(){
        install_contract();
    }

    #[test]
    fn install_contract_with_error(){
        // should error because the `fee` is not provided to contract in install step!
        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);

        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE*10)),
            None,
        );  
        let account_prod = GenesisAccount::account(
            public_key_producer,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );  
        let account_pub = GenesisAccount::account(
            public_key_publisher,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None
        );
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = runtime_args! {
            "ratio_verifier" => "0144f5adf499591351807bc83490314262bd6846beee80a16269a83c9901ecec8a".to_string()
        };
        let deploy_item = DeployItemBuilder::new()
        .with_empty_payment_bytes(runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT
        })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();
        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        genesis_config.ee_config_mut().push_account(account_pub);
        
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_failure();
    }

    #[test]
    fn mint_product_with_error(){
        // The price is given with u8, which should be u64, so it does not execute successfullu!
        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
        let (mut builder, contract_hash, _contract) = install_contract();
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u8 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_failure() // We expect errors when the mint entrypoint is called!
            .commit();
    }

    const DEPLOYER_ACCOUNT: [u8; 32] = [6u8; 32];
    const PRODUCER_ACCOUNT: [u8; 32] = [7u8; 32];
    const PUBLISHER_ACCOUNT: [u8; 32] = [8u8; 32];
    const _CUSTOMER_ACCOUNT: [u8; 32] = [9u8; 32];
    const CONTRACT_WASM: &str = "contract.wasm";
    
    #[test]
    fn mint_entrypoint(){

        // Create Accounts needed for this test
        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
                
        // Deploying the contract
        let (mut builder, contract_hash , _contract) = install_contract();
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // --------------------------------------
        
        // Query the contract, and verify that mint is working properly
            // query the tokens_cnt
        let tokens_cnt = builder
            .query(None, Key::Hash(contract_hash.value()), &["tokens_cnt".to_string()])
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u64>()
            .expect("should be u64.");
        assert_eq!(tokens_cnt, 1u64);
            // get the contract, to get dict urefs from it
        let contract : Contract = builder.get_contract(contract_hash).unwrap();
        let dict_uref = contract.named_keys().get("metadatas").unwrap().into_uref().unwrap();
            // use that dict uref, to query the metadata minted on chain
        let meta = builder
            .query_dictionary_item(None, dict_uref, "1")
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<NftMetadata>()
            .expect("should be NFTMetadata");

            // check the retreived is the same as the metadata
        assert_eq!(format!("{}" , meta).to_string().as_str(), "Nike Shoes,bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii,oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd,200,1234");
        // --------------------------------------

    }
 
    #[test]
    fn publish_request_entry_point(){
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

        // -------------------------------------
        
        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // --------------------------------------
        
        // Verify the publish request        
        let requests_cnt = builder
            .query(None, Key::Hash(contract_hash.value()), &["request_cnt".to_string()])
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u64>()
            .expect("should be u64.");
        assert_eq!(requests_cnt, 1u64);
        // -----------------------------------------
    }
        
    #[test]
    fn cancel_request_entry_point(){
        
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

        let (mut builder, contract_hash , contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // --------------------------------------
        // Do the cancel_request
        let cancel_request_id : u64 = 1;
        let contract_cancel_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "cancel_request",
            runtime_args! {
                "request_id" => cancel_request_id
            }
        ).build();
        builder
            .exec(contract_cancel_request)
            .expect_success()
            .commit();
        // -----------------------------------------
        let producer_requests_dict_uref = contract.named_keys().get("producer_requests").unwrap().into_uref().unwrap();
        // Now the requests_cnt should be reduced by 1 so it should be 0 again
        let requests_list : U64list = builder
            .query_dictionary_item(None, producer_requests_dict_uref, producer_account_addr.to_string().as_str())
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<U64list>()
            .expect("should be U64list");
        assert_eq!(requests_list.list.len() , 0usize);
    }

    #[test]
    fn cancel_request_entry_point_with_error(){
        // Should not execute because we are cancelling request with request_id = 2 but only 1 request has been made, so it does not exist
        
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // --------------------------------------
        // Do the cancel_request
        let cancel_request_id : u64 = 2;
        let contract_cancel_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "cancel_request",
            runtime_args! {
                "request_id" => cancel_request_id
            }
        ).build();
        builder
            .exec(contract_cancel_request)
            .expect_failure()
            .commit();
        // -----------------------------------------

    }

    #[test]
    fn cancel_request_entry_point_with_auth_error(){
        // Should not execute because we are calling cancel request with an account which has not sended the request!

        println!("Starting...");        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);
        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

        
        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // --------------------------------------
        // Do the cancel_request
        let cancel_request_id : u64 = 1;
        let contract_cancel_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "cancel_request",
            runtime_args! {
                "request_id" => cancel_request_id
            }
        ).build();
        builder
            .exec(contract_cancel_request)
            .expect_failure()
            .commit();

    }

    #[test]
    fn approve_entry_point(){
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
        
        let (mut builder, contract_hash , contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 1u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_success()
            .commit();
        // -----------------------------------
        // Verify the approvement
        let publishers_approved_uref = contract.named_keys().get("publishers_approved").unwrap().into_uref().unwrap();
        let approved_nft_list : U64list = builder
            .query_dictionary_item(None, publishers_approved_uref, publisher_account_addr.to_string().as_str())
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<U64list>()
            .expect("should be U64list");

            // there should exist one entity in there after aprovement
        assert_eq!(approved_nft_list.list.len() , 1);

        let approved_uref = contract.named_keys().get("approved").unwrap().into_uref().unwrap();
        let approved_nft : ApprovedNFT = builder
            .query_dictionary_item(None, approved_uref, "1")
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<ApprovedNFT>()
            .expect("should be U64list");

        // If you want to change the data of keys on the top of the file, you should edit account hashes below on your own
        assert_eq!(format!("{}" , approved_nft).to_string() , "{\"holder_id\":\"1\",\"amount\":\"10\",\"owneraccount\":\"3d5de8c609159a0954e773dd686fb7724428316cb30e00bdc899976127747f55\",\"publisheraccount\":\"105b69f2d74a211a6cb337cba6751a8f15cc7b44b7c65329c29731b67e1ac047\",\"token_id\":\"1\"}");
    }
    #[test]
    fn approve_entry_point_with_error(){
        // error : approve a request that does not exist
        
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
                
        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 2u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_failure()
            .commit();
    }
    
    #[test]
    fn approve_entry_point_with_auth_error(){
        // error : approve a request with an account that is not the producer

        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

        
        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 1u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_failure()
            .commit();
        // -----------------------------------
    }
    
    #[test]
    fn disapprove_entry_point(){
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
         
        let (mut builder, contract_hash , contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 1u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_success()
            .commit();
        // ------------------------------------
        
        // Get the approved object
        let approved_uref = contract.named_keys().get("approved").unwrap().into_uref().unwrap();
        let approved_nft : ApprovedNFT = builder
            .query_dictionary_item(None, approved_uref, "1")
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<ApprovedNFT>()
            .expect("should be U64list");
        // -------------------------------------
        // Call Disapprove on the approved request
        let disapprove_publisher_key : Key = publisher_account_addr.into();
        let contract_disapprove = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "disapprove",
            runtime_args! {
                "amount" => 2u64,
                "approved_id" => 1u64,
                "publisher-account" => disapprove_publisher_key
            }
        ).build();
        builder
            .exec(contract_disapprove)
            .expect_success()
            .commit();
        // ---------------------------------------
        // Get the approved object again, and compare their amount (it should be reduced by 2)
        let new_approved_nft : ApprovedNFT = builder
            .query_dictionary_item(None, approved_uref, "1")
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<ApprovedNFT>()
            .expect("should be U64list");
        
        assert_eq!(approved_nft.amount - new_approved_nft.amount,2u64);
    }
    #[test]
    fn disapprove_entry_point_error_amount(){
        // error : the amount is more than the approved amount

        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

        let (mut builder, contract_hash , _contract) = install_contract();        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 1u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_success()
            .commit();
        // Call Disapprove on the approved request
        let disapprove_publisher_key : Key = publisher_account_addr.into();
        let contract_disapprove = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "disapprove",
            runtime_args! {
                "amount" => 200u64,
                "approved_id" => 1u64,
                "publisher-account" => disapprove_publisher_key
            }
        ).build();
        builder
            .exec(contract_disapprove)
            .expect_failure()
            .commit();
    }

    #[test]
    fn disapprove_entry_point_auth_error(){
        // error : the account is not the producer
        
        // Create Accounts needed for this test        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
        // --------------------------------------
        
        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 1u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_success()
            .commit();
        // Call Disapprove on the approved request
        let disapprove_publisher_key : Key = publisher_account_addr.into();
        let contract_disapprove = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "disapprove",
            runtime_args! {
                "amount" => 2u64,
                "approved_id" => 1u64,
                "publisher-account" => disapprove_publisher_key
            }
        ).build();
        builder
            .exec(contract_disapprove)
            .expect_failure()
            .commit();
    }

    #[test]
    fn disapprove_entry_point_error_approved_id(){
        // error : wrong approved_id
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

        let (mut builder, contract_hash , _contract) = install_contract();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let price : u64 = 200;
        let comission : u64 = 1234;
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "metadata" => mint_metadata,
                "price" => price,
                "comission" => comission
            },
        ).build();
        builder
            .exec(contract_mint_request)
            .expect_success()
            .commit();
        // ------------------------------------------

        // Do the publish request to the producer
        let publish_prod_acc : Key = producer_account_addr.into();
        let publish_amount : u64 = 10;
        let publish_holder_id : u64 = 1;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // ------------------------------------
        // Do the approve on the request_id=1
        let contract_approve = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "approve",
            runtime_args! {
                "request_id" => 1u64
            }
        ).build();
        builder
            .exec(contract_approve)
            .expect_success()
            .commit();
        // -------------------------------------
        // Call Disapprove on the approved request
        let disapprove_publisher_key : Key = publisher_account_addr.into();
        let contract_disapprove = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "disapprove",
            runtime_args! {
                "amount" => 2u64,
                "approved_id" => 2u64,
                "publisher-account" => disapprove_publisher_key
            }
        ).build();
        builder
            .exec(contract_disapprove)
            .expect_failure()
            .commit();
    }

}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
