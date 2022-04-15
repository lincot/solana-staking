use anchor_lang::prelude::*;

#[account]
pub struct Factory {
    pub authority: Pubkey,
    pub stakings_count: u16,
}
impl Factory {
    pub const LEN: usize = 32 + 2;
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum RewardType {
    Absolute,
    Relative,
}

#[account]
pub struct Staking {
    pub bump: u8,
    pub authority: Pubkey,
    pub id: u16,
    pub withdrawal_timelock: i64,
    pub mint: Pubkey,
    pub reward_period: i64,
    pub reward_type: RewardType,
    /// - if reward_type is Absolute, it's a percent of member's staked tokens
    /// - if reward_type is Relative, it's an amount which will be shared according
    /// to the share of user stake in total stakes
    pub reward_amount: u64,
    pub stakes_sum: u64,
}
impl Staking {
    pub const LEN: usize = 1 + 32 + 2 + 8 + 32 + 8 + 1 + 8 + 8;
}

#[account]
pub struct Member {
    pub bump: u8,
    pub last_reward_ts: i64,
}
impl Member {
    pub const LEN: usize = 1 + 8;
}

#[account]
pub struct PendingWithdrawal {
    pub burned: bool,
    pub start_ts: i64,
    pub end_ts: i64,
    pub amount: u64,
}
impl PendingWithdrawal {
    pub const LEN: usize = 1 + 8 + 8 + 8;
}
