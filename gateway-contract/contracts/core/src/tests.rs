#![cfg(test)]

mod tests {
    use soroban_sdk::{testutils::Address as _, Env, String};
    use crate::types::ChainId;
    use crate::chain_registry::ChainRegistry;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);

        ChainRegistry::initialize(&env, owner.clone());
        
        let retrieved_owner = ChainRegistry::get_owner(&env)
            .expect("Owner should be set");
        
        assert_eq!(retrieved_owner, owner);
    }

    #[test]
    fn test_add_chain_address_ethereum() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "My Ethereum Wallet");

        owner.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            label.clone(),
        );

        assert!(result.is_ok());

        // Verify the address was stored
        let count = ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum);
        assert_eq!(count, 1);

        let stored_addr = ChainRegistry::get_chain_address_at(&env, ChainId::Ethereum, 0)
            .expect("Address should exist");
        
        assert_eq!(stored_addr.address, address);
        assert_eq!(stored_addr.label, label);
        assert_eq!(stored_addr.chain, ChainId::Ethereum);
    }

    #[test]
    fn test_add_chain_address_multiple_chains() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        // Add Ethereum address
        let eth_address = String::from_str(&env, "0xeth");
        let eth_label = String::from_str(&env, "Ethereum Wallet");
        
        owner.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            eth_address.clone(),
            eth_label,
        );
        assert!(result.is_ok());

        // Add Bitcoin address
        let btc_address = String::from_str(&env, "1A1z7agoat");
        let btc_label = String::from_str(&env, "Bitcoin Wallet");
        
        owner.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Bitcoin,
            btc_address.clone(),
            btc_label,
        );
        assert!(result.is_ok());

        // Add Solana address
        let sol_address = String::from_str(&env, "SolAddress123456789");
        let sol_label = String::from_str(&env, "Solana Wallet");
        
        owner.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Solana,
            sol_address.clone(),
            sol_label,
        );
        assert!(result.is_ok());

        // Verify counts
        assert_eq!(ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum), 1);
        assert_eq!(ChainRegistry::get_chain_address_count(&env, ChainId::Bitcoin), 1);
        assert_eq!(ChainRegistry::get_chain_address_count(&env, ChainId::Solana), 1);
    }

    #[test]
    fn test_prevent_duplicate_addresses() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "My Wallet");

        // First addition should succeed
        owner.require_auth();
        let result1 = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            label.clone(),
        );
        assert!(result1.is_ok());

        // Second addition with same address should fail
        owner.require_auth();
        let result2 = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            String::from_str(&env, "Different Label"),
        );
        
        assert!(result2.is_err());
        let error_msg = result2.unwrap_err();
        assert!(error_msg.contains(&String::from_str(&env, "already exists")));

        // Count should still be 1
        assert_eq!(ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum), 1);
    }

    #[test]
    fn test_multiple_addresses_same_chain() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        // Add multiple Ethereum addresses
        let addresses = [
            ("0xAddress1", "Wallet 1"),
            ("0xAddress2", "Wallet 2"),
            ("0xAddress3", "Wallet 3"),
        ];

        for (addr, label) in addresses.iter() {
            let address = String::from_str(&env, addr);
            let label_str = String::from_str(&env, label);
            
            owner.require_auth();
            let result = ChainRegistry::add_chain_address(
                &env,
                ChainId::Ethereum,
                address,
                label_str,
            );
            assert!(result.is_ok());
        }

        // Verify all addresses were stored
        let count = ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum);
        assert_eq!(count, 3);

        // Retrieve and verify each address
        for i in 0..3 {
            let stored = ChainRegistry::get_chain_address_at(&env, ChainId::Ethereum, i)
                .expect("Address should exist");
            assert_eq!(stored.chain, ChainId::Ethereum);
        }
    }

    #[test]
    fn test_unauthorized_add_fails() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        let unauthorized_user = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner);

        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "Wallet");

        // Attempt to add address as unauthorized user
        unauthorized_user.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address,
            label,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains(&String::from_str(&env, "Only owner")));
    }

    #[test]
    fn test_has_chain_address() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "Wallet");

        // Address should not exist initially
        assert!(!ChainRegistry::has_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone()
        ));

        // Add the address
        owner.require_auth();
        let _ = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            label,
        );

        // Address should now exist
        assert!(ChainRegistry::has_chain_address(
            &env,
            ChainId::Ethereum,
            address
        ));
    }

    #[test]
    fn test_remove_chain_address() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "Wallet");

        // Add address
        owner.require_auth();
        let _ = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            label,
        );

        assert_eq!(ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum), 1);

        // Remove address
        owner.require_auth();
        let result = ChainRegistry::remove_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
        );

        assert!(result.is_ok());
        assert_eq!(ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum), 0);
        assert!(!ChainRegistry::has_chain_address(
            &env,
            ChainId::Ethereum,
            address
        ));
    }

    #[test]
    fn test_unauthorized_remove_fails() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        let unauthorized = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "Wallet");

        // Add address as owner
        owner.require_auth();
        let _ = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            label,
        );

        // Try to remove as unauthorized user
        unauthorized.require_auth();
        let result = ChainRegistry::remove_chain_address(
            &env,
            ChainId::Ethereum,
            address,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains(&String::from_str(&env, "Only owner")));
    }

    #[test]
    fn test_change_owner() {
        let env = Env::default();
        let owner = soroban_sdk::Address::generate(&env);
        let new_owner = soroban_sdk::Address::generate(&env);
        
        ChainRegistry::initialize(&env, owner.clone());

        // Change owner
        owner.require_auth();
        let result = ChainRegistry::change_owner(&env, new_owner.clone());
        assert!(result.is_ok());

        // Verify new owner
        let current_owner = ChainRegistry::get_owner(&env).expect("Owner should exist");
        assert_eq!(current_owner, new_owner);

        // Old owner should no longer be able to add addresses
        let address = String::from_str(&env, "0x1234567890123456789012345678901234567890");
        let label = String::from_str(&env, "Wallet");
        
        owner.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address.clone(),
            label.clone(),
        );
        assert!(result.is_err());

        // New owner should be able to add addresses
        new_owner.require_auth();
        let result = ChainRegistry::add_chain_address(
            &env,
            ChainId::Ethereum,
            address,
            label,
        );
        assert!(result.is_ok());
    }
}
