use crate::error::*;
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
pub enum RewardType {
    InterestRate {
        num: u64,
        denom: u64,
    },
    Proportional {
        total_amount: u64,
        reward_period: u32,
    },
    Fixed {
        required_amount: u64,
        required_period: u32,
        reward_amount: u64,
    },
}
impl RewardType {
    pub const LEN: usize = 1 + 8 + 2 + 8;

    pub fn get_reward_amount(
        &self,
        staked_amount: u64,
        stakes_sum: u64,
        ts: u32,
        last_reward_ts: &mut u32,
    ) -> Result<u64> {
        if *last_reward_ts == 0 {
            *last_reward_ts = ts;
            return Ok(0);
        }

        let reward_amount = match *self {
            RewardType::InterestRate { num, denom } => {
                staked_amount
                    .checked_mul((ts - *last_reward_ts) as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_mul(num)
                    .ok_or(StakingError::Overflow)?
                    / denom
            }
            RewardType::Proportional {
                total_amount,
                reward_period,
            } => {
                if ts - *last_reward_ts < reward_period {
                    return Ok(0);
                }

                staked_amount
                    .checked_mul(total_amount)
                    .ok_or(StakingError::Overflow)?
                    / stakes_sum
            }
            RewardType::Fixed {
                required_amount,
                required_period,
                reward_amount,
            } => {
                if ts - *last_reward_ts < required_period || staked_amount < required_amount {
                    return Ok(0);
                }

                reward_amount
            }
        };

        *last_reward_ts = ts;

        Ok(reward_amount)
    }
}

#[account]
pub struct Staking {
    pub bump: u8,
    pub bump_vault: u8,
    pub authority: Pubkey,
    pub id: u16,
    pub withdrawal_timelock: u32,
    pub mint: Pubkey,
    pub reward_type: RewardType,
    pub stakes_sum: u64,
}
impl Staking {
    pub const LEN: usize = 1 + 1 + 32 + 2 + 4 + 32 + RewardType::LEN + 8;
}

#[account]
pub struct Member {
    pub bump: u8,
    pub bump_available: u8,
    pub bump_stake: u8,
    pub bump_pending: u8,
    pub last_reward_ts: u32,
    pub unclaimed_rewards: u64,
}
impl Member {
    pub const LEN: usize = 1 + 1 + 1 + 1 + 4 + 8;
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
