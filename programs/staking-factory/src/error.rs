use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    Overflow,
    #[msg("Cannot change staking type, may only change conditions")]
    CannotChangeStakingType,
    #[msg("Stakes history must be provided")]
    StakesHistory,
    #[msg("There is nothing to claim")]
    NothingToClaim,
    #[msg("The unstake timelock has not yet expired")]
    UnstakeTimelock,
}
