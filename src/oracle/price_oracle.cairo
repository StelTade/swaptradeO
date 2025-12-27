// SPDX-License-Identifier: MIT
// Price Oracle module for fetching and validating token prices

use starknet::{ContractAddress, get_block_timestamp};
use super::super::types::{PriceData};
use super::super::interfaces::IPriceOracle::{IPriceOracleDispatcher, IPriceOracleDispatcherTrait};

/// Maximum staleness for price data (default: 1 hour = 3600 seconds)
pub const DEFAULT_MAX_STALENESS: u64 = 3600_u64;

/// Minimum valid price (to avoid division by zero and invalid prices)
pub const MIN_VALID_PRICE: u256 = 1_u256;

/// Maximum slippage allowed in basis points (default: 5% = 500 bps)
pub const MAX_SLIPPAGE_BPS: u256 = 500_u256;

/// Basis points divisor (10000 bps = 100%)
pub const BPS_DIVISOR: u256 = 10000_u256;

/// Validates price data from oracle
/// Returns true if price is valid (not stale, not zero)
pub fn validate_price_data(price_data: PriceData, max_staleness: u64) -> bool {
    let current_time = get_block_timestamp();
    
    // Check if price is non-zero
    if price_data.price < MIN_VALID_PRICE {
        return false;
    }
    
    // Check if price data is not stale
    if current_time > price_data.timestamp {
        let age = current_time - price_data.timestamp;
        if age > max_staleness {
            return false;
        }
    }
    
    true
}

/// Fetches price from oracle and validates it
/// Panics if price is invalid or stale
pub fn fetch_and_validate_price(
    oracle_address: ContractAddress,
    token: felt252,
    max_staleness: u64
) -> PriceData {
    // Create oracle dispatcher
    let oracle = IPriceOracleDispatcher { contract_address: oracle_address };
    
    // Fetch latest price
    let (price, decimals, timestamp) = oracle.get_latest_price(token);
    
    // Create price data struct
    let price_data = PriceData {
        price,
        decimals,
        timestamp,
        round_id: 0_u128, // Latest price doesn't have round_id
    };
    
    // Validate price data
    assert(validate_price_data(price_data, max_staleness), 'Oracle: Stale or invalid price');
    
    price_data
}

/// Calculate minimum output amount based on oracle price and slippage
/// amount_in: Input token amount
/// price_in: Price of input token
/// price_out: Price of output token  
/// decimals_in: Decimals of input token price
/// decimals_out: Decimals of output token price
/// slippage_bps: Maximum allowed slippage in basis points
pub fn calculate_min_output(
    amount_in: u256,
    price_in: u256,
    price_out: u256,
    decimals_in: u8,
    decimals_out: u8,
    slippage_bps: u256
) -> u256 {
    assert(price_in > 0_u256, 'Invalid input price');
    assert(price_out > 0_u256, 'Invalid output price');
    assert(slippage_bps <= MAX_SLIPPAGE_BPS, 'Slippage too high');
    
    // Calculate expected output: (amount_in * price_in) / price_out
    let value_in = amount_in * price_in;
    let expected_out = value_in / price_out;
    
    // Adjust for decimal differences if needed
    let adjusted_out = adjust_for_decimals(expected_out, decimals_in, decimals_out);
    
    // Apply slippage: min_out = expected_out * (10000 - slippage_bps) / 10000
    let slippage_multiplier = BPS_DIVISOR - slippage_bps;
    let min_out = (adjusted_out * slippage_multiplier) / BPS_DIVISOR;
    
    min_out
}

/// Adjust amount for decimal differences between tokens
fn adjust_for_decimals(amount: u256, decimals_from: u8, decimals_to: u8) -> u256 {
    if decimals_from == decimals_to {
        return amount;
    }
    
    if decimals_from > decimals_to {
        // Divide by 10^(decimals_from - decimals_to)
        let diff = decimals_from - decimals_to;
        let divisor = pow_10(diff);
        amount / divisor
    } else {
        // Multiply by 10^(decimals_to - decimals_from)
        let diff = decimals_to - decimals_from;
        let multiplier = pow_10(diff);
        amount * multiplier
    }
}

/// Calculate 10^exponent
fn pow_10(exponent: u8) -> u256 {
    let mut result: u256 = 1_u256;
    let mut i: u8 = 0_u8;
    
    loop {
        if i >= exponent {
            break result;
        }
        result = result * 10_u256;
        i += 1_u8;
    }
}

/// Verify swap output meets oracle-based minimum
/// Returns true if actual output >= oracle-calculated minimum
pub fn verify_swap_output(
    amount_in: u256,
    amount_out: u256,
    price_in: u256,
    price_out: u256,
    decimals_in: u8,
    decimals_out: u8,
    max_slippage_bps: u256
) -> bool {
    let min_output = calculate_min_output(
        amount_in,
        price_in,
        price_out,
        decimals_in,
        decimals_out,
        max_slippage_bps
    );
    
    amount_out >= min_output
}

/// Get price ratio between two tokens
/// Returns price_in / price_out scaled by 1e18 for precision
pub fn get_price_ratio(price_in: u256, price_out: u256) -> u256 {
    assert(price_out > 0_u256, 'Division by zero');
    let scale: u256 = 1000000000000000000_u256; // 1e18
    (price_in * scale) / price_out
}

#[cfg(test)]
mod tests {
    use super::{
        validate_price_data, calculate_min_output, adjust_for_decimals, pow_10, 
        verify_swap_output, get_price_ratio,
    };
    use super::super::super::types::PriceData;
    use starknet::testing::set_block_timestamp;

    #[test]
    fn test_validate_price_data_valid() {
        set_block_timestamp(1500_u64); // Set current time
        
        let price_data = PriceData {
            price: 100_u256,
            decimals: 8_u8,
            timestamp: 1000_u64,
            round_id: 1_u128,
        };
        
        // Assume current time is 1500, age is 500 seconds (less than max staleness)
        let result = validate_price_data(price_data, 600_u64);
        assert(result == true, 'Should be valid');
    }

    #[test]
    fn test_validate_price_data_zero_price() {
        set_block_timestamp(1500_u64); // Set current time
        
        let price_data = PriceData {
            price: 0_u256,
            decimals: 8_u8,
            timestamp: 1000_u64,
            round_id: 1_u128,
        };
        
        let result = validate_price_data(price_data, 600_u64);
        assert(result == false, 'Should be invalid (zero)');
    }

    #[test]
    fn test_pow_10() {
        assert(pow_10(0_u8) == 1_u256, 'pow_10(0) should be 1');
        assert(pow_10(1_u8) == 10_u256, 'pow_10(1) should be 10');
        assert(pow_10(2_u8) == 100_u256, 'pow_10(2) should be 100');
        assert(pow_10(3_u8) == 1000_u256, 'pow_10(3) should be 1000');
        assert(pow_10(6_u8) == 1000000_u256, 'pow_10(6) should be 1000000');
    }

    #[test]
    fn test_adjust_for_decimals_equal() {
        let amount = 1000_u256;
        let result = adjust_for_decimals(amount, 18_u8, 18_u8);
        assert(result == 1000_u256, 'Should be unchanged');
    }

    #[test]
    fn test_adjust_for_decimals_increase() {
        let amount = 1000_u256;
        let result = adjust_for_decimals(amount, 6_u8, 8_u8);
        assert(result == 100000_u256, 'Should be multiplied by 100');
    }

    #[test]
    fn test_adjust_for_decimals_decrease() {
        let amount = 1000000_u256;
        let result = adjust_for_decimals(amount, 8_u8, 6_u8);
        assert(result == 10000_u256, 'Should be divided by 100');
    }

    #[test]
    fn test_calculate_min_output_no_slippage() {
        // 100 tokens at price 200 = 20000 value
        // Output price is 100, so expected output = 20000 / 100 = 200
        // With 0 slippage, min output = 200
        let result = calculate_min_output(
            100_u256,     // amount_in
            200_u256,     // price_in
            100_u256,     // price_out
            8_u8,         // decimals_in
            8_u8,         // decimals_out
            0_u256        // slippage_bps (0%)
        );
        assert(result == 200_u256, 'Should be 200');
    }

    #[test]
    fn test_calculate_min_output_with_slippage() {
        // Same as above but with 1% slippage (100 bps)
        // min output = 200 * (10000 - 100) / 10000 = 200 * 0.99 = 198
        let result = calculate_min_output(
            100_u256,     // amount_in
            200_u256,     // price_in
            100_u256,     // price_out
            8_u8,         // decimals_in
            8_u8,         // decimals_out
            100_u256      // slippage_bps (1%)
        );
        assert(result == 198_u256, 'Should be 198');
    }

    #[test]
    fn test_get_price_ratio() {
        let ratio = get_price_ratio(200_u256, 100_u256);
        let expected = 2000000000000000000_u256; // 2 * 1e18
        assert(ratio == expected, 'Ratio should be 2e18');
    }

    #[test]
    fn test_verify_swap_output_success() {
        // amount_in = 100, amount_out = 195
        // price_in = 200, price_out = 100
        // Expected: (100 * 200) / 100 = 200
        // Min with 1% slippage: 200 * 0.99 = 198
        // Actual output 195 < 198, should fail
        let result = verify_swap_output(
            100_u256,
            195_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        assert(result == false, 'Should fail slippage check');
    }

    #[test]
    fn test_verify_swap_output_pass() {
        // Same setup but output is 199 >= 198
        let result = verify_swap_output(
            100_u256,
            199_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        assert(result == true, 'Should pass slippage check');
    }

    // ========== EDGE CASES & ANOMALIES TESTS ==========

    #[test]
    fn test_validate_price_data_stale() {
        set_block_timestamp(1700_u64); // Set current time to 1700
        
        // Price is 700 seconds old, max staleness is 600
        let price_data = PriceData {
            price: 100_u256,
            decimals: 8_u8,
            timestamp: 1000_u64,
            round_id: 1_u128,
        };
        
        // With max staleness of 600, a 700 second old price should be invalid
        let result = validate_price_data(price_data, 600_u64);
        assert(result == false, 'Should be stale');
    }

    #[test]
    fn test_validate_price_data_extremely_stale() {
        set_block_timestamp(10001_u64); // Set current time
        
        // Price is extremely old (10000 seconds)
        let price_data = PriceData {
            price: 100_u256,
            decimals: 8_u8,
            timestamp: 1_u64,
            round_id: 1_u128,
        };
        
        let result = validate_price_data(price_data, 600_u64);
        assert(result == false, 'Extremely stale data');
    }

    #[test]
    fn test_validate_price_data_edge_timestamp() {
        set_block_timestamp(1000_u64); // Set current time
        
        // Price at exactly the staleness boundary
        let price_data = PriceData {
            price: 100_u256,
            decimals: 8_u8,
            timestamp: 400_u64,  // Exactly 600 seconds old at time 1000
            round_id: 1_u128,
        };
        
        // At exactly the boundary, should still be valid
        let result = validate_price_data(price_data, 600_u64);
        assert(result == true, 'Edge timestamp should pass');
    }

    #[test]
    fn test_price_anomaly_extremely_high() {
        set_block_timestamp(1500_u64); // Set current time
        
        // Test with extremely high price (potential oracle malfunction)
        let max_u256 = u256 { low: 340282366920938463463374607431768211455, high: 340282366920938463463374607431768211455 };
        
        let price_data = PriceData {
            price: max_u256,
            decimals: 8_u8,
            timestamp: 1000_u64,
            round_id: 1_u128,
        };
        
        // Should still validate if not stale (anomaly detection would be separate)
        let result = validate_price_data(price_data, 600_u64);
        assert(result == true, 'High price should validate');
    }

    #[test]
    fn test_price_anomaly_minimum_valid() {
        set_block_timestamp(1500_u64); // Set current time
        
        // Test with minimum valid price (1 wei)
        let price_data = PriceData {
            price: 1_u256,
            decimals: 8_u8,
            timestamp: 1000_u64,
            round_id: 1_u128,
        };
        
        let result = validate_price_data(price_data, 600_u64);
        assert(result == true, 'Minimum price should validate');
    }

    #[test]
    fn test_calculate_min_output_zero_amount() {
        // Edge case: zero input amount
        let result = calculate_min_output(
            0_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256
        );
        assert(result == 0_u256, 'Zero input = zero output');
    }

    #[test]
    fn test_calculate_min_output_very_small_amount() {
        // Edge case: very small input amount (1 wei)
        let result = calculate_min_output(
            1_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        // (1 * 200) / 100 = 2, with 1% slippage = 1 (rounded down)
        assert(result >= 1_u256, 'Should handle small amounts');
    }

    #[test]
    fn test_calculate_min_output_maximum_slippage() {
        // Test with maximum allowed slippage (500 bps = 5%)
        let result = calculate_min_output(
            1000_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            500_u256  // 5% slippage (max allowed)
        );
        // Expected: (1000 * 200) / 100 = 2000
        // With 5% slippage: 2000 * 0.95 = 1900
        assert(result == 1900_u256, 'Max slippage calculation');
    }

    #[test]
    #[should_panic(expected: ('Slippage too high',))]
    fn test_calculate_min_output_excessive_slippage() {
        // Oracle downtime scenario: excessive slippage protection
        calculate_min_output(
            100_u256,
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            600_u256  // 6% - exceeds max of 5%
        );
    }

    #[test]
    #[should_panic(expected: ('Invalid input price',))]
    fn test_calculate_min_output_zero_input_price() {
        // Oracle failure: returns zero price for input token
        calculate_min_output(
            100_u256,
            0_u256,  // Oracle downtime/failure
            100_u256,
            8_u8,
            8_u8,
            100_u256
        );
    }

    #[test]
    #[should_panic(expected: ('Invalid output price',))]
    fn test_calculate_min_output_zero_output_price() {
        // Oracle failure: returns zero price for output token
        calculate_min_output(
            100_u256,
            200_u256,
            0_u256,  // Oracle downtime/failure
            8_u8,
            8_u8,
            100_u256
        );
    }

    #[test]
    fn test_verify_swap_extreme_price_difference() {
        // Edge case: extreme price difference between tokens
        let result = verify_swap_output(
            100_u256,
            19900_u256,  // Large output
            20000_u256,  // Very high input price
            100_u256,    // Low output price
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        // Expected: (100 * 20000) / 100 = 20000
        // With 1% slippage: 19800
        // Actual: 19900 >= 19800
        assert(result == true, 'Should handle price extremes');
    }

    #[test]
    fn test_verify_swap_price_manipulation_attempt() {
        // Simulate price manipulation: output far below expected
        let result = verify_swap_output(
            1000_u256,
            1000_u256,   // Suspiciously low output
            200_u256,    // Input price
            100_u256,    // Output price
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        // Expected: (1000 * 200) / 100 = 2000
        // With 1% slippage: 1980
        // Actual: 1000 < 1980 - should fail
        assert(result == false, 'Should detect manipulation');
    }

    #[test]
    fn test_adjust_for_decimals_extreme_difference() {
        // Edge case: extreme decimal difference (18 vs 0)
        let amount = 1000000000000000000_u256; // 1 token with 18 decimals
        let result = adjust_for_decimals(amount, 18_u8, 0_u8);
        // Should divide by 10^18
        assert(result == 1_u256, 'Extreme decimal conversion');
    }

    #[test]
    fn test_adjust_for_decimals_zero_to_eighteen() {
        // Reverse: 0 decimals to 18 decimals
        let amount = 1_u256;
        let result = adjust_for_decimals(amount, 0_u8, 18_u8);
        // Should multiply by 10^18
        assert(result == 1000000000000000000_u256, 'Zero to 18 decimals');
    }

    #[test]
    fn test_get_price_ratio_equal_prices() {
        // Edge case: equal prices
        let ratio = get_price_ratio(100_u256, 100_u256);
        let expected = 1000000000000000000_u256; // 1 * 1e18
        assert(ratio == expected, 'Equal prices = 1:1 ratio');
    }

    #[test]
    fn test_get_price_ratio_very_small_output() {
        // Edge case: very small output price (1 wei)
        let ratio = get_price_ratio(1000000_u256, 1_u256);
        let expected = 1000000000000000000000000_u256; // 1000000 * 1e18
        assert(ratio == expected, 'Small output price');
    }

    #[test]
    #[should_panic(expected: ('Division by zero',))]
    fn test_get_price_ratio_zero_output_price() {
        // Oracle downtime: zero output price causes division by zero
        get_price_ratio(200_u256, 0_u256);
    }

    #[test]
    fn test_verify_swap_output_exact_minimum() {
        // Edge case: output exactly equals minimum required
        let result = verify_swap_output(
            100_u256,
            198_u256,   // Exactly at minimum
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256  // 1% slippage, min = 198
        );
        assert(result == true, 'Exact minimum should pass');
    }

    #[test]
    fn test_verify_swap_output_one_below_minimum() {
        // Edge case: output one wei below minimum
        let result = verify_swap_output(
            100_u256,
            197_u256,   // One below minimum of 198
            200_u256,
            100_u256,
            8_u8,
            8_u8,
            100_u256  // 1% slippage
        );
        assert(result == false, 'One below min should fail');
    }

    #[test]
    fn test_multiple_decimal_conversions() {
        // Test chain of decimal conversions doesn't lose precision
        let amount = 1000000_u256;
        
        // Convert 6->8->18->8->6
        let step1 = adjust_for_decimals(amount, 6_u8, 8_u8);
        let step2 = adjust_for_decimals(step1, 8_u8, 18_u8);
        let step3 = adjust_for_decimals(step2, 18_u8, 8_u8);
        let step4 = adjust_for_decimals(step3, 8_u8, 6_u8);
        
        assert(step4 == amount, 'Round-trip should preserve');
    }
}

