use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Reward vendors must have at least one token unit per pool token")]
    InsufficientReward,
    #[msg("Reward expiry must be after the current clock timestamp.")]
    InvalidExpiry,
    #[msg("The unstake timelock has not yet expired.")]
    UnstakeTimelock,
}
