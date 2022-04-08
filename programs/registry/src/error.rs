use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Reward vendors must have at least one token unit per pool token")]
    InsufficientReward,
    #[msg("Supplied reward must be less than 2^64")]
    RewardTooHigh,
    #[msg("Reward expiry must be after the current clock timestamp")]
    InvalidExpiry,
    #[msg("The unstake timelock has not yet expired")]
    UnstakeTimelock,
    #[msg("The vendor is not yet eligible for expiry")]
    VendorNotYetExpired,
}
