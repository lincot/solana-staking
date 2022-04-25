use crate::account::*;
use crate::error::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

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
    pub const LEN: usize = 1 + 8 + 4 + 8;

    pub fn validate_fields(&self) -> Result<()> {
        if matches!(
            *self,
            Self::InterestRate { denom: 0, .. }
                | Self::Proportional {
                    reward_period: 0,
                    ..
                }
                | Self::Fixed {
                    required_period: 0,
                    ..
                }
        ) {
            return err!(StakingError::Overflow);
        }

        Ok(())
    }

    pub fn get_reward_amount(
        &self,
        staked_amount: u64,
        stakes_sum: u64,
        last_reward_ts: &mut u32,
        current_ts: u32,
        config_start_ts: u32,
        config_end_ts: u32,
    ) -> Result<u64> {
        if *last_reward_ts == 0 {
            *last_reward_ts = current_ts;
            return Ok(0);
        }

        let start_ts = config_start_ts.max(*last_reward_ts);
        let end_ts = config_end_ts.min(current_ts);

        if start_ts >= end_ts {
            return Ok(0);
        }

        let reward_amount = match *self {
            Self::InterestRate { num, denom } => {
                let rewards_count = end_ts - start_ts;
                *last_reward_ts += rewards_count;

                staked_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_mul(num)
                    .ok_or(StakingError::Overflow)?
                    / denom
            }
            Self::Proportional {
                total_amount,
                reward_period,
                ..
            } => {
                let rewards_count = (end_ts - start_ts) / reward_period;
                *last_reward_ts += rewards_count * reward_period;

                staked_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_mul(total_amount)
                    .ok_or(StakingError::Overflow)?
                    / stakes_sum
            }
            Self::Fixed {
                required_amount,
                required_period,
                reward_amount,
            } => {
                if staked_amount < required_amount {
                    return Ok(0);
                }

                let rewards_count = (end_ts - start_ts) / required_period;
                *last_reward_ts += rewards_count * required_period;

                let edge = if current_ts >= config_end_ts {
                    let part = config_end_ts - *last_reward_ts;
                    *last_reward_ts = config_end_ts;
                    reward_amount
                        .checked_mul(part as u64)
                        .ok_or(StakingError::Overflow)?
                        .checked_div(required_period as u64)
                        .ok_or(StakingError::Overflow)?
                } else {
                    0
                };

                reward_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_add(edge)
                    .ok_or(StakingError::Overflow)?
            }
        };

        Ok(reward_amount)
    }
}

impl Default for RewardType {
    fn default() -> Self {
        Self::Fixed {
            required_amount: 1,
            required_period: 1,
            reward_amount: 1,
        }
    }
}

pub fn calculate_rewards(
    staking: &Account<Staking>,
    config_history: &Account<ConfigHistory>,
    member: &mut Account<Member>,
    stake: &Account<TokenAccount>,
    current_ts: u32,
) -> Result<u64> {
    let mut res = 0u64;

    for i in 0..config_history.len {
        let reward_amount = (config_history.reward_types[i as usize]).get_reward_amount(
            stake.amount,
            staking.stakes_sum,
            &mut member.last_reward_ts,
            current_ts,
            config_history.start_timestamps[i as usize],
            if i + 1 == config_history.len {
                u32::MAX
            } else {
                config_history.start_timestamps[(i + 1) as usize]
            },
        )?;
        res = res
            .checked_add(reward_amount)
            .ok_or(StakingError::Overflow)?;
    }

    Ok(res)
}
