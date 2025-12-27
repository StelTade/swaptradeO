# Oracle Integration Guide

## Overview

The SwapTrade contract now includes a sophisticated price oracle integration system that protects users from front-running, price manipulation, and unfavorable swap rates. This integration follows industry best practices similar to Chainlink's oracle design.

## Features

### 1. **Decentralized Price Feeds**

- Integrates with external price oracle contracts
- Fetches real-time token prices for swap validation
- Supports multiple oracle providers per token

### 2. **Minimum Rate Enforcement**

- Automatically calculates fair minimum output amounts based on oracle prices
- Compares swap outputs against oracle-backed minimums
- Reverts transactions that fall below acceptable thresholds

### 3. **Stale Data Protection**

- Validates timestamp of oracle price updates
- Configurable maximum data age (default: 1 hour)
- Prevents swaps when price data is outdated

### 4. **Slippage Management**

- Configurable slippage tolerance (default: 1% = 100 basis points)
- Maximum slippage capped at 5% for safety
- Enforced at the protocol level

### 5. **Event Emissions**

- `OraclePriceUsed`: Logs price data usage
- `OracleStaleData`: Alerts when data is too old
- `OracleFailure`: Records oracle access failures
- `OracleConfigUpdated`: Tracks configuration changes
- `MinimumRateEnforced`: Documents rate protection triggers

## Architecture

### Core Components

```
src/
├── interfaces/
│   └── IPriceOracle.cairo        # Oracle interface (similar to Chainlink)
├── oracle/
│   └── price_oracle.cairo         # Price validation and calculation logic
├── types.cairo                    # Oracle-related data structures
└── lib.cairo                      # Main contract with oracle integration
```

### Key Data Structures

#### `PriceData`

```cairo
struct PriceData {
    price: u256,          // Price value
    decimals: u8,         // Decimal precision
    timestamp: u64,       // Update timestamp
    round_id: u128,       // Oracle round ID
}
```

#### `OracleConfig`

```cairo
struct OracleConfig {
    oracle_address: ContractAddress,  // Oracle contract address
    max_staleness: u64,               // Maximum age in seconds
    is_active: bool,                  // Active status
}
```

#### `SwapQuote`

```cairo
struct SwapQuote {
    amount_out: u256,           // Expected output
    price: u256,                // Oracle price used
    min_amount_out: u256,       // Minimum acceptable output
    slippage_bps: u256,         // Slippage in basis points
}
```

## Usage

### Administrator Functions

#### Setting Up Oracles

```cairo
// Set oracle for a token
set_oracle_address(
    token: felt252,                    // Token identifier
    oracle_address: ContractAddress,   // Oracle contract
    max_staleness: u64                 // Max age in seconds
)
```

#### Enabling Oracle Protection

```cairo
// Enable/disable oracle globally
set_oracle_enabled(enabled: bool)

// Set global staleness limit
set_global_max_staleness(max_staleness: u64)

// Set maximum slippage
set_max_slippage_bps(slippage_bps: u256)
```

### User Functions

#### Getting Swap Quotes

```cairo
// Get quote with oracle prices
let quote = get_swap_quote(
    token_in: felt252,
    token_out: felt252,
    amount_in: u256
)
// Returns: SwapQuote with expected output and minimum
```

#### Executing Swaps

```cairo
// Swap with oracle protection (if enabled)
swap(
    token_in: felt252,
    token_out: felt252,
    amount_in: u256,
    min_amount_out: u256,     // Your minimum
    recipient: ContractAddress
)
// If oracle is enabled, enforces MAX(min_amount_out, oracle_minimum)
```

### View Functions

```cairo
// Check oracle status
is_oracle_enabled() -> bool

// Get oracle configuration
get_oracle_config(token: felt252) -> OracleConfig

// Get settings
get_global_max_staleness() -> u64
get_max_slippage_bps() -> u256
```

## Configuration Examples

### Example 1: Basic Setup

```cairo
// 1. Set oracle for ETH
set_oracle_address(
    0xETH,                           // ETH token
    0x1234...ORACLE_ADDRESS,         // Chainlink oracle
    3600_u64                         // 1 hour max staleness
)

// 2. Set oracle for USDC
set_oracle_address(
    0xUSDC,
    0x5678...ORACLE_ADDRESS,
    3600_u64
)

// 3. Enable oracle protection
set_oracle_enabled(true)

// 4. Set 0.5% maximum slippage
set_max_slippage_bps(50_u256)
```

### Example 2: Conservative Settings

```cairo
// Short staleness window for high security
set_global_max_staleness(600_u64)   // 10 minutes

// Lower slippage tolerance
set_max_slippage_bps(25_u256)       // 0.25%
```

### Example 3: Flexible Settings

```cairo
// Longer staleness window
set_global_max_staleness(7200_u64)  // 2 hours

// Higher slippage for volatile markets
set_max_slippage_bps(300_u256)      // 3%
```

## Security Considerations

### Stale Data Protection

The system prevents swaps when price data is outdated:

```cairo
// Price must be recent
if (current_time - price_timestamp) > max_staleness {
    // Transaction reverts
}
```

### Price Validation

All prices must meet minimum validity criteria:

```cairo
// Price must be positive
assert(price >= MIN_VALID_PRICE, 'Invalid price')

// Price must not be stale
assert(validate_price_data(price_data), 'Stale data')
```

### Slippage Limits

Protocol enforces maximum slippage:

```cairo
const MAX_SLIPPAGE_BPS: u256 = 500_u256;  // 5% maximum
```

## Oracle Interface Compatibility

The `IPriceOracle` interface is designed to be compatible with Chainlink-style oracles:

```cairo
trait IPriceOracle {
    // Get latest price
    fn get_latest_price(token: felt252) -> (u256, u8, u64);

    // Get specific round data
    fn get_round_data(token: felt252, round_id: u128) -> (u128, u256, u8, u64, u128);

    // Get decimals
    fn decimals(token: felt252) -> u8;

    // Get description
    fn description(token: felt252) -> felt252;

    // Get version
    fn version() -> u32;
}
```

## Error Handling

### Common Errors

| Error                            | Cause                           | Solution                                      |
| -------------------------------- | ------------------------------- | --------------------------------------------- |
| `Oracle: Stale or invalid price` | Price data is too old           | Check oracle is updating regularly            |
| `Oracle for token_in not active` | Oracle not configured for token | Set oracle address for token                  |
| `Oracle not enabled`             | Global oracle disabled          | Enable oracle with `set_oracle_enabled(true)` |
| `Slippage: amount_out < min`     | Output below minimum            | Increase slippage tolerance or retry          |
| `Slippage too high`              | Slippage exceeds 5%             | Use lower slippage value                      |

### Fallback Behavior

When oracle is disabled, the contract operates in normal mode:

- No oracle price checks
- Only user-specified minimums enforced
- Standard slippage protection applies

## Testing

### Unit Tests

Test oracle utility functions:

```bash
scarb test oracle_price
```

### Integration Tests

Test full oracle integration:

```bash
scarb test oracle_tests
```

### Test Coverage

The test suite covers:

- ✅ Oracle configuration
- ✅ Price validation
- ✅ Stale data detection
- ✅ Slippage calculation
- ✅ Minimum rate enforcement
- ✅ Access control
- ✅ Edge cases

## Price Calculation Formula

### Basic Calculation

```
Expected Output = (Amount In × Price In) / Price Out
```

### With Slippage Protection

```
Minimum Output = Expected Output × (10000 - Slippage BPS) / 10000
```

### Example

```
Amount In: 100 tokens
Price In: $200 per token
Price Out: $100 per token
Slippage: 1% (100 BPS)

Expected Output = (100 × 200) / 100 = 200 tokens
Minimum Output = 200 × (10000 - 100) / 10000 = 198 tokens
```

## Best Practices

### For Administrators

1. **Regular Oracle Updates**: Ensure oracles update frequently
2. **Conservative Staleness**: Use shorter windows for volatile assets
3. **Monitor Events**: Watch for `OracleStaleData` and `OracleFailure` events
4. **Test Oracles**: Verify oracle functionality before enabling

### For Users

1. **Check Quotes**: Use `get_swap_quote()` before swapping
2. **Set Reasonable Minimums**: Don't set minimum too tight in volatile markets
3. **Monitor Slippage**: Understand current slippage settings
4. **Verify Transactions**: Check event logs for confirmation

### For Developers

1. **Implement IPriceOracle**: Follow interface exactly
2. **Update Frequently**: Keep price data fresh
3. **Handle Errors**: Provide clear error messages
4. **Test Thoroughly**: Cover all edge cases

## Deployment Checklist

- [ ] Deploy price oracle contracts
- [ ] Configure oracle addresses for all tokens
- [ ] Set appropriate staleness limits
- [ ] Configure slippage tolerance
- [ ] Test with small amounts first
- [ ] Enable oracle protection
- [ ] Monitor events and logs
- [ ] Document oracle sources

## Constants Reference

```cairo
// Oracle module constants
DEFAULT_MAX_STALENESS: u64 = 3600_u64;    // 1 hour
MIN_VALID_PRICE: u256 = 1_u256;           // Minimum valid price
MAX_SLIPPAGE_BPS: u256 = 500_u256;        // 5% maximum
BPS_DIVISOR: u256 = 10000_u256;           // 100% in basis points
```

## Future Enhancements

Potential improvements for future versions:

1. **Multiple Oracle Support**: Aggregate prices from multiple sources
2. **Dynamic Slippage**: Adjust based on market volatility
3. **Circuit Breakers**: Pause swaps during extreme price movements
4. **Oracle Reputation**: Track oracle reliability
5. **Fallback Oracles**: Use backup oracles if primary fails

## Support

For questions or issues:

- Check test files for usage examples
- Review event logs for debugging
- Consult the CONTRIBUTING.md guide
- Open an issue on GitHub

## License

This oracle integration is part of the SwapTrade contract and follows the same license terms.
