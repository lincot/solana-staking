use crate::account::*;
use crate::error::*;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug)]
pub enum RewardParams {
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
impl Default for RewardParams {
    fn default() -> Self {
        Self::Fixed {
            required_amount: 0,
            required_period: 0,
            reward_amount: 0,
        }
    }
}
impl RewardParams {
    pub const LEN: usize = 1 + 8 + 4 + 8;

    pub fn validate_fields(&self) -> Result<()> {
        match self {
            Self::InterestRate { denom: 0, .. } => err!(StakingError::Zero),
            Self::Proportional {
                reward_period: 0, ..
            } => err!(StakingError::Zero),
            Self::Fixed {
                required_period: 0, ..
            } => err!(StakingError::Zero),
            _ => Ok(()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn get_reward_amount(
        &self,
        staked_amount: u64,
        current_stakes_sum: u64,
        last_reward_ts: &mut u32,
        current_ts: u32,
        config_start_ts: u32,
        config_end_ts: u32,
        stakes_history: &mut Account<StakesHistory>,
        offset: u8,
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
                let claimed_rewards_count = (start_ts - config_start_ts) / reward_period;
                let all_rewards_count = (end_ts - config_start_ts) / reward_period;
                let rewards_count = all_rewards_count - claimed_rewards_count;
                *last_reward_ts += rewards_count * reward_period;

                let mut reward_amount = 0u64;

                for i in offset as u32 + claimed_rewards_count..offset as u32 + all_rewards_count {
                    if (stakes_history.len as u32) <= i {
                        // no one has checked this reward yet so its stakes_sum becomes current
                        stakes_history.stakes_sums[i as usize] = current_stakes_sum;
                        stakes_history.len += 1;
                    }

                    reward_amount = reward_amount
                        .checked_add(
                            staked_amount
                                .checked_mul(total_amount)
                                .ok_or(StakingError::Overflow)?
                                .checked_div(stakes_history.stakes_sums[i as usize])
                                .unwrap_or(0),
                        )
                        .ok_or(StakingError::Overflow)?;
                }

                reward_amount
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

                let partial_reward = if current_ts >= config_end_ts {
                    let partial_period = config_end_ts - *last_reward_ts;
                    *last_reward_ts = config_end_ts;
                    reward_amount
                        .checked_mul(partial_period as u64)
                        .ok_or(StakingError::Overflow)?
                        / required_period as u64
                } else {
                    0
                };

                reward_amount
                    .checked_mul(rewards_count as u64)
                    .ok_or(StakingError::Overflow)?
                    .checked_add(partial_reward)
                    .ok_or(StakingError::Overflow)?
            }
        };

        Ok(reward_amount)
    }
}

pub fn calculate_rewards<'info>(
    current_ts: u32,
    staking: &Account<'info, Staking>,
    config_history: &Account<'info, ConfigHistory>,
    member: &mut Account<'info, Member>,
    stakes_history: &mut Account<'info, StakesHistory>,
) -> Result<u64> {
    let mut res = 0u64;

    for i in 0..config_history.len {
        let offset = stakes_history.offsets[i as usize];
        let reward_amount = {
            (config_history.reward_params[i as usize]).get_reward_amount(
                member.stake_amount,
                staking.stakes_sum,
                &mut member.last_reward_ts,
                current_ts,
                config_history.start_timestamps[i as usize],
                if i + 1 == config_history.len {
                    u32::MAX
                } else {
                    config_history.start_timestamps[(i + 1) as usize]
                },
                stakes_history,
                offset,
            )?
        };
        res = res
            .checked_add(reward_amount)
            .ok_or(StakingError::Overflow)?;
    }

    Ok(res)
}
