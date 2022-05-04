use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("post_initialize called multiple times")]
    DuplicatePostInit {},

    #[error("Invalid launch config")]
    InvalidLaunchConfig {},

    #[error("Invalid host portion: it should be smaller than 1.0")]
    InvalidHostPortion {},

    #[error("Invalid deposit: {reason}")]
    InvalidDeposit { reason: String },

    #[error("Invalid withdraw: {reason}")]
    InvalidWithdraw { reason: String },

    #[error("Invalid withdraw tokens: {reason}")]
    InvalidWithdrawTokens { reason: String },

    #[error("Invalid admin withdraw: {reason}")]
    InvalidAdminWithdraw { reason: String },

    #[error("Invalid release tokens: {reason}")]
    InvalidReleaseTokens { reason: String },

    #[error("Fee can not be bigger than 1")]
    InvalidFee {},
}
