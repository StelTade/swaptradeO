// SPDX-License-Identifier: MIT
// Interface for ERC20 token in Cairo

#[starknet::interface]
pub trait IERC20<TContractState> {
    fn transfer(ref self: TContractState, recipient: felt252, amount: u256) -> bool;
    fn transfer_from(ref self: TContractState, sender: felt252, recipient: felt252, amount: u256) -> bool;
    fn approve(ref self: TContractState, spender: felt252, amount: u256) -> bool;
    fn balance_of(self: @TContractState, account: felt252) -> u256;
    fn allowance(self: @TContractState, owner: felt252, spender: felt252) -> u256;
}
