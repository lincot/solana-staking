use crate::reward::*;
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

#[account]
pub struct Staking {
    pub bump: u8,
    pub bump_vault: u8,
    pub authority: Pubkey,
    pub id: u16,
    pub withdrawal_timelock: u32,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_params: RewardParams,
    pub stakes_sum: u64,
}
impl Staking {
    pub const LEN: usize = 1 + 1 + 32 + 2 + 4 + 32 + 32 + RewardParams::LEN + 8;
}

#[account]
pub struct ConfigHistory {
    pub bump: u8,
    pub len: u8,
    pub reward_params: [RewardParams; 32],
    pub start_timestamps: [u32; 32],
}
impl ConfigHistory {
    pub const LEN: usize = 1 + 1 + (RewardParams::LEN + 4) * 32;
}

#[account]
pub struct StakesHistory {
    pub bump: u8,
    pub len: u8,
    pub stakes_sums: [u64; 128],
    /// first stakes_sum for each config
    pub offsets: [u8; 32],
}
impl StakesHistory {
    pub const LEN: usize = 1 + 1 + 8 * 128 + 32;
}

#[account]
pub struct Member {
    pub bump: u8,
    pub available_amount: u64,
    pub stake_amount: u64,
    pub pending_amount: u64,
    pub rewards_amount: u64,
    pub last_reward_ts: u32,
}
impl Member {
    pub const LEN: usize = 1 + 8 + 8 + 8 + 8 + 4;
}

#[account]
pub struct PendingWithdrawal {
    pub bump: u8,
    pub active: bool,
    pub end_ts: u32,
    pub amount: u64,
}
impl PendingWithdrawal {
    pub const LEN: usize = 1 + 1 + 4 + 8;
}
