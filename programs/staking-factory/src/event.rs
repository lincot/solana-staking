use crate::reward::RewardParams;
use anchor_lang::prelude::*;

#[event]
pub struct InitializeEvent {}

#[event]
pub struct CreateStakingEvent {
    pub id: u16,
    pub authority: Pubkey,
    pub reward_params: RewardParams,
}

#[event]
pub struct ChangeConfigEvent {
    pub id: u16,
    pub new_reward_params: Option<RewardParams>,
}

#[event]
pub struct RegisterMemberEvent {
    pub beneficiary: Pubkey,
}

#[event]
pub struct DepositEvent {
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[event]
pub struct StakeEvent {
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ClaimRewardEvent {
    pub beneficiary: Pubkey,
    pub amount_to_beneficiary: u64,
    pub factory_fee: u64,
}

#[event]
pub struct StartUnstakeEvent {
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EndUnstakeEvent {
    pub beneficiary: Pubkey,
}

#[event]
pub struct WithdrawEvent {
    pub beneficiary: Pubkey,
    pub amount: u64,
}
