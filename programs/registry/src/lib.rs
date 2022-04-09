use anchor_lang::prelude::*;

use account::*;
use context::*;
use error::*;

pub mod account;
pub mod context;
pub mod error;

declare_id!("Dbh87pAqWbJP44449LJuLy4vX2jwUpJVfTB8BRSzAwjB");

#[program]
pub mod registry {
    use super::*;
    use anchor_spl::token::{self, Transfer};

    pub fn initialize(
        ctx: Context<Initialize>,
        nonce: u8,
        mint: Pubkey,
        authority: Pubkey,
        withdrawal_timelock: i64,
        stake_rate: u64,
        reward_queue_len: u32,
    ) -> Result<()> {
        let registrar = &mut ctx.accounts.registrar;

        registrar.authority = authority;
        registrar.nonce = nonce;
        registrar.mint = mint;
        registrar.pool_mint = ctx.accounts.pool_mint.key();
        registrar.stake_rate = stake_rate;
        registrar.reward_queue = ctx.accounts.reward_queue.key();
        registrar.withdrawal_timelock = withdrawal_timelock;
        registrar.vendor_vault = ctx.accounts.vendor_vault.key();

        let reward_queue = &mut ctx.accounts.reward_queue;
        reward_queue
            .events
            .resize(reward_queue_len as usize, Default::default());

        Ok(())
    }

    pub fn create_member(ctx: Context<CreateMember>, nonce: u8) -> Result<()> {
        let member = &mut ctx.accounts.member;
        member.registrar = *ctx.accounts.registrar.to_account_info().key;
        member.beneficiary = *ctx.accounts.beneficiary.key;
        member.balances = BalanceSandbox {
            spt: ctx.accounts.spt.key(),
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
        token::transfer(cpi_ctx, amount).map_err(Into::into)
    }

    pub fn stake(ctx: Context<Stake>, spt_amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let seeds = &[
            ctx.accounts.registrar.to_account_info().key.as_ref(),
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
        let token_amount = spt_amount
            .checked_mul(ctx.accounts.registrar.stake_rate)
            .unwrap();
        token::transfer(cpi_ctx, token_amount)?;

        let seeds = &[
            ctx.accounts.registrar.to_account_info().key.as_ref(),
            &[ctx.accounts.registrar.nonce],
        ];
        let registrar_signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.pool_mint.to_account_info(),
                to: ctx.accounts.spt.to_account_info(),
                authority: ctx.accounts.registrar_signer.to_account_info(),
            },
            registrar_signer,
        );
        token::mint_to(cpi_ctx, spt_amount)?;

        let member = &mut ctx.accounts.member;
        member.last_stake_ts = ts;

        Ok(())
    }

    pub fn drop_reward(
        ctx: Context<DropReward>,
        fixed: u64,
        per_spt: f64,
        expiry_ts: i64,
    ) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let per_spt_total = per_spt * ctx.accounts.pool_mint.supply as f64;
        let per_spt_total = if per_spt_total > u64::MAX as f64 {
            return err!(RegistryError::RewardTooHigh);
        } else {
            per_spt_total as u64
        };

        let total = fixed
            .checked_add(per_spt_total)
            .ok_or(RegistryError::RewardTooHigh)?;

        if total < ctx.accounts.pool_mint.supply {
            return err!(RegistryError::InsufficientReward);
        }
        if ts >= expiry_ts {
            return err!(RegistryError::InvalidExpiry);
        }

        let reward_queue = &mut ctx.accounts.reward_queue;
        let cursor = reward_queue.append(Reward {
            vendor: *ctx.accounts.vendor.to_account_info().key,
            ts,
        })?;

        let vendor = &mut ctx.accounts.vendor;
        vendor.registrar = *ctx.accounts.registrar.to_account_info().key;
        vendor.mint = ctx.accounts.vendor_vault.mint;
        vendor.pool_token_supply = ctx.accounts.pool_mint.supply;
        vendor.reward_event_q_cursor = cursor;
        vendor.start_ts = ts;
        vendor.expiry_ts = expiry_ts;
        vendor.total = total;
        vendor.expired = false;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let spt_amount = ctx.accounts.spt.amount;
        let reward_amount = spt_amount
            .checked_mul(ctx.accounts.vendor.total)
            .unwrap()
            .checked_div(ctx.accounts.vendor.pool_token_supply)
            .unwrap();
        assert!(reward_amount > 0);

        let seeds = &[
            ctx.accounts.registrar.to_account_info().key.as_ref(),
            &[ctx.accounts.registrar.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vendor_vault.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.registrar_signer.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, reward_amount)?;

        let member = &mut ctx.accounts.member;
        member.rewards_cursor = ctx.accounts.vendor.reward_event_q_cursor + 1;

        Ok(())
    }

    pub fn start_unstake(ctx: Context<StartUnstake>, spt_amount: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let seeds = &[
            ctx.accounts.registrar.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[ctx.accounts.member.nonce],
        ];
        let member_signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.pool_mint.to_account_info(),
                to: ctx.accounts.spt.to_account_info(),
                authority: ctx.accounts.member_signer.to_account_info(),
            },
            member_signer,
        );
        token::burn(cpi_ctx, spt_amount)?;

        let token_amount = spt_amount
            .checked_mul(ctx.accounts.registrar.stake_rate)
            .unwrap();

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.stake.to_account_info(),
                to: ctx.accounts.pending.to_account_info(),
                authority: ctx.accounts.member_signer.to_account_info(),
            },
            member_signer,
        );
        token::transfer(cpi_ctx, token_amount)?;

        let pending_withdrawal = &mut ctx.accounts.pending_withdrawal;
        pending_withdrawal.burned = false;
        pending_withdrawal.member = *ctx.accounts.member.to_account_info().key;
        pending_withdrawal.start_ts = ts;
        pending_withdrawal.end_ts = ts + ctx.accounts.registrar.withdrawal_timelock;
        pending_withdrawal.amount = token_amount;
        pending_withdrawal.pool = ctx.accounts.registrar.pool_mint;
        pending_withdrawal.registrar = *ctx.accounts.registrar.to_account_info().key;

        let member = &mut ctx.accounts.member;
        member.last_stake_ts = ts;

        Ok(())
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        if ctx.accounts.pending_withdrawal.end_ts > ts {
            return err!(RegistryError::UnstakeTimelock);
        }

        let seeds = &[
            ctx.accounts.registrar.to_account_info().key.as_ref(),
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
            ctx.accounts.registrar.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[ctx.accounts.member.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.available.to_account_info(),
            to: ctx.accounts.depositor.to_account_info(),
            authority: ctx.accounts.member_signer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        token::transfer(cpi_ctx, amount).map_err(Into::into)
    }
}
