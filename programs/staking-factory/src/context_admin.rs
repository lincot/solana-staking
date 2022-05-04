use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

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
    #[account(
        init,
        payer = authority,
        seeds = [b"config_history", staking.key().as_ref()],
        bump,
        space = 8 + ConfigHistory::LEN,
   )]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(
        init,
        payer = authority,
        seeds = [b"stakes_history", staking.key().as_ref()],
        bump,
        space = 8 + StakesHistory::LEN,
    )]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChangeConfig<'info> {
    #[account(mut, has_one = authority)]
    pub staking: Account<'info, Staking>,
    #[account(mut, seeds = [b"config_history", staking.key().as_ref()], bump = config_history.bump)]
    pub config_history: Box<Account<'info, ConfigHistory>>,
    #[account(mut, seeds = [b"stakes_history", staking.key().as_ref()], bump = stakes_history.bump)]
    pub stakes_history: Box<Account<'info, StakesHistory>>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
