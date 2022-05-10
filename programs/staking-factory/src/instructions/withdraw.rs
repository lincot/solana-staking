use crate::{error::*, event::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub staking: Account<'info, Staking>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    #[account(mut, associated_token::authority = member, associated_token::mint = staking.stake_mint)]
    pub member_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

fn transfer_from_member_vault(ctx: &Context<Withdraw>, amount: u64) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[
        b"member".as_ref(),
        &ctx.accounts.staking.id.to_le_bytes(),
        ctx.accounts.beneficiary.to_account_info().key.as_ref(),
        &[ctx.accounts.member.bump],
    ]];
    let cpi_accounts = Transfer {
        from: ctx.accounts.member_vault.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: ctx.accounts.member.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    token::transfer(cpi_ctx, amount)
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    if ctx.accounts.member.available_amount < amount {
        return err!(StakingError::InsufficientBalance);
    }

    transfer_from_member_vault(&ctx, amount)?;

    ctx.accounts.member.available_amount -= amount;

    emit!(WithdrawEvent {
        beneficiary: ctx.accounts.beneficiary.key(),
        amount,
    });

    Ok(())
}
