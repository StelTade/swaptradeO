// SPDX-License-Identifier: MIT
// Oracle Integration Tests

#[cfg(test)]
mod oracle_tests {
    use starknet::testing::{set_caller_address, set_block_timestamp};
    use starknet::contract_address_const;
    use core::starknet::ContractAddress;
    use swaptrade_contract::SwapTrade;
    use swaptrade_contract::types::{Token, OracleConfig, PriceData};
    use swaptrade_contract::oracle::price_oracle;

    // Mock oracle address for testing
    fn mock_oracle_address() -> ContractAddress {
        contract_address_const::<0xORACLE>()
    }

    // Test setting oracle configuration
    #[test]
    fn test_set_oracle_config() {
        let mut state = SwapTrade::contract_state_for_testing();
        let owner = contract_address_const::<0x123>();
        let token: felt252 = 0xTOKEN;
        let oracle_addr = mock_oracle_address();
        
        // Set owner as caller
        set_caller_address(owner);
        
        // Set oracle configuration
        SwapTrade::set_oracle_address(ref state, token, oracle_addr, 3600_u64);
        
        // Verify configuration was set
        let config = SwapTrade::get_oracle_config(@state, token);
        assert(config.oracle_address == oracle_addr, 'Oracle address mismatch');
        assert(config.max_staleness == 3600_u64, 'Max staleness mismatch');
        assert(config.is_active == true, 'Oracle should be active');
    }

    // Test only owner can set oracle config
    #[test]
    #[should_panic(expected: ('Only owner can call',))]
    fn test_set_oracle_config_non_owner() {
        let mut state = SwapTrade::contract_state_for_testing();
        let non_owner = contract_address_const::<0x456>();
        let token: felt252 = 0xTOKEN;
        let oracle_addr = mock_oracle_address();
        
        // Set non-owner as caller
        set_caller_address(non_owner);
        
        // This should panic
        SwapTrade::set_oracle_address(ref state, token, oracle_addr, 3600_u64);
    }

    // Test enable/disable oracle
    #[test]
    fn test_oracle_enable_disable() {
        let mut state = SwapTrade::contract_state_for_testing();
        let owner = contract_address_const::<0x123>();
        set_caller_address(owner);
        
        // Initially disabled
        assert(SwapTrade::is_oracle_enabled(@state) == false, 'Should be disabled');
        
        // Enable oracle
        SwapTrade::set_oracle_enabled(ref state, true);
        assert(SwapTrade::is_oracle_enabled(@state) == true, 'Should be enabled');
        
        // Disable oracle
        SwapTrade::set_oracle_enabled(ref state, false);
        assert(SwapTrade::is_oracle_enabled(@state) == false, 'Should be disabled again');
    }

    // Test setting global max staleness
    #[test]
    fn test_set_global_max_staleness() {
        let mut state = SwapTrade::contract_state_for_testing();
        let owner = contract_address_const::<0x123>();
        set_caller_address(owner);
        
        // Default should be 3600 (1 hour)
        let default = SwapTrade::get_global_max_staleness(@state);
        assert(default == 3600_u64, 'Default should be 3600');
        
        // Set new staleness
        SwapTrade::set_global_max_staleness(ref state, 7200_u64);
        let new_staleness = SwapTrade::get_global_max_staleness(@state);
        assert(new_staleness == 7200_u64, 'Staleness should be 7200');
    }

    // Test setting max slippage
    #[test]
    fn test_set_max_slippage() {
        let mut state = SwapTrade::contract_state_for_testing();
        let owner = contract_address_const::<0x123>();
        set_caller_address(owner);
        
        // Default should be 100 bps (1%)
        let default = SwapTrade::get_max_slippage_bps(@state);
        assert(default == 100_u256, 'Default should be 100 bps');
        
        // Set new slippage
        SwapTrade::set_max_slippage_bps(ref state, 200_u256);
        let new_slippage = SwapTrade::get_max_slippage_bps(@state);
        assert(new_slippage == 200_u256, 'Slippage should be 200 bps');
    }

    // Test max slippage validation
    #[test]
    #[should_panic(expected: ('Slippage too high',))]
    fn test_set_max_slippage_too_high() {
        let mut state = SwapTrade::contract_state_for_testing();
        let owner = contract_address_const::<0x123>();
        set_caller_address(owner);
        
        // Try to set slippage above max (500 bps = 5%)
        SwapTrade::set_max_slippage_bps(ref state, 600_u256);
    }

    // Test swap with oracle disabled works normally
    #[test]
    fn test_swap_oracle_disabled() {
        let mut state = SwapTrade::contract_state_for_testing();
        let user = contract_address_const::<0x123>();
        set_caller_address(user);
        
        let token_in: felt252 = 0xAAA;
        let token_out: felt252 = 0xBBB;
        
        // Register tokens
        SwapTrade::register_token(ref state, token_in, 'TokenA', 'A', 18_u8);
        SwapTrade::register_token(ref state, token_out, 'TokenB', 'B', 18_u8);
        
        // Set user balance
        SwapTrade::set_balance(ref state, user, token_in, 1000_u256);
        
        // Swap should work without oracle (oracle is disabled by default)
        SwapTrade::swap(
            ref state,
            token_in,
            token_out,
            100_u256,
            90_u256,
            user
        );
        
        // Check balances
        let balance_in = SwapTrade::get_balance(@state, user, token_in);
        let balance_out = SwapTrade::get_balance(@state, user, token_out);
        assert(balance_in == 900_u256, 'Input balance incorrect');
        assert(balance_out == 100_u256, 'Output balance incorrect');
    }
}

// Tests for price oracle utility functions
#[cfg(test)]
mod price_oracle_unit_tests {
    use swaptrade_contract::oracle::price_oracle;
    use swaptrade_contract::types::PriceData;
    use starknet::testing::set_block_timestamp;

    #[test]
    fn test_validate_price_valid() {
        set_block_timestamp(2000_u64);
        
        let price_data = PriceData {
            price: 100_u256,
            decimals: 8_u8,
            timestamp: 1500_u64,  // 500 seconds old
            round_id: 1_u128,
        };
        
        let result = price_oracle::validate_price_data(price_data, 600_u64);
        assert(result == true, 'Should be valid');
    }

    #[test]
    fn test_validate_price_stale() {
        set_block_timestamp(2000_u64);
        
        let price_data = PriceData {
            price: 100_u256,
            decimals: 8_u8,
            timestamp: 500_u64,  // 1500 seconds old
            round_id: 1_u128,
        };
        
        let result = price_oracle::validate_price_data(price_data, 600_u64);
        assert(result == false, 'Should be stale');
    }

    #[test]
    fn test_validate_price_zero() {
        set_block_timestamp(2000_u64);
        
        let price_data = PriceData {
            price: 0_u256,
            decimals: 8_u8,
            timestamp: 1900_u64,
            round_id: 1_u128,
        };
        
        let result = price_oracle::validate_price_data(price_data, 600_u64);
        assert(result == false, 'Should be invalid (zero)');
    }

    #[test]
    fn test_calculate_min_output_basic() {
        // Input: 100 tokens at price 200
        // Output price: 100
        // Expected: (100 * 200) / 100 = 200
        // With 1% slippage: 200 * 0.99 = 198
        let result = price_oracle::calculate_min_output(
            100_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        assert(result == 198_u256, 'Min output incorrect');
    }

    #[test]
    fn test_calculate_min_output_zero_slippage() {
        let result = price_oracle::calculate_min_output(
            100_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            0_u256  // 0% slippage
        );
        assert(result == 200_u256, 'Should be 200');
    }

    #[test]
    #[should_panic(expected: ('Invalid input price',))]
    fn test_calculate_min_output_zero_price_in() {
        price_oracle::calculate_min_output(
            100_u256,
            0_u256,  // Invalid
            100_u256,
            8_u8,
            8_u8,
            100_u256
        );
    }

    #[test]
    #[should_panic(expected: ('Invalid output price',))]
    fn test_calculate_min_output_zero_price_out() {
        price_oracle::calculate_min_output(
            100_u256,
            200_u256,
            0_u256,  // Invalid
            8_u8,
            8_u8,
            100_u256
        );
    }

    #[test]
    #[should_panic(expected: ('Slippage too high',))]
    fn test_calculate_min_output_excessive_slippage() {
        price_oracle::calculate_min_output(
            100_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            600_u256  // 6% - above max of 5%
        );
    }

    #[test]
    fn test_verify_swap_output_pass() {
        let result = price_oracle::verify_swap_output(
            100_u256,   // amount_in
            199_u256,   // amount_out
            200_u256,   // price_in
            100_u256,   // price_out
            8_u8,
            8_u8,
            100_u256    // 1% slippage, min = 198
        );
        assert(result == true, 'Should pass verification');
    }

    #[test]
    fn test_verify_swap_output_fail() {
        let result = price_oracle::verify_swap_output(
            100_u256,   // amount_in
            195_u256,   // amount_out (below min of 198)
            200_u256,   // price_in
            100_u256,   // price_out
            8_u8,
            8_u8,
            100_u256    // 1% slippage
        );
        assert(result == false, 'Should fail verification');
    }

    #[test]
    fn test_get_price_ratio() {
        let ratio = price_oracle::get_price_ratio(200_u256, 100_u256);
        // Expected: (200 * 1e18) / 100 = 2 * 1e18
        let expected = 2000000000000000000_u256;
        assert(ratio == expected, 'Price ratio incorrect');
    }

    #[test]
    #[should_panic(expected: ('Division by zero',))]
    fn test_get_price_ratio_zero_output() {
        price_oracle::get_price_ratio(200_u256, 0_u256);
    }
}
