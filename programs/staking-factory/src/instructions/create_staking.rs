use crate::{event::*, reward::RewardParams, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct CreateStaking<'info> {
    #[account(mut, seeds = [b"factory"], bump = factory.bump)]
    pub factory: Account<'info, Factory>,
    #[account(
        init,
        payer = authority,
        seeds = [b"staking", factory.stakings_count.to_le_bytes().as_ref()],
        bump,
        space = 8 + Staking::LEN,
    )]
    pub staking: Account<'info, Staking>,
    #[account(
        init,
        payer = authority,
        seeds = [b"config_history", staking.key().as_ref()],
        bump,
        space = 8 + ConfigHistory::LEN,
   )]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(
        init,
        payer = authority,
        seeds = [b"stakes_history", staking.key().as_ref()],
        bump,
        space = 8 + StakesHistory::LEN,
    )]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn create_staking(
    ctx: Context<CreateStaking>,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
    unstake_timelock: u32,
    reward_params: RewardParams,
) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    reward_params.validate_fields()?;

    ctx.accounts.staking.bump = *ctx.bumps.get("staking").unwrap();
    ctx.accounts.staking.authority = ctx.accounts.authority.key();
    ctx.accounts.staking.id = ctx.accounts.factory.stakings_count;
    ctx.accounts.staking.stake_mint = stake_mint;
    ctx.accounts.staking.reward_mint = reward_mint;
    ctx.accounts.staking.unstake_timelock = unstake_timelock;
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
