use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub registrar: Account<'info, Registrar>,
    #[account(zero)]
    pub reward_queue: Account<'info, RewardQueue>,
    #[account(constraint = pool_mint.decimals == 0)]
    pub pool_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct CreateMember<'info> {
    pub registrar: Account<'info, Registrar>,
    #[account(zero)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        constraint = spt.owner == member_signer.key(),
        constraint = spt.mint == registrar.pool_mint
    )]
    pub spt: Account<'info, TokenAccount>,
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
    #[account(signer, has_one = pool_mint, has_one = reward_queue)]
    pub registrar: Account<'info, Registrar>,
    pub reward_queue: Account<'info, RewardQueue>,
    #[account(mut)]
    pub pool_mint: Box<Account<'info, Mint>>,
    #[account(mut, has_one = beneficiary, has_one = registrar)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.spt)]
    pub spt: Account<'info, TokenAccount>,
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
pub struct DropReward<'info> {
    #[account(has_one = reward_queue, has_one = pool_mint)]
    pub registrar: Account<'info, Registrar>,
    #[account(mut)]
    pub reward_queue: Account<'info, RewardQueue>,
    pub pool_mint: Account<'info, Mint>,
    #[account(zero, signer)]
    pub vendor: Account<'info, RewardVendor>,
    #[account(mut)]
    pub vendor_vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = depositor.owner == depositor_authority.key())]
    pub depositor: Account<'info, TokenAccount>,
    pub depositor_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    pub registrar: Account<'info, Registrar>,
    #[account(mut, has_one = registrar, has_one = beneficiary)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.spt)]
    pub spt: Account<'info, TokenAccount>,
    #[account(has_one = registrar, has_one = vault)]
    pub vendor: Account<'info, RewardVendor>,
    /// CHECK:
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(
        seeds = [registrar.key().as_ref(), vendor.key().as_ref()],
        bump = vendor.nonce,
    )]
    pub vendor_signer: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub to: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    #[account(has_one = reward_queue)]
    pub registrar: Box<Account<'info, Registrar>>,
    pub reward_queue: Account<'info, RewardQueue>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(zero)]
    pub pending_withdrawal: Account<'info, PendingWithdrawal>,
    #[account(has_one = beneficiary, has_one = registrar)]
    pub member: Box<Account<'info, Member>>,
    pub beneficiary: Signer<'info>,
    #[account(mut, address = member.balances.spt)]
    pub spt: Account<'info, TokenAccount>,
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
    pub member: Account<'info, Member>,
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

#[derive(Accounts)]
pub struct ExpireReward<'info> {
    pub registrar: Account<'info, Registrar>,
    #[account(mut, signer, has_one = registrar, has_one = vault, has_one = expiry_receiver)]
    pub vendor: Account<'info, RewardVendor>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(
        seeds = [registrar.to_account_info().key.as_ref(), vendor.to_account_info().key.as_ref()],
        bump = vendor.nonce
    )]
    pub vendor_signer: UncheckedAccount<'info>,
    pub expiry_receiver: Signer<'info>,
    #[account(mut, constraint = expiry_receiver_token.owner == expiry_receiver.key())]
    pub expiry_receiver_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}