use crate::{error::*, event::*, reward::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut, seeds = [b"stakes_history", staking.key().as_ref()], bump = stakes_history.bump)]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    if ctx.accounts.member.available_amount < amount {
        return err!(StakingError::InsufficientBalance);
    }

    let rewards = calculate_rewards(
        ts,
        &ctx.accounts.staking,
        &ctx.accounts.config_history,
        &mut ctx.accounts.member,
        &mut ctx.accounts.stakes_history,
    )?;
    ctx.accounts.member.rewards_amount += rewards;

    ctx.accounts.member.available_amount -= amount;
    ctx.accounts.member.stake_amount += amount;
    ctx.accounts.staking.stakes_sum += amount;

    emit!(StakeEvent {
        beneficiary: ctx.accounts.beneficiary.key(),
        amount,
    });

    Ok(())
}
