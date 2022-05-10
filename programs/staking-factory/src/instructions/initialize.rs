use crate::{event::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, seeds = [b"factory"], bump, space = 8 + Factory::LEN)]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.factory.bump = *ctx.bumps.get("factory").unwrap();
    ctx.accounts.factory.authority = ctx.accounts.authority.key();

    emit!(InitializeEvent {});

    Ok(())
}
