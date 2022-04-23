use crate::error::*;
use anchor_lang::prelude::*;

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

    pub fn validate_fields(&self) -> Result<()> {
        match *self {
            RewardType::InterestRate { denom: 0, .. }
            | RewardType::Proportional {
                reward_period: 0, ..
            }
            | RewardType::Fixed {
                required_period: 0, ..
            } => err!(StakingError::Overflow),
            _ => Ok(()),
        }
    }

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
                let rewards_count = ts - *last_reward_ts;
                *last_reward_ts += rewards_count;

                staked_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_mul(num)
                    .ok_or(StakingError::Overflow)?
                    / denom
            }
            RewardType::Proportional {
                total_amount,
                reward_period,
            } => {
                let rewards_count = (ts - *last_reward_ts) / reward_period;
                *last_reward_ts += rewards_count * reward_period;

                staked_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_mul(total_amount)
                    .ok_or(StakingError::Overflow)?
                    / stakes_sum
            }
            RewardType::Fixed {
                required_amount,
                required_period,
                reward_amount,
            } => {
                if staked_amount < required_amount {
                    return Ok(0);
                }

                let rewards_count = (ts - *last_reward_ts) / required_period;
                *last_reward_ts += rewards_count * required_period;

                reward_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
            }
        };

        Ok(reward_amount)
    }
}
