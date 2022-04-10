use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub registrar: Account<'info, Registrar>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref()], bump = nonce)]
    pub registrar_signer: UncheckedAccount<'info>,
    #[account(constraint = pool_mint.decimals == 0)]
    pub pool_mint: Account<'info, Mint>,
    #[account(constraint = vendor_vault.owner == registrar_signer.key())]
    pub vendor_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct CreateMember<'info> {
    pub registrar: Box<Account<'info, Registrar>>,
    #[account(zero)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        constraint = available.owner == member_signer.key(),
        constraint = available.mint == registrar.mint
    )]
    pub available: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = stake.owner == member_signer.key(),
        constraint = stake.mint == registrar.mint 
    )]
    pub stake: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = pending.owner == member_signer.key(),
        constraint = pending.mint == registrar.mint
    )]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref(), member.key().as_ref()], bump)]
    pub member_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(has_one = beneficiary)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, constraint = depositor.owner == beneficiary.key())]
    pub depositor: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(has_one = pool_mint)]
    pub registrar: Account<'info, Registrar>,
    #[account(mut)]
    pub pool_mint: Box<Account<'info, Mint>>,
    #[account(mut, has_one = beneficiary, has_one = registrar)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, address = member.balances.stake)]
    pub stake: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
    pub member_signer: UncheckedAccount<'info>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref()], bump = registrar.nonce)]
    pub registrar_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(has_one = vendor_vault)]
    pub registrar: Account<'info, Registrar>,
    #[account(mut, has_one = registrar, has_one = beneficiary)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut)]
    pub vendor_vault: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(mut)]
    pub to: UncheckedAccount<'info>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref()], bump = registrar.nonce)]
    pub registrar_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    pub registrar: Box<Account<'info, Registrar>>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(zero)]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(has_one = beneficiary, has_one = registrar)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.stake)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut, address = member.balances.pending)]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(
        seeds = [registrar.key().as_ref(), member.key().as_ref()],
        bump = member.nonce,
    )]
    pub member_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EndUnstake<'info> {
    pub registrar: Account<'info, Registrar>,
    #[account(has_one = registrar, has_one = beneficiary)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, has_one = registrar, has_one = member, constraint = !pending_withdrawal.burned)]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, address = member.balances.pending)]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(
        seeds = [registrar.key().as_ref(), member.key().as_ref()],
        bump = member.nonce,
    )]
    pub member_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub registrar: Account<'info, Registrar>,
    #[account(has_one = registrar, has_one = beneficiary)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(
        seeds = [registrar.key().as_ref(), member.key().as_ref()],
        bump = member.nonce,
    )]
    pub member_signer: UncheckedAccount<'info>,
    #[account(mut)]
    pub depositor: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
