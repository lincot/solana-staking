use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    /// 6000 0x1770
    #[msg("Overflow")]
    Overflow,
    /// 6001 0x1771
    #[msg("0 is not allowed as a value")]
    Zero,
    /// 6002 0x1772
    #[msg("Cannot change staking type, may only change conditions")]
    CannotChangeStakingType,
    /// 6003 0x1773
    #[msg("Stakes history must be provided")]
    StakesHistory,
    /// 6004 0x1774
    #[msg("There is nothing to claim")]
    NothingToClaim,
    /// 6005 0x1775
    #[msg("The unstake timelock has not yet expired")]
    UnstakeTimelock,
}
