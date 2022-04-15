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
#[instruction(nonce: u8)]
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
    pub reward_vault: Account<'info, TokenAccount>,
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
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct CreateMember<'info> {
    pub staking: Box<Account<'info, Staking>>,
    #[account(address = staking.mint)]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump,
        space = 8 + Member::LEN,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"available", member.key().as_ref()],
        bump,
        token::authority = member,
        token::mint = mint,
    )]
    pub available: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"stake", member.key().as_ref()],
        bump,
        token::authority = member,
        token::mint = mint,
    )]
    pub stake: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = beneficiary,
        seeds = [b"pending", member.key().as_ref()],
        bump,
        token::authority = member,
        token::mint = mint,
    )]
    pub pending: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()], bump)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, token::authority = beneficiary)]
    pub depositor: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()], bump)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"stake", member.key().as_ref()], bump)]
    pub stake: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()], bump)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(seeds = [b"stake", member.key().as_ref()], bump)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"reward_vault", staking.key().as_ref()], bump)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut, token::authority = beneficiary, token::mint = reward_vault.mint)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(
        init_if_needed,
        payer = beneficiary,
        seeds = [b"pending_withdrawal", member.key().as_ref()],
        bump,
        space = 8 + PendingWithdrawal::LEN,
    )]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()], bump)]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"stake", member.key().as_ref()], bump)]
    pub stake: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"pending", member.key().as_ref()], bump)]
    pub pending: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()], bump)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pending_withdrawal", member.key().as_ref()],
        bump,
        constraint = !pending_withdrawal.burned
    )]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"pending", member.key().as_ref()], bump)]
    pub pending: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()], bump)]
    pub member: Account<'info, Member>,
    pub beneficiary: Signer<'info>,
    #[account(mut, seeds = [b"available", member.key().as_ref()], bump)]
    pub available: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
