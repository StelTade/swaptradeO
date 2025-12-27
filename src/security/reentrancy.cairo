// SPDX-License-Identifier: MIT
// Reentrancy guard for Cairo contracts
// Note: This is a simplified version for Cairo 2.x
// In production, implement proper reentrancy protection in your contract

// Constants for reentrancy status
pub const NOT_ENTERED: felt252 = 1;
pub const ENTERED: felt252 = 2;

// Note: Reentrancy guard should be implemented directly in the contract
// using storage variables and checks before/after function calls
