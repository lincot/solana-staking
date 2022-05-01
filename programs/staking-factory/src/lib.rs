use anchor_lang::prelude::*;

use context_admin::*;
use context_user::*;
use error::*;
use event::*;
use reward::*;

pub mod account;
pub mod context_admin;
pub mod context_user;
pub mod error;
pub mod event;
pub mod reward;

declare_id!("74Gn5o8MXGWuNgApSz7kkfcdWHGpVAcrgs41ZfW1bHbK");

const FACTORY_FEE_NUM: u64 = 3;
const FACTORY_FEE_DENOM: u64 = 100;

#[program]
pub mod staking_factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.factory.bump = *ctx.bumps.get("factory").unwrap();
        ctx.accounts.factory.authority = ctx.accounts.authority.key();

        emit!(InitializeEvent {});

        Ok(())
    }

    pub fn create_staking(
        ctx: Context<CreateStaking>,
        stake_mint: Pubkey,
        withdrawal_timelock: u32,
        reward_params: RewardParams,
    ) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        reward_params.validate_fields()?;

        ctx.accounts.staking.bump = *ctx.bumps.get("staking").unwrap();
        ctx.accounts.staking.bump_vault = *ctx.bumps.get("reward_vault").unwrap();
        ctx.accounts.staking.authority = ctx.accounts.authority.key();
        ctx.accounts.staking.id = ctx.accounts.factory.stakings_count;
        ctx.accounts.staking.stake_mint = stake_mint;
        ctx.accounts.staking.reward_mint = ctx.accounts.reward_mint.key();
        ctx.accounts.staking.withdrawal_timelock = withdrawal_timelock;
        ctx.accounts.staking.reward_params = reward_params;

        ctx.accounts.config_history.bump = *ctx.bumps.get("config_history").unwrap();
        ctx.accounts.config_history.len = 1;
        ctx.accounts.config_history.reward_params[0] = reward_params;
        ctx.accounts.config_history.start_timestamps[0] = ts;

        ctx.accounts.stakes_history.bump = *ctx.bumps.get("stakes_history").unwrap();

        ctx.accounts.factory.stakings_count += 1;

        emit!(CreateStakingEvent {
            id: ctx.accounts.staking.id,
            authority: ctx.accounts.staking.authority,
            reward_params,
        });

        Ok(())
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

            if let RewardParams::Proportional { reward_period, .. } =
                ctx.accounts.staking.reward_params
            {
                let len = ctx.accounts.config_history.len as usize;
                let rewards_dropped =
                    (ts - ctx.accounts.config_history.start_timestamps[len - 1]) / reward_period;
                ctx.accounts.stakes_history.offsets[len] =
                    ctx.accounts.stakes_history.offsets[len - 1] + rewards_dropped as u8;
            }

            ctx.accounts.staking.reward_params = new_reward_params;

            let len = ctx.accounts.config_history.len as usize;
            ctx.accounts.config_history.reward_params[len] = new_reward_params;
            ctx.accounts.config_history.start_timestamps[len] = ts;
            ctx.accounts.config_history.len += 1;
        }

        emit!(ChangeConfigEvent {
            id: ctx.accounts.staking.id,
            new_reward_params,
        });

        Ok(())
    }

    pub fn register_member(ctx: Context<RegisterMember>) -> Result<()> {
        ctx.accounts.member.bump = *ctx.bumps.get("member").unwrap();
        ctx.accounts.member.bump_available = *ctx.bumps.get("available").unwrap();
        ctx.accounts.member.bump_stake = *ctx.bumps.get("stake").unwrap();
        ctx.accounts.member.bump_pending = *ctx.bumps.get("pending").unwrap();

        ctx.accounts.pending_withdrawal.bump = *ctx.bumps.get("pending_withdrawal").unwrap();

        emit!(RegisterMemberEvent {
            beneficiary: ctx.accounts.beneficiary.key()
        });

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.transfer(amount)?;

        emit!(DepositEvent {
            beneficiary: ctx.accounts.beneficiary.key(),
            amount,
        });

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        let rewards = calculate_rewards(
            ts,
            &ctx.accounts.staking,
            &ctx.accounts.config_history,
            &mut ctx.accounts.member,
            &ctx.accounts.stake,
            &mut ctx.accounts.stakes_history,
        )?;
        ctx.accounts.member.unclaimed_rewards = (ctx.accounts.member.unclaimed_rewards)
            .checked_add(rewards)
            .ok_or(StakingError::Overflow)?;

        ctx.accounts.transfer(amount)?;

        ctx.accounts.staking.stakes_sum = (ctx.accounts.staking.stakes_sum)
            .checked_add(amount)
            .ok_or(StakingError::Overflow)?;

        emit!(StakeEvent {
            beneficiary: ctx.accounts.beneficiary.key(),
            amount,
        });

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        let rewards = calculate_rewards(
            ts,
            &ctx.accounts.staking,
            &ctx.accounts.config_history,
            &mut ctx.accounts.member,
            &ctx.accounts.stake,
            &mut ctx.accounts.stakes_history,
        )?;

        let total_amount = rewards
            .checked_add(ctx.accounts.member.unclaimed_rewards)
            .ok_or(StakingError::Overflow)?;
        if total_amount == 0 {
            return err!(StakingError::NothingToClaim);
        }

        let factory_fee = total_amount
            .checked_mul(FACTORY_FEE_NUM)
            .ok_or(StakingError::Overflow)?
            / FACTORY_FEE_DENOM;
        let amount_to_user = total_amount - factory_fee;

        ctx.accounts.transfer_to_user(amount_to_user)?;
        ctx.accounts.transfer_to_factory_owner(factory_fee)?;

        ctx.accounts.member.unclaimed_rewards = 0;

        emit!(ClaimRewardEvent {
            beneficiary: ctx.accounts.beneficiary.key(),
            total_amount,
            amount_to_user,
            factory_fee,
        });

        Ok(())
    }

    pub fn start_unstake(ctx: Context<StartUnstake>, amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        let rewards = calculate_rewards(
            ts,
            &ctx.accounts.staking,
            &ctx.accounts.config_history,
            &mut ctx.accounts.member,
            &ctx.accounts.stake,
            &mut ctx.accounts.stakes_history,
        )?;
        ctx.accounts.member.unclaimed_rewards = (ctx.accounts.member.unclaimed_rewards)
            .checked_add(rewards)
            .ok_or(StakingError::Overflow)?;

        ctx.accounts.transfer(amount)?;

        ctx.accounts.pending_withdrawal.active = true;
        ctx.accounts.pending_withdrawal.end_ts = ts + ctx.accounts.staking.withdrawal_timelock;
        ctx.accounts.pending_withdrawal.amount = amount;

        ctx.accounts.staking.stakes_sum -= amount;

        emit!(StartUnstakeEvent {
            beneficiary: ctx.accounts.beneficiary.key(),
            amount,
        });

        Ok(())
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        if ctx.accounts.pending_withdrawal.end_ts > ts {
            return err!(StakingError::UnstakeTimelock);
        }

        ctx.accounts.transfer()?;

        ctx.accounts.pending_withdrawal.active = false;

        emit!(EndUnstakeEvent {
            beneficiary: ctx.accounts.beneficiary.key(),
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.transfer(amount)?;

        emit!(WithdrawEvent {
            beneficiary: ctx.accounts.beneficiary.key(),
            amount,
        });

        Ok(())
    }
}
