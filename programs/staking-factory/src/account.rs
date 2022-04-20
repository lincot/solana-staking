use anchor_lang::prelude::*;

#[account]
pub struct Factory {
    pub bump: u8,
    pub authority: Pubkey,
    pub stakings_count: u16,
}
impl Factory {
    pub const LEN: usize = 1 + 32 + 2;
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug)]
pub enum RewardAmount {
    Absolute { num: u64, denom: u64 },
    Relative { total_amount: u64 },
}
impl RewardAmount {
    pub const LEN: usize = 1 + 8 + 8;
}

#[account]
pub struct Staking {
    pub bump: u8,
    pub bump_vault: u8,
    pub authority: Pubkey,
    pub id: u16,
    pub withdrawal_timelock: i64,
    pub mint: Pubkey,
    pub reward_period: i64,
    pub reward_amount: RewardAmount,
    pub stakes_sum: u64,
}
impl Staking {
    pub const LEN: usize = 1 + 1 + 32 + 2 + 8 + 32 + 8 + RewardAmount::LEN + 8;
}

#[account]
pub struct Member {
    pub bump: u8,
    pub bump_available: u8,
    pub bump_stake: u8,
    pub bump_pending: u8,
    pub last_reward_ts: i64,
}
impl Member {
    pub const LEN: usize = 1 + 1 + 1 + 1 + 8;
}

#[account]
pub struct PendingWithdrawal {
    pub bump: u8,
    pub burned: bool,
    pub start_ts: i64,
    pub end_ts: i64,
    pub amount: u64,
}
impl PendingWithdrawal {
    pub const LEN: usize = 1 + 1 + 8 + 8 + 8;
}
