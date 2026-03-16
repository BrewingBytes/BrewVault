//! Application-wide error type.
//!
//! [`TotpError`] is the single `Err` variant used across the codebase.
//! Adding [`From`] implementations for upstream error types (e.g.
//! `rusqlite::Error`) keeps call sites clean — most functions can propagate
//! errors with `?` without any explicit mapping.

/// All errors that BrewVault can encounter at runtime.
#[derive(Debug)]
pub enum TotpError {
    /// The TOTP library failed to build or generate a code, usually because
    /// the entry's secret is not valid base32.
    TOTPGenerationError,
    /// A database operation failed. The inner [`rusqlite::Error`] carries the
    /// original cause and can be displayed or logged directly.
    StorageError(rusqlite::Error),
}

impl std::fmt::Display for TotpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TotpError::TOTPGenerationError => write!(f, "failed to generate TOTP code"),
            TotpError::StorageError(e) => write!(f, "storage error: {e}"),
        }
    }
}

impl From<rusqlite::Error> for TotpError {
    /// Wraps a [`rusqlite::Error`] so that storage functions can be called
    /// with `?` in any context that returns `Result<_, TotpError>`.
    fn from(e: rusqlite::Error) -> Self {
        TotpError::StorageError(e)
    }
}
