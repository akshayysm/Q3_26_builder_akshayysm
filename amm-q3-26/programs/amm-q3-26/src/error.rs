use anchor_lang::prelude::*;
use constant_product_curve::CurveError;

#[error_code]
pub enum AmmError {
    #[msg("The pool is locked.")]
    PoolLocked,

    #[msg("Slippage exceeded.")]
    SlippageExceeded,

    #[msg("Invalid amount.")]
    InvalidAmount,

    #[msg("Fee is greater than 100%.")]
    InvalidFee,

    #[msg("Overflow detected.")]
    Overflow,

    #[msg("Underflow detected.")]
    Underflow,

    #[msg("Curve error.")]
    CurveError,
}

impl From<CurveError> for AmmError {
    fn from(error: CurveError) -> AmmError {
        match error {
            CurveError::InvalidPrecision => AmmError::CurveError,
            CurveError::Overflow => AmmError::Overflow,
            CurveError::Underflow => AmmError::Underflow,
            CurveError::InvalidFeeAmount => AmmError::InvalidFee,
            CurveError::InsufficientBalance => AmmError::InvalidAmount,
            CurveError::ZeroBalance => AmmError::InvalidAmount,
            CurveError::SlippageLimitExceeded => AmmError::SlippageExceeded,
        }
    }
}