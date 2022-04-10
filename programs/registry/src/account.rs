use anchor_lang::prelude::*;

#[account]
pub struct Registrar {
    pub authority: Pubkey,
    pub nonce: u8,
    pub withdrawal_timelock: i64,
    pub reward_queue: Pubkey,
    pub mint: Pubkey,
    pub pool_mint: Pubkey,
    pub stake_rate: u64,
    pub vendor_vault: Pubkey,
    pub reward_amount: u64,
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
    pub last_stake_ts: i64,
    pub last_reward_ts: i64,
    pub nonce: u8,
}

#[account]
pub struct PendingWithdrawal {
    pub registrar: Pubkey,
    pub member: Pubkey,
    pub burned: bool,
    pub pool: Pubkey,
    pub start_ts: i64,
    pub end_ts: i64,
    pub amount: u64,
}
