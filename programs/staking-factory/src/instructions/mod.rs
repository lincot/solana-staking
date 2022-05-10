pub use crate::instructions::{
    change_config::*, claim_reward::*, create_staking::*, deposit::*, end_unstake::*,
    initialize::*, register_member::*, stake::*, start_unstake::*, withdraw::*,
};

pub mod change_config;
pub mod claim_reward;
pub mod create_staking;
pub mod deposit;
pub mod end_unstake;
pub mod initialize;
pub mod register_member;
pub mod stake;
pub mod start_unstake;
pub mod withdraw;
