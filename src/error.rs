use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized - only {owner} can call it")]
    Unauthorized { owner: String },

    #[error("Invalid coin denom: {denom}. Send ATOM")]
    InvalidCoinDenom { denom: String },

    #[error("Invalid bid: {amount}. Send at least {required_amount}")]
    InvalidBid { amount: u64, required_amount: u64 },

    #[error("Bidding is closed.")]
    UnauthorizedBid {},
}
