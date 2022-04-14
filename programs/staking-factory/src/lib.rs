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
    use std::convert::TryFrom;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let factory = &mut ctx.accounts.factory;

        factory.bump = *ctx.bumps.get("factory").unwrap();
        factory.authority = ctx.accounts.authority.key();

        Ok(())
    }

    pub fn create_staking(
        ctx: Context<CreateStaking>,
        nonce: u8,
        mint: Pubkey,
        withdrawal_timelock: i64,
        reward_period: i64,
        reward_type: u8,
        reward_amount: u64,
    ) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let factory = &mut ctx.accounts.factory;

        staking.bump = nonce;
        staking.authority = ctx.accounts.authority.key();
        staking.factory = factory.key();
        staking.mint = mint;
        staking.withdrawal_timelock = withdrawal_timelock;
        staking.reward_vault = ctx.accounts.reward_vault.key();
        if RewardType::try_from(reward_type).is_err() {
            return err!(StakingError::InvalidType);
        }
        staking.reward_type = reward_type;
        staking.reward_amount = reward_amount;
        staking.reward_period = reward_period;

        factory.stakings_count += 1;

        Ok(())
    }

    pub fn change_config(
        ctx: Context<ChangeConfig>,
        reward_amount: Option<u64>,
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

    pub fn create_member(ctx: Context<CreateMember>, nonce: u8) -> Result<()> {
        let member = &mut ctx.accounts.member;
        member.staking = ctx.accounts.staking.key();
        member.beneficiary = *ctx.accounts.beneficiary.key;
        member.balances = Balances {
            available: ctx.accounts.available.key(),
            stake: ctx.accounts.stake.key(),
            pending: ctx.accounts.pending.key(),
        };
        member.nonce = nonce;

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
            ctx.accounts.staking.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[ctx.accounts.member.nonce],
        ];
        let member_signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.available.to_account_info(),
                to: ctx.accounts.stake.to_account_info(),
                authority: ctx.accounts.member_signer.to_account_info(),
            },
            member_signer,
        );
        token::transfer(cpi_ctx, amount)?;

        ctx.accounts.staking.stakes_sum += amount;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        if ts - ctx.accounts.member.last_reward_ts < ctx.accounts.staking.reward_period {
            return err!(StakingError::ClaimTimelock);
        }

        let reward_amount = match RewardType::try_from(ctx.accounts.staking.reward_type).unwrap() {
            RewardType::Absolute => {
                ctx.accounts.stake.amount * ctx.accounts.staking.reward_amount / 100
            }
            RewardType::Relative => {
                ctx.accounts.stake.amount * ctx.accounts.staking.reward_amount
                    / ctx.accounts.staking.stakes_sum
            }
        };

        let seeds = &[
            ctx.accounts.staking.to_account_info().key.as_ref(),
            &[ctx.accounts.staking.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.staking_signer.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, reward_amount)
    }

    pub fn start_unstake(ctx: Context<StartUnstake>, amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let seeds = &[
            ctx.accounts.staking.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[ctx.accounts.member.nonce],
        ];
        let member_signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.stake.to_account_info(),
                to: ctx.accounts.pending.to_account_info(),
                authority: ctx.accounts.member_signer.to_account_info(),
            },
            member_signer,
        );
        token::transfer(cpi_ctx, amount)?;

        let pending_withdrawal = &mut ctx.accounts.pending_withdrawal;
        pending_withdrawal.burned = false;
        pending_withdrawal.member = *ctx.accounts.member.to_account_info().key;
        pending_withdrawal.start_ts = ts;
        pending_withdrawal.end_ts = ts + ctx.accounts.staking.withdrawal_timelock;
        pending_withdrawal.amount = amount;
        pending_withdrawal.staking = *ctx.accounts.staking.to_account_info().key;

        ctx.accounts.staking.stakes_sum -= amount;

        Ok(())
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        if ctx.accounts.pending_withdrawal.end_ts > ts {
            return err!(StakingError::UnstakeTimelock);
        }

        let seeds = &[
            ctx.accounts.staking.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[ctx.accounts.member.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pending.to_account_info(),
                to: ctx.accounts.available.to_account_info(),
                authority: ctx.accounts.member_signer.to_account_info(),
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
            ctx.accounts.staking.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[ctx.accounts.member.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.available.to_account_info(),
            to: ctx.accounts.receiver.to_account_info(),
            authority: ctx.accounts.member_signer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}
