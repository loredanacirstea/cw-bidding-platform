use cosmwasm_std::{StdError, Uint128, OverflowError, DecimalRangeExceeded, CheckedFromRatioError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unauthorized - bid is closed")]
    UnauthorizedWhileClosed {},

    #[error("Unauthorized - bid is open")]
    UnauthorizedWhileOpen {},

    #[error("Invalid bid amount. Found 0 ATOM")]
    InvalidBidZeroAmount {},

    #[error("Invalid bid - sent {amount}, required at least {required_amount}")]
    InvalidBidAmount { amount: Uint128, required_amount: Uint128 },

    #[error("Invalid retract amount. Found 0 ATOM")]
    InvalidRetractZeroAmount {},
}
