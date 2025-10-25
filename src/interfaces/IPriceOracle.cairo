// SPDX-License-Identifier: MIT
// Price Oracle Interface for fetching token prices
// Similar to Chainlink's AggregatorV3Interface

use starknet::ContractAddress;

#[starknet::interface]
pub trait IPriceOracle<TContractState> {
    /// Get the latest price for a token pair
    /// Returns: (price, decimals, timestamp)
    /// price: The current price with specified decimals
    /// decimals: Number of decimal places in the price
    /// timestamp: Unix timestamp of when the price was last updated
    fn get_latest_price(self: @TContractState, token: felt252) -> (u256, u8, u64);
    
    /// Get price data including round information
    /// Returns: (round_id, price, decimals, timestamp, answered_in_round)
    fn get_round_data(self: @TContractState, token: felt252, round_id: u128) -> (u128, u256, u8, u64, u128);
    
    /// Get the decimals used in price representation
    fn decimals(self: @TContractState, token: felt252) -> u8;
    
    /// Get description/name of the price feed
    fn description(self: @TContractState, token: felt252) -> felt252;
    
    /// Get the version of the oracle
    fn version(self: @TContractState) -> u32;
}

#[starknet::interface]
pub trait IPriceOracleAdmin<TContractState> {
    /// Set oracle address for a specific token pair
    fn set_oracle_address(ref self: TContractState, token: felt252, oracle_address: ContractAddress);
    
    /// Set maximum allowed staleness for price data (in seconds)
    fn set_max_staleness(ref self: TContractState, max_staleness: u64);
    
    /// Enable or disable oracle usage
    fn set_oracle_enabled(ref self: TContractState, enabled: bool);
    
    /// Get oracle address for a token
    fn get_oracle_address(self: @TContractState, token: felt252) -> ContractAddress;
    
    /// Get maximum staleness setting
    fn get_max_staleness(self: @TContractState) -> u64;
    
    /// Check if oracle is enabled
    fn is_oracle_enabled(self: @TContractState) -> bool;
}
