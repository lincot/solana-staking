use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 2)]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct NewStaking<'info> {
    #[account(init, payer = payer, space = 8 + Staking::LEN)]
    pub staking: Account<'info, Staking>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref()], bump = nonce)]
    pub staking_signer: UncheckedAccount<'info>,
    #[account(token::authority = staking_signer)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChangeConfig<'info> {
    #[account(mut, signer)]
    pub staking: Account<'info, Staking>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct CreateMember<'info> {
    pub staking: Box<Account<'info, Staking>>,
    #[account(address = staking.mint)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = beneficiary, space = 8 + Member::LEN)]
    pub member: Box<Account<'info, Member>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(init, payer = beneficiary, token::authority = member_signer, token::mint = mint)]
    pub available: Account<'info, TokenAccount>,
    #[account(init, payer = beneficiary, token::authority = member_signer, token::mint = mint)]
    pub stake: Account<'info, TokenAccount>,
    #[account(init, payer = beneficiary, token::authority = member_signer, token::mint = mint)]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref(), member.key().as_ref()], bump = nonce)]
    pub member_signer: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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
    pub staking: Account<'info, Staking>,
    #[account(has_one = beneficiary, has_one = staking)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, address = member.balances.stake)]
    pub stake: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
    pub member_signer: UncheckedAccount<'info>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref()], bump = staking.nonce)]
    pub staking_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(has_one = reward_vault)]
    pub staking: Account<'info, Staking>,
    #[account(mut, has_one = staking, has_one = beneficiary)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(address = member.balances.stake)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut, token::authority = beneficiary, token::mint = reward_vault.mint)]
    pub to: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref()], bump = staking.nonce)]
    pub staking_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(init, payer = beneficiary, space = 8 + PendingWithdrawal::LEN)]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(has_one = beneficiary, has_one = staking)]
    pub member: Box<Account<'info, Member>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.stake)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut, address = member.balances.pending)]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
    pub member_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(has_one = staking, has_one = beneficiary)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, has_one = staking, has_one = member, constraint = !pending_withdrawal.burned)]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, address = member.balances.pending)]
    pub pending: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
    pub member_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub staking: Account<'info, Staking>,
    #[account(has_one = staking, has_one = beneficiary)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.available)]
    pub available: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(seeds = [staking.key().as_ref(), member.key().as_ref()], bump = member.nonce)]
    pub member_signer: UncheckedAccount<'info>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
