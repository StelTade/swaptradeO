// SPDX-License-Identifier: MIT
// Tests for SwapTrade swap logic

use starknet::testing::{set_caller_address, get_caller_address};
use starknet::{ContractAddress, contract_address_const};
use super::SwapTrade;

#[test]
fn test_swap_success() {
    let mut state = SwapTrade::contract_state_for_testing();
    let user = contract_address_const::<0x111>();
    let recipient = contract_address_const::<0x222>();
    set_caller_address(user);
    // Register tokens
    SwapTrade::register_token(ref state, 0xAAA, 'TokenA', 'A', 18_u8);
    SwapTrade::register_token(ref state, 0xBBB, 'TokenB', 'B', 18_u8);
    // Set balances
    SwapTrade::set_balance(ref state, user, 0xAAA, 1000_u256);
    SwapTrade::set_balance(ref state, recipient, 0xBBB, 500_u256);
    // Swap
    SwapTrade::swap(ref state, 0xAAA, 0xBBB, 100_u256, 100_u256, recipient);
    // Check balances
    let user_balance = SwapTrade::get_balance(@state, user, 0xAAA);
    let recipient_balance = SwapTrade::get_balance(@state, recipient, 0xBBB);
    assert(user_balance == 900_u256, 'User balance not debited');
    assert(recipient_balance == 600_u256, 'Recipient not credited');
}

#[test]
#[should_panic(expected: ('Slippage: amount_out < min_amount_out',))]
fn test_swap_slippage() {
    let mut state = SwapTrade::contract_state_for_testing();
    let user = contract_address_const::<0x111>();
    let recipient = contract_address_const::<0x222>();
    set_caller_address(user);
    SwapTrade::register_token(ref state, 0xAAA, 'TokenA', 'A', 18_u8);
    SwapTrade::register_token(ref state, 0xBBB, 'TokenB', 'B', 18_u8);
    SwapTrade::set_balance(ref state, user, 0xAAA, 1000_u256);
    SwapTrade::set_balance(ref state, recipient, 0xBBB, 500_u256);
    // min_amount_out is higher than swap output
    SwapTrade::swap(ref state, 0xAAA, 0xBBB, 100_u256, 200_u256, recipient);
}

#[test]
#[should_panic(expected: ('Insufficient balance',))]
fn test_swap_insufficient_balance() {
    let mut state = SwapTrade::contract_state_for_testing();
    let user = contract_address_const::<0x111>();
    let recipient = contract_address_const::<0x222>();
    set_caller_address(user);
    SwapTrade::register_token(ref state, 0xAAA, 'TokenA', 'A', 18_u8);
    SwapTrade::register_token(ref state, 0xBBB, 'TokenB', 'B', 18_u8);
    SwapTrade::set_balance(ref state, user, 0xAAA, 50_u256);
    SwapTrade::set_balance(ref state, recipient, 0xBBB, 500_u256);
    // Try to swap more than balance
    SwapTrade::swap(ref state, 0xAAA, 0xBBB, 100_u256, 100_u256, recipient);
}
