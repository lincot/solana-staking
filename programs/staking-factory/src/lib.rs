use crate::{instructions::*, reward::*};
use anchor_lang::prelude::*;

pub mod error;
pub mod event;
pub mod instructions;
pub mod reward;
pub mod state;

declare_id!("74Gn5o8MXGWuNgApSz7kkfcdWHGpVAcrgs41ZfW1bHbK");

#[program]
pub mod staking_factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn create_staking(
        ctx: Context<CreateStaking>,
        stake_mint: Pubkey,
        reward_mint: Pubkey,
        unstake_timelock: u32,
        reward_params: RewardParams,
    ) -> Result<()> {
        instructions::create_staking(
            ctx,
            stake_mint,
            reward_mint,
            unstake_timelock,
            reward_params,
        )
    }

    pub fn change_config(
        ctx: Context<ChangeConfig>,
        new_reward_params: Option<RewardParams>,
    ) -> Result<()> {
        instructions::change_config(ctx, new_reward_params)
    }

    pub fn register_member(ctx: Context<RegisterMember>) -> Result<()> {
        instructions::register_member(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit(ctx, amount)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake(ctx, amount)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        instructions::claim_reward(ctx)
    }

    pub fn start_unstake(ctx: Context<StartUnstake>, amount: u64) -> Result<()> {
        instructions::start_unstake(ctx, amount)
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<()> {
        instructions::end_unstake(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }
}
