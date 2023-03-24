

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use std::path::PathBuf;
    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
        DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::CLTyped;
    use casper_types::{
        account::AccountHash, runtime_args, Key, Motes, PublicKey, RuntimeArgs, SecretKey, U512, ContractHash, U256, Contract,
    };
    use casper_types::bytesrepr::{ToBytes, FromBytes};
    // Defining Objects needed to be used with testing contract : 
    pub struct U64list{
        pub list : Vec<u64>
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
        pub holder_id : u64,
        pub amount : u64,
        pub owneraccount : AccountHash,
        pub publisheraccount : AccountHash,
        pub token_id : u64,
        pub percentage : u8
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
    impl Display for ApprovedNFT{
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{{\"holder_id\":\"{}\",\"amount\":\"{}\",\"owneraccount\":\"{}\",\"publisheraccount\":\"{}\",\"token_id\":\"{}\",\"percentage\":\"{}\"}}",self.holder_id,self.amount,self.owneraccount,self.publisheraccount,self.token_id,self.percentage)
        }
    }    
    // --------------------------------------------------------------------------------------------------------------
    
    const DEPLOYER_ACCOUNT: [u8; 32] = [6u8; 32];
    const PRODUCER_ACCOUNT: [u8; 32] = [7u8; 32];
    const PUBLISHER_ACCOUNT: [u8; 32] = [8u8; 32];
    const _CUSTOMER_ACCOUNT: [u8; 32] = [9u8; 32];
    const CONTRACT_WASM: &str = "contract.wasm";
    
    #[test]
    fn mint_entrypoint(){
        /*
         * 1. Deploy the contract
         * 2. Mint a product with producer account
         * 3. Verify the minted product
         */
        println!("Starting...");
        
        // Create Accounts needed for this test
        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);
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
        // -------------------------------------

        // Get genesis config up and running, add accounts to it
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // --------------------------------------
        
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let timestamp : u64 = 1677241273;
        let session_args = runtime_args! {
            "timestamp" => timestamp,
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
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_success();
        println!("Deployed The contract!");
        // ---------------------------------------
        
        // Get the contract hash from NAMED_KEYS of the deployer account
        let contract_hash = builder
            .get_expected_account(account_addr)
            .named_keys()
            .get("droplink_contract")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
        println!("Contract hash = {}" , contract_hash);
        // ---------------------------------------

        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 100;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_price : U256 = U256::from_dec_str("100").unwrap();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "price" => mint_price,
                "metadata" => mint_metadata
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
            .into_t::<String>()
            .expect("should be string");

            // check the retreived is the same as the metadata
        assert_eq!(meta, "Nike Shoes,bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii,oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd,100".to_string());
        // --------------------------------------

    }

    #[test]
    fn publish_request_entry_point(){
        /*
         * 1. Deploy the contract
         * 2. Mint a product using producer's account
         * 3. Send a publish_request using publisher's account
         * 4. verify that the publish_request has been sent on chain 
         */
        println!("Starting...");
        
        // Create Accounts needed for this test
        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

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
        // -------------------------------------

        // Get genesis config up and running, add accounts to it
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        genesis_config.ee_config_mut().push_account(account_pub);
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // --------------------------------------
        
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let timestamp : u64 = 1677241273;
        let session_args = runtime_args! {
            "timestamp" => timestamp,
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
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_success();
        println!("Deployed The contract!");
        // ---------------------------------------
        
        // Get the contract hash from NAMED_KEYS of the deployer account
        let contract_hash = builder
            .get_expected_account(account_addr)
            .named_keys()
            .get("droplink_contract")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
        println!("Contract hash = {}" , contract_hash);
        // ---------------------------------------
        let contract : Contract = builder.get_contract(contract_hash).unwrap();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 2000;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_price : U256 = U256::from_dec_str("100").unwrap();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "price" => mint_price,
                "metadata" => mint_metadata
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
        let publish_comission : u8 = 23;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
                "comission" => publish_comission
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();
        // --------------------------------------
        
        // Verify the publish request
        let requests_dict_uref = contract.named_keys().get("request_objects").unwrap().into_uref().unwrap();
        
        let requests_cnt = builder
            .query(None, Key::Hash(contract_hash.value()), &["request_cnt".to_string()])
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u64>()
            .expect("should be u64.");
        assert_eq!(requests_cnt, 1u64);

        let request = builder
            .query_dictionary_item(None, requests_dict_uref, "1")
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<String>()
            .expect("should be string");
        assert_eq!(request ,"1,10,23,3d5de8c609159a0954e773dd686fb7724428316cb30e00bdc899976127747f55,105b69f2d74a211a6cb337cba6751a8f15cc7b44b7c65329c29731b67e1ac047".to_string());
        // -----------------------------------------
    }

    #[test]
    fn cancel_request_entry_point(){
        /*
         * 1. Deploy the contract
         * 2. Mint a product using producer's account
         * 3. Do a publish_request using publisher's account
         * 4. Do the cancel_request using publisher's account 
         */
        println!("Starting...");
        
        // Create Accounts needed for this test
        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

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
        // -------------------------------------

        // Get genesis config up and running, add accounts to it
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        genesis_config.ee_config_mut().push_account(account_pub);
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // --------------------------------------
        
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let timestamp : u64 = 1677241273;
        let session_args = runtime_args! {
            "timestamp" => timestamp,
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
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_success();
        println!("Deployed The contract!");
        // ---------------------------------------
        
        // Get the contract hash from NAMED_KEYS of the deployer account
        let contract_hash = builder
            .get_expected_account(account_addr)
            .named_keys()
            .get("droplink_contract")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
        println!("Contract hash = {}" , contract_hash);
        // ---------------------------------------
        let contract : Contract = builder.get_contract(contract_hash).unwrap();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 2000;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_price : U256 = U256::from_dec_str("100").unwrap();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "price" => mint_price,
                "metadata" => mint_metadata
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
        let publish_comission : u8 = 23;
        // Do the publish request to the producer
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
                "comission" => publish_comission
            }
        ).build();
        builder
            .exec(contract_publish_request)
            .expect_success()
            .commit();

        let producer_requests_dict_uref = contract.named_keys().get("producer_requests").unwrap().into_uref().unwrap();
        let requests_list : U64list = builder
            .query_dictionary_item(None, producer_requests_dict_uref, producer_account_addr.to_string().as_str())
            .expect("should exist dict")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<U64list>()
            .expect("should be U64list");
        assert_eq!(requests_list.list.len() , 1usize);
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
    fn approve_entry_point(){
        /* 
         * 1. Deploy contract
         * 2. Mint product
         * 3. Do publish_request
         * 4. Approve it
         * 5. Verify that the request is approved 
         */
        println!("Starting...");
        
        // Create Accounts needed for this test
        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

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
        // -------------------------------------

        // Get genesis config up and running, add accounts to it
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        genesis_config.ee_config_mut().push_account(account_pub);
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // --------------------------------------
        
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let timestamp : u64 = 1677241273;
        let session_args = runtime_args! {
            "timestamp" => timestamp,
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
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_success();
        println!("Deployed The contract!");
        // ---------------------------------------
        
        // Get the contract hash from NAMED_KEYS of the deployer account
        let contract_hash = builder
            .get_expected_account(account_addr)
            .named_keys()
            .get("droplink_contract")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
        println!("Contract hash = {}" , contract_hash);
        // ---------------------------------------
        let contract : Contract = builder.get_contract(contract_hash).unwrap();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 2000;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_price : U256 = U256::from_dec_str("100").unwrap();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "price" => mint_price,
                "metadata" => mint_metadata
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
        let publish_comission : u8 = 23;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
                "comission" => publish_comission
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

        // If you want to change the data of keys on the top of the file, you should edit account hashes below on you own
        assert_eq!(format!("{}" , approved_nft).to_string() , "{\"holder_id\":\"1\",\"amount\":\"10\",\"owneraccount\":\"3d5de8c609159a0954e773dd686fb7724428316cb30e00bdc899976127747f55\",\"publisheraccount\":\"105b69f2d74a211a6cb337cba6751a8f15cc7b44b7c65329c29731b67e1ac047\",\"token_id\":\"1\",\"percentage\":\"23\"}");
    }

    #[test]
    fn disapprove_entry_point(){
        /*
         * 1. Delpoy contract
         * 2. Mint product using producer account
         * 3. Send publish request using publisher account
         * 4. Approve it using producer's account
         * 5. Disapprove a portion of it and verify it 
         */
        println!("Starting...");
        
        // Create Accounts needed for this test
        let secret_key = SecretKey::ed25519_from_bytes(DEPLOYER_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);
        
        let secret_key_publisher = SecretKey::ed25519_from_bytes(PUBLISHER_ACCOUNT).unwrap();
        let public_key_publisher = PublicKey::from(&secret_key_publisher);
        let publisher_account_addr = AccountHash::from(&public_key_publisher);

        let secret_key_producer = SecretKey::ed25519_from_bytes(PRODUCER_ACCOUNT).unwrap();
        let public_key_producer = PublicKey::from(&secret_key_producer);
        let producer_account_addr = AccountHash::from(&public_key_producer);

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
        // -------------------------------------

        // Get genesis config up and running, add accounts to it
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);
        genesis_config.ee_config_mut().push_account(account_prod);
        genesis_config.ee_config_mut().push_account(account_pub);
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // --------------------------------------
        
        // Deploying the contract
        let session_code = PathBuf::from(CONTRACT_WASM);
        let timestamp : u64 = 1677241273;
        let session_args = runtime_args! {
            "timestamp" => timestamp,
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
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();
        builder.exec(execute_request).commit().expect_success();
        println!("Deployed The contract!");
        // ---------------------------------------
        
        // Get the contract hash from NAMED_KEYS of the deployer account
        let contract_hash = builder
            .get_expected_account(account_addr)
            .named_keys()
            .get("droplink_contract")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
        println!("Contract hash = {}" , contract_hash);
        // ---------------------------------------
        let contract : Contract = builder.get_contract(contract_hash).unwrap();
        
        // Call the Mint EntryPoint of the deployed contract, and mint a product with producer account
        let mint_amount : u64 = 2000;
        let mint_recipient : Key = producer_account_addr.into();
        let mint_price : U256 = U256::from_dec_str("100").unwrap();
        let mint_name = "Nike Shoes";
        let mint_token_uri = "bafkreibjrxjhy7evb7e5rp6sfyp6rqi2slczpgl3p2pafqhqn7xx226rii";
        let mint_checksum = "oijepriwguhjpersijf[aopcoisemriguhspiodcpsoeiruhgd";
        let mint_metadata = format!("{{\"name\" : \"{}\", \"token_uri\" : \"{}\" , \"checksum\" : \"{}\"}}", mint_name, mint_token_uri, mint_checksum).to_string();
        let contract_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            producer_account_addr,
            contract_hash,
            "mint",
            runtime_args! {
                "amount" => mint_amount,
                "recipient" => mint_recipient,
                "price" => mint_price,
                "metadata" => mint_metadata
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
        let publish_comission : u8 = 23;
        let contract_publish_request = ExecuteRequestBuilder::contract_call_by_hash(
            publisher_account_addr,
            contract_hash,
            "publish_request",
            runtime_args! {
                "producer-account" => publish_prod_acc,
                "amount" => publish_amount,
                "holder_id" => publish_holder_id,
                "comission" => publish_comission
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

}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
