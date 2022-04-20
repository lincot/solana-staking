use anchor_lang::prelude::*;

use account::*;
use context::*;
use error::*;

pub mod account;
pub mod context;
pub mod error;

declare_id!("74Gn5o8MXGWuNgApSz7kkfcdWHGpVAcrgs41ZfW1bHbK");

#[program]
pub mod staking_factory {
    use super::*;
    use anchor_spl::token::{self, Transfer};

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let factory = &mut ctx.accounts.factory;

        factory.bump = *ctx.bumps.get("factory").unwrap();
        factory.authority = ctx.accounts.authority.key();

        Ok(())
    }

    pub fn create_staking(
        ctx: Context<CreateStaking>,
        mint: Pubkey,
        withdrawal_timelock: i64,
        reward_period: i64,
        reward_amount: RewardAmount,
    ) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let factory = &mut ctx.accounts.factory;

        staking.bump = *ctx.bumps.get("staking").unwrap();
        staking.bump_vault = *ctx.bumps.get("reward_vault").unwrap();
        staking.authority = ctx.accounts.authority.key();
        staking.id = factory.stakings_count;
        staking.mint = mint;
        staking.withdrawal_timelock = withdrawal_timelock;
        staking.reward_amount = reward_amount;
        staking.reward_period = reward_period;

        factory.stakings_count += 1;

        Ok(())
    }

    pub fn change_config(
        ctx: Context<ChangeConfig>,
        reward_amount: Option<RewardAmount>,
        reward_period: Option<i64>,
    ) -> Result<()> {
        let staking = &mut ctx.accounts.staking;

        if let Some(reward_amount) = reward_amount {
            staking.reward_amount = reward_amount;
        }
        if let Some(reward_period) = reward_period {
            staking.reward_period = reward_period;
        }

        Ok(())
    }

    pub fn create_member(ctx: Context<CreateMember>) -> Result<()> {
        let member = &mut ctx.accounts.member;

        member.bump = *ctx.bumps.get("member").unwrap();
        member.bump_available = *ctx.bumps.get("available").unwrap();
        member.bump_stake = *ctx.bumps.get("stake").unwrap();
        member.bump_pending = *ctx.bumps.get("pending").unwrap();

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.available.to_account_info(),
                authority: ctx.accounts.beneficiary.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let seeds = &[
            b"member".as_ref(),
            &ctx.accounts.staking.id.to_le_bytes(),
            ctx.accounts.beneficiary.to_account_info().key.as_ref(),
            &[ctx.accounts.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.available.to_account_info(),
                to: ctx.accounts.stake.to_account_info(),
                authority: ctx.accounts.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)?;

        let staking = &mut ctx.accounts.staking;

        staking.stakes_sum = staking
            .stakes_sum
            .checked_add(amount)
            .ok_or(StakingError::Overflow)?;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        if ts - ctx.accounts.member.last_reward_ts < ctx.accounts.staking.reward_period {
            return err!(StakingError::ClaimTimelock);
        }

        let staked_amount = ctx.accounts.stake.amount;

        if staked_amount == 0 {
            return err!(StakingError::NothingToClaim);
        }

        let reward_amount = match ctx.accounts.staking.reward_amount {
            RewardAmount::Absolute { num, denom } => {
                staked_amount
                    .checked_mul(num)
                    .ok_or(StakingError::Overflow)?
                    / denom
            }
            RewardAmount::Relative { total_amount } => {
                staked_amount
                    .checked_mul(total_amount)
                    .ok_or(StakingError::Overflow)?
                    / ctx.accounts.staking.stakes_sum
            }
        };

        let seeds = &[
            b"staking".as_ref(),
            &ctx.accounts.staking.id.to_le_bytes(),
            &[ctx.accounts.staking.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.staking.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, reward_amount)
    }

    pub fn start_unstake(ctx: Context<StartUnstake>, amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let seeds = &[
            b"member".as_ref(),
            &ctx.accounts.staking.id.to_le_bytes(),
            ctx.accounts.beneficiary.to_account_info().key.as_ref(),
            &[ctx.accounts.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.stake.to_account_info(),
                to: ctx.accounts.pending.to_account_info(),
                authority: ctx.accounts.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)?;

        let pending_withdrawal = &mut ctx.accounts.pending_withdrawal;
        pending_withdrawal.bump = *ctx.bumps.get("pending_withdrawal").unwrap();
        pending_withdrawal.burned = false;
        pending_withdrawal.start_ts = ts;
        pending_withdrawal.end_ts = ts + ctx.accounts.staking.withdrawal_timelock;
        pending_withdrawal.amount = amount;

        ctx.accounts.staking.stakes_sum -= amount;

        Ok(())
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        if ctx.accounts.pending_withdrawal.end_ts > ts {
            return err!(StakingError::UnstakeTimelock);
        }

        let seeds = &[
            b"member".as_ref(),
            &ctx.accounts.staking.id.to_le_bytes(),
            ctx.accounts.beneficiary.to_account_info().key.as_ref(),
            &[ctx.accounts.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pending.to_account_info(),
                to: ctx.accounts.available.to_account_info(),
                authority: ctx.accounts.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, ctx.accounts.pending_withdrawal.amount)?;

        let pending_withdrawal = &mut ctx.accounts.pending_withdrawal;
        pending_withdrawal.burned = true;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let seeds = &[
            b"member".as_ref(),
            &ctx.accounts.staking.id.to_le_bytes(),
            ctx.accounts.beneficiary.to_account_info().key.as_ref(),
            &[ctx.accounts.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.available.to_account_info(),
            to: ctx.accounts.receiver.to_account_info(),
            authority: ctx.accounts.member.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}
