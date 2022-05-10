use crate::{event::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterMember<'info> {
    pub staking: Account<'info, Staking>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump,
        space = 8 + Member::LEN,
    )]
    pub member: Account<'info, Member>,
    pub system_program: Program<'info, System>,
}

pub fn register_member(ctx: Context<RegisterMember>) -> Result<()> {
    ctx.accounts.member.bump = *ctx.bumps.get("member").unwrap();

    emit!(RegisterMemberEvent {
        beneficiary: ctx.accounts.beneficiary.key()
    });

    Ok(())
}
