use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

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
impl<'info> Deposit<'info> {
    pub fn transfer(&self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.depositor.to_account_info(),
                to: self.available.to_account_info(),
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
impl<'info> Stake<'info> {
    pub fn transfer(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.available.to_account_info(),
                to: self.stake.to_account_info(),
                authority: self.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(seeds = [b"factory"], bump = factory.bump)]
    pub factory: Box<Account<'info, Factory>>,
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
impl<'info> ClaimReward<'info> {
    pub fn transfer_to_user(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"staking".as_ref(),
            &self.staking.id.to_le_bytes(),
            &[self.staking.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.reward_vault.to_account_info(),
                to: self.destination.to_account_info(),
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
                from: self.reward_vault.to_account_info(),
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
impl<'info> StartUnstake<'info> {
    pub fn transfer(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.stake.to_account_info(),
                to: self.pending.to_account_info(),
                authority: self.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, amount)
    }
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
impl<'info> EndUnstake<'info> {
    pub fn transfer(&self) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pending.to_account_info(),
                to: self.available.to_account_info(),
                authority: self.member.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, self.pending_withdrawal.amount)
    }
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
impl<'info> Withdraw<'info> {
    pub fn transfer(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[
            b"member".as_ref(),
            &self.staking.id.to_le_bytes(),
            self.beneficiary.to_account_info().key.as_ref(),
            &[self.member.bump],
        ]];
        let cpi_accounts = Transfer {
            from: self.available.to_account_info(),
            to: self.destination.to_account_info(),
            authority: self.member.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }
}
