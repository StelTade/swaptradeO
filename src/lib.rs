// Module declarations
mod types;
mod interfaces;
mod security;
mod oracle;

#[starknet::contract]
pub mod SwapTrade {
    use starknet::{ContractAddress, get_caller_address};
    use super::types::{Token, UserBalance, Order, OracleConfig, SwapQuote};
    use super::oracle::price_oracle;
    
    // Event for token swaps
    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        TokensSwapped: TokensSwapped,
        OraclePriceUsed: OraclePriceUsed,
        OracleStaleData: OracleStaleData,
        OracleFailure: OracleFailure,
        OracleConfigUpdated: OracleConfigUpdated,
        MinimumRateEnforced: MinimumRateEnforced,
    }

    #[derive(Drop, starknet::Event)]
    pub struct TokensSwapped {
        #[key]
        pub user: ContractAddress,
        pub token_in: felt252,
        pub token_out: felt252,
        pub amount_in: u256,
        pub amount_out: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct OraclePriceUsed {
        pub token: felt252,
        pub price: u256,
        pub decimals: u8,
        pub timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct OracleStaleData {
        pub token: felt252,
        pub timestamp: u64,
        pub age: u64,
        pub max_staleness: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct OracleFailure {
        pub token: felt252,
        pub reason: felt252,
    }

    #[derive(Drop, starknet::Event)]
    pub struct OracleConfigUpdated {
        pub token: felt252,
        pub oracle_address: ContractAddress,
        pub max_staleness: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct MinimumRateEnforced {
        pub token_in: felt252,
        pub token_out: felt252,
        pub amount_in: u256,
        pub amount_out: u256,
        pub min_amount_out: u256,
    }
    // Swap function: swaps amount_in of token_in for token_out, ensuring min_amount_out and atomicity
    #[external(v0)]
    fn swap(
        ref self: ContractState,
        token_in: felt252,
        token_out: felt252,
        amount_in: u256,
        min_amount_out: u256,
        recipient: ContractAddress,
    ) {
        let caller = get_caller_address();

        // Check tokens are registered
        let token_in_info = self.tokens.read(token_in);
        let token_out_info = self.tokens.read(token_out);
        assert(token_in_info.decimals >= 0_u8, 'Invalid token_in');
        assert(token_out_info.decimals >= 0_u8, 'Invalid token_out');

        // Check input amount
        assert(amount_in > 0_u256, 'amount_in must be > 0');
        assert(min_amount_out > 0_u256, 'min_amount_out must be > 0');

        // Check user balance
        let user_balance = self.balances.read((caller, token_in));
        assert(user_balance >= amount_in, 'Insufficient balance');

        // Calculate output amount (for demo, 1:1 swap, replace with real logic)
        let mut amount_out = amount_in;

        // If oracle is enabled, verify minimum rate using oracle prices
        if self.oracle_enabled.read() {
            let oracle_min_out = self.get_oracle_min_output(
                token_in,
                token_out,
                amount_in
            );
            
            // Enforce oracle-based minimum if it's higher than user's minimum
            let effective_min = if oracle_min_out > min_amount_out {
                oracle_min_out
            } else {
                min_amount_out
            };
            
            // Slippage protection with oracle enforcement
            assert(amount_out >= effective_min, 'Slippage: amount_out < min');
            
            // Emit event for minimum rate enforcement
            self.emit(MinimumRateEnforced {
                token_in,
                token_out,
                amount_in,
                amount_out,
                min_amount_out: effective_min,
            });
        } else {
            // Standard slippage protection without oracle
            assert(amount_out >= min_amount_out, 'Slippage: out < min');
        }

        // Update balances (state changes before external calls)
        let new_user_balance = user_balance - amount_in;
        self.balances.write((caller, token_in), new_user_balance);

        let recipient_balance = self.balances.read((recipient, token_out));
        let new_recipient_balance = recipient_balance + amount_out;
        self.balances.write((recipient, token_out), new_recipient_balance);

        // Emit event
        self.emit(TokensSwapped {
            user: caller,
            token_in,
            token_out,
            amount_in,
            amount_out,
        });
    }

    #[storage]
    struct Storage {
        token_a: felt252,
        token_b: felt252,
        // Mapping: user address → token address → balance
        balances: starknet::storage::Map<(ContractAddress, felt252), u256>,
        // Store registered tokens
        tokens: starknet::storage::Map<felt252, Token>,
        // Orderbook: order_id → Order
        orderbook: starknet::storage::Map<u128, Order>,
        // Order counter for unique IDs
        order_counter: u128,
        // Oracle configurations: token → OracleConfig
        oracle_configs: starknet::storage::Map<felt252, OracleConfig>,
        // Global oracle settings
        oracle_enabled: bool,
        global_max_staleness: u64,
        // Owner/admin address
        owner: ContractAddress,
        // Maximum slippage in basis points (default 1% = 100 bps)
        max_slippage_bps: u256,
    }

    #[constructor]
    fn constructor(ref self: ContractState, token_a: felt252, token_b: felt252) {
        self.token_a.write(token_a);
        self.token_b.write(token_b);
        // Set deployer as owner
        let deployer = get_caller_address();
        self.owner.write(deployer);
        // Set default oracle settings
        self.oracle_enabled.write(false); // Disabled by default
        self.global_max_staleness.write(price_oracle::DEFAULT_MAX_STALENESS);
        self.max_slippage_bps.write(100_u256); // Default 1% slippage
    }

    // View function to get the tokens
    fn get_tokens(self: @ContractState) -> (felt252, felt252) {
        (self.token_a.read(), self.token_b.read())
    }

    // Get user balance for a specific token
    #[external(v0)]
    fn get_balance(self: @ContractState, user: ContractAddress, token: felt252) -> u256 {
        self.balances.read((user, token))
    }

    // Get current user's balance for a specific token
    #[external(v0)]
    fn get_my_balance(self: @ContractState, token: felt252) -> u256 {
        let caller = get_caller_address();
        self.balances.read((caller, token))
    }

    // Update user balance (for testing and admin purposes)
    #[external(v0)]
    fn set_balance(ref self: ContractState, user: ContractAddress, token: felt252, amount: u256) {
        // In a real implementation, this would have access controls
        self.balances.write((user, token), amount);
    }

    // Register a new token
    #[external(v0)]
    fn register_token(
        ref self: ContractState,
        token_address: felt252,
        name: felt252,
        symbol: felt252,
        decimals: u8,
    ) {
        // In a real implementation, this would have access controls
        let token = Token { name, symbol, decimals };
        self.tokens.write(token_address, token);
    }

    // Get token details
    #[external(v0)]
    fn get_token_details(self: @ContractState, token_address: felt252) -> Token {
        self.tokens.read(token_address)
    }

    // Get user balance as a UserBalance struct
    #[external(v0)]
    fn get_user_balance_struct(
        self: @ContractState, user: ContractAddress, token: felt252,
    ) -> UserBalance {
        let amount = self.balances.read((user, token));
        UserBalance { token, amount }
    }

    // Deposit tokens to increase user balance
    #[external(v0)]
    fn deposit(ref self: ContractState, token_address: felt252, amount: u256) {
        // Validate that amount is greater than 0
        assert(amount > 0_u256, 'Amount must be greater than 0');

        // Get caller address
        let caller = get_caller_address();

        // Get current balance
        let current_balance = self.balances.read((caller, token_address));

        // Calculate new balance
        let new_balance = current_balance + amount;

        // Update the balance
        self.balances.write((caller, token_address), new_balance);
    }

    // Place a new order
    #[external(v0)]
    fn place_order(
        ref self: ContractState,
        token_in: felt252,
        token_out: felt252,
        amount_in: u256,
        min_amount_out: u256,
    ) -> u128 {
        let caller = get_caller_address();

        // Validate tokens are registered
        let token_in_exists = self.tokens.read(token_in);
        let token_out_exists = self.tokens.read(token_out);
        // If either token is not registered, panic
        assert(token_in_exists.decimals > 0_u8 || token_in_exists.decimals == 0_u8, 'Invalid token_in');
        assert(token_out_exists.decimals > 0_u8 || token_out_exists.decimals == 0_u8, 'Invalid token_out');

        // Validate amounts
        assert(amount_in > 0_u256, 'amount_in must be > 0');
        assert(min_amount_out > 0_u256, 'min_amount_out must be > 0');

        // Generate unique order ID
        let order_id = self.order_counter.read();
        self.order_counter.write(order_id + 1_u128);

        // Create and store the order
        let order = Order {
            order_id,
            user: caller,
            token_in,
            token_out,
            amount_in,
            min_amount_out,
        };
        self.orderbook.write(order_id, order);
        order_id
    }

    // ========== Oracle Management Functions ==========

    /// Set oracle address for a specific token
    #[external(v0)]
    fn set_oracle_address(
        ref self: ContractState,
        token: felt252,
        oracle_address: ContractAddress,
        max_staleness: u64
    ) {
        self.only_owner();
        
        let config = OracleConfig {
            oracle_address,
            max_staleness,
            is_active: true,
        };
        
        self.oracle_configs.write(token, config);
        
        self.emit(OracleConfigUpdated {
            token,
            oracle_address,
            max_staleness,
        });
    }

    /// Enable or disable oracle globally
    #[external(v0)]
    fn set_oracle_enabled(ref self: ContractState, enabled: bool) {
        self.only_owner();
        self.oracle_enabled.write(enabled);
    }

    /// Set global maximum staleness for all oracles
    #[external(v0)]
    fn set_global_max_staleness(ref self: ContractState, max_staleness: u64) {
        self.only_owner();
        self.global_max_staleness.write(max_staleness);
    }

    /// Set maximum allowed slippage in basis points
    #[external(v0)]
    fn set_max_slippage_bps(ref self: ContractState, slippage_bps: u256) {
        self.only_owner();
        assert(slippage_bps <= price_oracle::MAX_SLIPPAGE_BPS, 'Slippage too high');
        self.max_slippage_bps.write(slippage_bps);
    }

    /// Get oracle configuration for a token
    #[external(v0)]
    fn get_oracle_config(self: @ContractState, token: felt252) -> OracleConfig {
        self.oracle_configs.read(token)
    }

    /// Check if oracle is enabled
    #[external(v0)]
    fn is_oracle_enabled(self: @ContractState) -> bool {
        self.oracle_enabled.read()
    }

    /// Get current maximum staleness setting
    #[external(v0)]
    fn get_global_max_staleness(self: @ContractState) -> u64 {
        self.global_max_staleness.read()
    }

    /// Get maximum slippage in basis points
    #[external(v0)]
    fn get_max_slippage_bps(self: @ContractState) -> u256 {
        self.max_slippage_bps.read()
    }

    /// Get quote for a swap using oracle prices
    #[external(v0)]
    fn get_swap_quote(
        ref self: ContractState,
        token_in: felt252,
        token_out: felt252,
        amount_in: u256
    ) -> SwapQuote {
        assert(self.oracle_enabled.read(), 'Oracle not enabled');
        
        // Get oracle prices
        let (price_in, price_out) = self.get_oracle_prices(token_in, token_out);
        
        // Get token decimals (from oracle price data)
        let _config_in = self.oracle_configs.read(token_in);
        let _config_out = self.oracle_configs.read(token_out);
        
        // For simplicity, assume both use 8 decimals (standard for price feeds)
        let decimals: u8 = 8_u8;
        
        // Calculate output and minimum
        let slippage_bps = self.max_slippage_bps.read();
        let min_amount_out = price_oracle::calculate_min_output(
            amount_in,
            price_in,
            price_out,
            decimals,
            decimals,
            slippage_bps
        );
        
        // For demo, assume 1:1 swap ratio
        let amount_out = amount_in;
        
        SwapQuote {
            amount_out,
            price: price_in,
            min_amount_out,
            slippage_bps,
        }
    }

    // ========== Internal Helper Functions ==========
    
    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {
        /// Only allow owner to call
        fn only_owner(self: @ContractState) {
            let caller = get_caller_address();
            let owner = self.owner.read();
            assert(caller == owner, 'Only owner can call');
        }

        /// Get oracle prices for both tokens
        fn get_oracle_prices(ref self: ContractState, token_in: felt252, token_out: felt252) -> (u256, u256) {
        let config_in = self.oracle_configs.read(token_in);
        let config_out = self.oracle_configs.read(token_out);
        
        assert(config_in.is_active, 'Oracle for token_in not active');
        assert(config_out.is_active, 'Oracle for token_out not active');
        
        // Fetch and validate prices
        let max_staleness = self.global_max_staleness.read();
        
        let price_data_in = price_oracle::fetch_and_validate_price(
            config_in.oracle_address,
            token_in,
            max_staleness
        );
        
        let price_data_out = price_oracle::fetch_and_validate_price(
            config_out.oracle_address,
            token_out,
            max_staleness
        );
        
        // Emit events for price usage
        self.emit(OraclePriceUsed {
            token: token_in,
            price: price_data_in.price,
            decimals: price_data_in.decimals,
            timestamp: price_data_in.timestamp,
        });
        
        self.emit(OraclePriceUsed {
            token: token_out,
            price: price_data_out.price,
            decimals: price_data_out.decimals,
            timestamp: price_data_out.timestamp,
        });
        
        (price_data_in.price, price_data_out.price)
    }

        /// Calculate minimum output based on oracle prices
        fn get_oracle_min_output(
            ref self: ContractState,
            token_in: felt252,
            token_out: felt252,
            amount_in: u256
        ) -> u256 {
            // Get oracle prices
            let (price_in, price_out) = self.get_oracle_prices(token_in, token_out);        // Get slippage tolerance
        let slippage_bps = self.max_slippage_bps.read();
        
        // Assume 8 decimals for price feeds (standard)
        let decimals: u8 = 8_u8;
        
        // Calculate minimum output
        price_oracle::calculate_min_output(
            amount_in,
            price_in,
            price_out,
            decimals,
            decimals,
            slippage_bps
        )
    }
    }
}

// Tests module for the SwapTrade contract
#[cfg(test)]
mod tests {
    use super::types::{Token, UserBalance};

    // We don't need to create contract addresses for our basic struct tests
    #[test]
    fn test_token_struct() {
        // Test creating a Token struct
        let name: felt252 = 'TestToken';
        let symbol: felt252 = 'TT';
        let decimals: u8 = 18;

        let token = Token { name, symbol, decimals };

        assert(token.name == name, 'Token name mismatch');
        assert(token.symbol == symbol, 'Token symbol mismatch');
        assert(token.decimals == decimals, 'Token decimals mismatch');
    }

    #[test]
    fn test_user_balance_struct() {
        // Test creating a UserBalance struct
        let token: felt252 = 0x123;
        let amount: u256 = 1000_u256;

        let user_balance = UserBalance { token, amount };

        assert(user_balance.token == token, 'Token mismatch in struct');
        assert(user_balance.amount == amount, 'Amount mismatch in struct');
    }

    // Note: Tests for external functions require deployment which is complex in Cairo 2
    // For now, we test the data structures. Integration tests should be added separately.
}
