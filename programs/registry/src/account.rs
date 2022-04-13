use anchor_lang::prelude::*;

#[account]
pub struct Registrar {
    pub nonce: u8,
    pub withdrawal_timelock: i64,
    pub reward_queue: Pubkey,
    pub mint: Pubkey,
    pub reward_vault: Pubkey,
    pub reward_period: i64,
    pub reward_type: u8,
    /// if reward_type is 0, it's a percent of member's staked tokens
    /// if reward_type is 1, it's an amount which will be shared according to the share of user stake in total stakes
    pub reward_amount: u64,
    pub stakes_sum: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, Clone, PartialEq)]
pub struct BalanceSandbox {
    pub available: Pubkey,
    pub stake: Pubkey,
    pub pending: Pubkey,
}

#[account]
pub struct Member {
    pub registrar: Pubkey,
    pub beneficiary: Pubkey,
    pub metadata: Pubkey,
    pub balances: BalanceSandbox,
    pub last_reward_ts: i64,
    pub nonce: u8,
}

#[account]
pub struct PendingWithdrawal {
    pub registrar: Pubkey,
    pub member: Pubkey,
    pub burned: bool,
    pub start_ts: i64,
    pub end_ts: i64,
    pub amount: u64,
}
