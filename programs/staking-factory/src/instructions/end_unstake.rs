use crate::{error::*, event::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EndUnstake<'info> {
    pub staking: Account<'info, Staking>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
        constraint = member.pending_unstake_active,
    )]
    pub member: Account<'info, Member>,
}

pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    if ctx.accounts.member.pending_unstake_end_ts > ts {
        return err!(StakingError::UnstakeTimelock);
    }

    ctx.accounts.member.available_amount += ctx.accounts.member.pending_amount;
    ctx.accounts.member.pending_amount = 0;

    ctx.accounts.member.pending_unstake_active = false;

    emit!(EndUnstakeEvent {
        beneficiary: ctx.accounts.beneficiary.key(),
    });

    Ok(())
}
