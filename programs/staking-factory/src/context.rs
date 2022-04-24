use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, seeds = [b"factory"], bump, space = 8 + Factory::LEN)]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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
    pub reward_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        seeds = [b"reward_vault", staking.key().as_ref()],
        bump,
        token::authority = staking,
        token::mint = reward_mint,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        seeds = [b"config_history", staking.key().as_ref()],
        bump,
        space = 8 + ConfigHistory::LEN,
   )]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChangeConfig<'info> {
    #[account(mut, has_one = authority)]
    pub staking: Account<'info, Staking>,
    #[account(mut, seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterMember<'info> {
    pub staking: Box<Account<'info, Staking>>,
    #[account(address = staking.stake_mint)]
    pub stake_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump,
        space = 8 + Member::LEN,
    )]
    pub member: Account<'info, Member>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"pending_withdrawal", member.key().as_ref()],
        bump,
        space = 8 + PendingWithdrawal::LEN,
    )]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"available", member.key().as_ref()],
        bump,
        token::authority = member,
        token::mint = stake_mint,
    )]
    pub available: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"stake", member.key().as_ref()],
        bump,
        token::authority = member,
        token::mint = stake_mint,
    )]
    pub stake: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"pending", member.key().as_ref()],
        bump,
        token::authority = member,
        token::mint = stake_mint,
    )]
    pub pending: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub staking: Account<'info, Staking>,
    #[account(
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump = member.bump_available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, token::authority = beneficiary)]
    pub depositor: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump = member.bump_available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"stake", member.key().as_ref()], bump = member.bump_stake)]
    pub stake: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(seeds = [b"factory"], bump = factory.bump)]
    pub factory: Box<Account<'info, Factory>>,
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(seeds = [b"stake", member.key().as_ref()], bump = member.bump_stake)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"reward_vault", staking.key().as_ref()], bump = staking.bump_vault)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    #[account(mut, token::authority = factory.authority, token::mint = reward_vault.mint)]
    pub factory_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    #[account(
        mut,
        seeds = [b"pending_withdrawal", member.key().as_ref()],
        bump = pending_withdrawal.bump,
        constraint = !pending_withdrawal.active,
    )]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"stake", member.key().as_ref()], bump = member.bump_stake)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"pending", member.key().as_ref()], bump = member.bump_pending)]
    pub pending: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pending_withdrawal", member.key().as_ref()],
        bump = pending_withdrawal.bump,
        constraint = pending_withdrawal.active,
    )]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump = member.bump_available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"pending", member.key().as_ref()], bump = member.bump_pending)]
    pub pending: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub staking: Account<'info, Staking>,
    #[account(
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump = member.bump_available)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
