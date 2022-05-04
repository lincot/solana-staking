use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

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

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub staking: Account<'info, Staking>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut, associated_token::authority = member, associated_token::mint = staking.stake_mint)]
    pub member_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> Deposit<'info> {
    pub fn transfer(&self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.from.to_account_info(),
                to: self.member_vault.to_account_info(),
                authority: self.beneficiary.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut, seeds = [b"stakes_history", staking.key().as_ref()], bump = stakes_history.bump)]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(seeds = [b"factory"], bump = factory.bump)]
    pub factory: Account<'info, Factory>,
    #[account(mut, token::authority = factory.authority, token::mint = staking.reward_mint)]
    pub factory_vault: Account<'info, TokenAccount>,
    pub staking: Account<'info, Staking>,
    #[account(mut, associated_token::authority = staking, associated_token::mint = staking.reward_mint)]
    pub staking_vault: Account<'info, TokenAccount>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut, seeds = [b"stakes_history", staking.key().as_ref()], bump = stakes_history.bump)]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> ClaimReward<'info> {
    pub fn transfer_to_beneficiary(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"staking".as_ref(),
            &self.staking.id.to_le_bytes(),
            &[self.staking.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.staking_vault.to_account_info(),
                to: self.to.to_account_info(),
                authority: self.staking.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }

    pub fn transfer_to_factory_owner(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"staking".as_ref(),
            &self.staking.id.to_le_bytes(),
            &[self.staking.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.staking_vault.to_account_info(),
                to: self.factory_vault.to_account_info(),
                authority: self.staking.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct StartUnstake<'info> {
    pub staking: Account<'info, Staking>,
    #[account(seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut, seeds = [b"stakes_history", staking.key().as_ref()], bump = stakes_history.bump)]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"member", staking.id.to_le_bytes().as_ref(), beneficiary.key().as_ref()],
        bump = member.bump,
        constraint = !member.pending_unstake_active,
    )]
    pub member: Account<'info, Member>,
}

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
impl<'info> Withdraw<'info> {
    pub fn transfer(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ]];
        let cpi_accounts = Transfer {
            from: self.member_vault.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.member.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }
}
