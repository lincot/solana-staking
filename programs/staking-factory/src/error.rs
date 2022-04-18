use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Reward vendors must have at least one token unit per pool token")]
    InsufficientReward,
    #[msg("Reward expiry must be after the current clock timestamp")]
    InvalidExpiry,
    #[msg("There is nothing to claim")]
    NothingToClaim,
    #[msg("Reward can only be claimed once in reward period")]
    ClaimTimelock,
    #[msg("The unstake timelock has not yet expired")]
    UnstakeTimelock,
    #[msg("The vendor is not yet eligible for expiry")]
    VendorNotYetExpired,
    #[msg("Invalid staking type")]
    InvalidType,
    Overflow,
}
