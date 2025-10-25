#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct Token {
    pub name: felt252,
    pub symbol: felt252,
    pub decimals: u8,
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct UserBalance {
    pub token: felt252, // Token address as felt252
    pub amount: u256 // Amount with support for large numbers
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct Order {
    pub order_id: u128, // Unique order ID
    pub user: starknet::ContractAddress, // Who placed the order
    pub token_in: felt252,
    pub token_out: felt252,
    pub amount_in: u256,
    pub min_amount_out: u256,
}

// Oracle-related types

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct PriceData {
    pub price: u256,          // Price value
    pub decimals: u8,         // Decimal precision
    pub timestamp: u64,       // When the price was updated
    pub round_id: u128,       // Oracle round ID
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct OracleConfig {
    pub oracle_address: starknet::ContractAddress,  // Address of price oracle contract
    pub max_staleness: u64,                         // Maximum age of price data in seconds
    pub is_active: bool,                            // Whether this oracle is active
}

#[derive(Copy, Drop, Serde)]
pub struct SwapQuote {
    pub amount_out: u256,           // Calculated output amount
    pub price: u256,                // Oracle price used
    pub min_amount_out: u256,       // Minimum acceptable output
    pub slippage_bps: u256,         // Slippage in basis points (100 bps = 1%)
}

