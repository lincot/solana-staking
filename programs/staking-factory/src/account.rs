use anchor_lang::prelude::*;
use num_enum::TryFromPrimitive;

#[account]
pub struct Factory {
    pub bump: u8,
    pub authority: Pubkey,
    pub stakings_count: u16,
}
impl Factory {
    pub const LEN: usize = 1 + 32 + 2;
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum RewardType {
    Absolute,
    Relative,
}

#[account]
pub struct Staking {
    pub bump: u8,
    pub authority: Pubkey,
    pub factory: Pubkey,
    pub id: u16,
    pub withdrawal_timelock: i64,
    pub mint: Pubkey,
    pub reward_vault: Pubkey,
    pub reward_period: i64,
    pub reward_type: u8,
    /// - if reward_type is Absolute, it's a percent of member's staked tokens
    /// - if reward_type is Relative, it's an amount which will be shared according
    /// to the share of user stake in total stakes
    pub reward_amount: u64,
    pub stakes_sum: u64,
}
impl Staking {
    pub const LEN: usize = 1 + 32 + 32 + 2 + 8 + 32 + 32 + 8 + 1 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, Clone, PartialEq)]
pub struct Balances {
    pub available: Pubkey,
    pub stake: Pubkey,
    pub pending: Pubkey,
}
impl Balances {
    pub const LEN: usize = 3 * 32;
}

#[account]
pub struct Member {
    pub staking: Pubkey,
    pub beneficiary: Pubkey,
    pub metadata: Pubkey,
    pub balances: Balances,
    pub last_reward_ts: i64,
    pub nonce: u8,
}
impl Member {
    pub const LEN: usize = 32 + 32 + 32 + Balances::LEN + 8 + 1;
}

#[account]
pub struct PendingWithdrawal {
    pub staking: Pubkey,
    pub member: Pubkey,
    pub burned: bool,
    pub start_ts: i64,
    pub end_ts: i64,
    pub amount: u64,
}
impl PendingWithdrawal {
    pub const LEN: usize = 32 + 32 + 1 + 8 + 8 + 8;
}
