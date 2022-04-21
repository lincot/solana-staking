use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use account::*;
use context::*;
use error::*;

pub mod account;
pub mod context;
pub mod error;

declare_id!("74Gn5o8MXGWuNgApSz7kkfcdWHGpVAcrgs41ZfW1bHbK");

const FACTORY_FEE_NUM: u64 = 3;
const FACTORY_FEE_DENOM: u64 = 100;

#[program]
pub mod staking_factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.factory.bump = *ctx.bumps.get("factory").unwrap();
        ctx.accounts.factory.authority = ctx.accounts.authority.key();

        Ok(())
    }

    pub fn create_staking(
        ctx: Context<CreateStaking>,
        mint: Pubkey,
        withdrawal_timelock: u32,
        reward_amount: RewardAmount,
    ) -> Result<()> {
        ctx.accounts.staking.bump = *ctx.bumps.get("staking").unwrap();
        ctx.accounts.staking.bump_vault = *ctx.bumps.get("reward_vault").unwrap();
        ctx.accounts.staking.authority = ctx.accounts.authority.key();
        ctx.accounts.staking.id = ctx.accounts.factory.stakings_count;
        ctx.accounts.staking.mint = mint;
        ctx.accounts.staking.withdrawal_timelock = withdrawal_timelock;
        ctx.accounts.staking.reward_amount = reward_amount;

        ctx.accounts.factory.stakings_count += 1;

        Ok(())
    }

    pub fn change_config(
        ctx: Context<ChangeConfig>,
        reward_amount: Option<RewardAmount>,
    ) -> Result<()> {
        if let Some(reward_amount) = reward_amount {
            ctx.accounts.staking.reward_amount = reward_amount;
        }

        Ok(())
    }

    pub fn create_member(ctx: Context<CreateMember>) -> Result<()> {
        ctx.accounts.member.bump = *ctx.bumps.get("member").unwrap();
        ctx.accounts.member.bump_available = *ctx.bumps.get("available").unwrap();
        ctx.accounts.member.bump_stake = *ctx.bumps.get("stake").unwrap();
        ctx.accounts.member.bump_pending = *ctx.bumps.get("pending").unwrap();

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.transfer(amount)?;

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        ctx.accounts.member.unclaimed_rewards = (ctx.accounts.member.unclaimed_rewards)
            .checked_add((ctx.accounts.staking.reward_amount).get(
                ctx.accounts.stake.amount,
                ctx.accounts.staking.stakes_sum,
                ts,
                &mut ctx.accounts.member.last_reward_ts,
            )?)
            .ok_or(StakingError::Overflow)?;

        ctx.accounts.transfer(amount)?;

        ctx.accounts.staking.stakes_sum = (ctx.accounts.staking.stakes_sum)
            .checked_add(amount)
            .ok_or(StakingError::Overflow)?;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        let total_amount = ctx.accounts.staking.reward_amount.get(
            ctx.accounts.stake.amount,
            ctx.accounts.staking.stakes_sum,
            ts,
            &mut ctx.accounts.member.last_reward_ts,
        )?;
        let total_amount = total_amount
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

        Ok(())
    }

    pub fn start_unstake(ctx: Context<StartUnstake>, amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        ctx.accounts.member.unclaimed_rewards = (ctx.accounts.member.unclaimed_rewards)
            .checked_add((ctx.accounts.staking.reward_amount).get(
                ctx.accounts.stake.amount,
                ctx.accounts.staking.stakes_sum,
                ts,
                &mut ctx.accounts.member.last_reward_ts,
            )?)
            .ok_or(StakingError::Overflow)?;

        ctx.accounts.transfer(amount)?;

        ctx.accounts.pending_withdrawal.bump = *ctx.bumps.get("pending_withdrawal").unwrap();
        ctx.accounts.pending_withdrawal.burned = false;
        ctx.accounts.pending_withdrawal.start_ts = ts;
        ctx.accounts.pending_withdrawal.end_ts = ts + ctx.accounts.staking.withdrawal_timelock;
        ctx.accounts.pending_withdrawal.amount = amount;

        ctx.accounts.staking.stakes_sum -= amount;

        Ok(())
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        if ctx.accounts.pending_withdrawal.end_ts > ts {
            return err!(StakingError::UnstakeTimelock);
        }

        ctx.accounts.transfer()?;

        ctx.accounts.pending_withdrawal.burned = true;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.transfer(amount)?;

        Ok(())
    }
}

impl<'info> Deposit<'info> {
    fn transfer(&self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.depositor.to_account_info(),
                to: self.available.to_account_info(),
                authority: self.beneficiary.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)
    }
}

impl<'info> Stake<'info> {
    fn transfer(&self, amount: u64) -> Result<()> {
        let seeds = &[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.available.to_account_info(),
                to: self.stake.to_account_info(),
                authority: self.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}

impl<'info> ClaimReward<'info> {
    fn transfer_to_user(&self, amount: u64) -> Result<()> {
        let seeds = &[
            b"staking".as_ref(),
            &self.staking.id.to_le_bytes(),
            &[self.staking.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.reward_vault.to_account_info(),
                to: self.to.to_account_info(),
                authority: self.staking.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }

    fn transfer_to_factory_owner(&self, amount: u64) -> Result<()> {
        let seeds = &[
            b"staking".as_ref(),
            &self.staking.id.to_le_bytes(),
            &[self.staking.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.reward_vault.to_account_info(),
                to: self.factory_vault.to_account_info(),
                authority: self.staking.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}

impl<'info> StartUnstake<'info> {
    fn transfer(&self, amount: u64) -> Result<()> {
        let seeds = &[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.stake.to_account_info(),
                to: self.pending.to_account_info(),
                authority: self.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}

impl<'info> EndUnstake<'info> {
    fn transfer(&self) -> Result<()> {
        let seeds = &[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pending.to_account_info(),
                to: self.available.to_account_info(),
                authority: self.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, self.pending_withdrawal.amount)
    }
}

impl<'info> Withdraw<'info> {
    fn transfer(&self, amount: u64) -> Result<()> {
        let seeds = &[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: self.available.to_account_info(),
            to: self.receiver.to_account_info(),
            authority: self.member.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }
}
