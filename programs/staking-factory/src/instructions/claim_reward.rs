use crate::{event::*, reward::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

const FACTORY_FEE_NUM: u64 = 3;
const FACTORY_FEE_DENOM: u64 = 100;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(seeds = [b"factory"], bump = factory.bump)]
    pub factory: Account<'info, Factory>,
    #[account(mut, token::authority = factory.authority, token::mint = staking.reward_mint)]
    pub factory_vault: Account<'info, TokenAccount>,
    pub staking: Account<'info, Staking>,
    #[account(mut, associated_token::authority = staking, associated_token::mint = staking.reward_mint)]
    pub staking_vault: Account<'info, TokenAccount>,
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
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

fn transfer_to_beneficiary(ctx: &Context<ClaimReward>, amount: u64) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[
        b"staking".as_ref(),
        &ctx.accounts.staking.id.to_le_bytes(),
        &[ctx.accounts.staking.bump],
    ]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.staking.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx, amount)
}

fn transfer_to_factory_owner(ctx: &Context<ClaimReward>, amount: u64) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[
        b"staking".as_ref(),
        &ctx.accounts.staking.id.to_le_bytes(),
        &[ctx.accounts.staking.bump],
    ]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.factory_vault.to_account_info(),
            authority: ctx.accounts.staking.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx, amount)
}

pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    let rewards = calculate_rewards(
        ts,
        &ctx.accounts.staking,
        &ctx.accounts.config_history,
        &mut ctx.accounts.member,
        &mut ctx.accounts.stakes_history,
    )?;
    ctx.accounts.member.rewards_amount += rewards;

    let factory_fee = ctx.accounts.member.rewards_amount * FACTORY_FEE_NUM / FACTORY_FEE_DENOM;
    transfer_to_factory_owner(&ctx, factory_fee)?;

    let amount_to_beneficiary = ctx.accounts.member.rewards_amount - factory_fee;
    transfer_to_beneficiary(&ctx, amount_to_beneficiary)?;

    ctx.accounts.member.rewards_amount = 0;

    emit!(ClaimRewardEvent {
        beneficiary: ctx.accounts.beneficiary.key(),
        amount_to_beneficiary,
        factory_fee,
    });

    Ok(())
}
