use crate::{event::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub staking: Account<'info, Staking>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut, associated_token::authority = member, associated_token::mint = staking.stake_mint)]
    pub member_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

fn transfer_to_member_vault(ctx: &Context<Deposit>, amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.member_vault.to_account_info(),
            authority: ctx.accounts.beneficiary.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount)
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    transfer_to_member_vault(&ctx, amount)?;

    ctx.accounts.member.available_amount += amount;

    emit!(DepositEvent {
        beneficiary: ctx.accounts.beneficiary.key(),
        amount,
    });

    Ok(())
}
