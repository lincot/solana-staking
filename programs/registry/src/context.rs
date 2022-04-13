use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub registrar: Account<'info, Registrar>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref()], bump = nonce)]
    pub registrar_signer: UncheckedAccount<'info>,
    #[account(token::authority = registrar_signer)]
    pub reward_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct ChangeConfig<'info> {
    #[account(mut, signer)]
    pub registrar: Account<'info, Registrar>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct CreateMember<'info> {
    pub registrar: Account<'info, Registrar>,
    #[account(zero)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(token::authority = member_signer, token::mint = registrar.mint)]
    pub available: Account<'info, TokenAccount>,
    #[account(token::authority = member_signer, token::mint = registrar.mint)]
    pub stake: Account<'info, TokenAccount>,
    #[account(token::authority = member_signer, token::mint = registrar.mint)]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref(), member.key().as_ref()], bump = nonce)]
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
    #[account(mut, token::authority = beneficiary)]
    pub depositor: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub registrar: Account<'info, Registrar>,
    #[account(has_one = beneficiary, has_one = registrar)]
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
    #[account(has_one = reward_vault)]
    pub registrar: Account<'info, Registrar>,
    #[account(mut, has_one = registrar, has_one = beneficiary)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(address = member.balances.stake)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut, token::authority = beneficiary, token::mint = reward_vault.mint)]
    pub to: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [registrar.key().as_ref()], bump = registrar.nonce)]
    pub registrar_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    pub registrar: Account<'info, Registrar>,
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
    #[account(seeds = [registrar.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
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
    #[account(seeds = [registrar.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
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
    #[account(seeds = [registrar.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
    pub member_signer: UncheckedAccount<'info>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
