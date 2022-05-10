use crate::{error::*, event::*, reward::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ChangeConfig<'info> {
    #[account(mut, has_one = authority)]
    pub staking: Account<'info, Staking>,
    #[account(mut, seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut, seeds = [b"stakes_history", staking.key().as_ref()], bump = stakes_history.bump)]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn change_config(
    ctx: Context<ChangeConfig>,
    new_reward_params: Option<RewardParams>,
) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    if let Some(new_reward_params) = new_reward_params {
        if std::mem::discriminant(&ctx.accounts.staking.reward_params)
            != std::mem::discriminant(&new_reward_params)
        {
            return err!(StakingError::CannotChangeStakingType);
        }

        new_reward_params.validate_fields()?;

        let len = ctx.accounts.config_history.len as usize;

        if ts < ctx.accounts.config_history.start_timestamps[len - 1] {
            // last config is not yet started, so just change its params
            ctx.accounts.config_history.reward_params[len - 1] = new_reward_params;
        } else {
            let next_start_ts = match ctx.accounts.staking.reward_params {
                RewardParams::Proportional { reward_period, .. } => {
                    let time_from_last_reward = (ts
                        - ctx.accounts.config_history.start_timestamps[len - 1])
                        % reward_period;
                    let next_start_ts = if time_from_last_reward == 0 {
                        ts
                    } else {
                        ts + reward_period - time_from_last_reward
                    };

                    let total_rewards = (next_start_ts
                        - ctx.accounts.config_history.start_timestamps[len - 1])
                        / reward_period;
                    ctx.accounts.stakes_history.offsets[len] =
                        ctx.accounts.stakes_history.offsets[len - 1] + total_rewards as u8;

                    next_start_ts
                }
                _ => ts,
            };

            ctx.accounts.staking.reward_params = new_reward_params;

            ctx.accounts.config_history.reward_params[len] = new_reward_params;
            ctx.accounts.config_history.start_timestamps[len] = next_start_ts;
            ctx.accounts.config_history.len += 1;
        }
    }

    emit!(ChangeConfigEvent {
        id: ctx.accounts.staking.id,
        new_reward_params,
    });

    Ok(())
}
