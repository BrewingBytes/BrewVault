//! Application-wide error type.
//!
//! [`TotpError`] is the single `Err` variant used across the codebase.
//! Adding [`From`] implementations for upstream error types (e.g.
//! `rusqlite::Error`) keeps call sites clean — most functions can propagate
//! errors with `?` without any explicit mapping.

/// Errors specific to backup export and import operations.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// An I/O error reading or writing the backup file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The file is not a valid `.brewvault` / Aegis-compatible JSON file.
    #[error("invalid backup file: {0}")]
    InvalidFormat(String),
    /// The supplied passphrase is incorrect (AES-GCM decryption failed).
    #[error("incorrect passphrase")]
    WrongPassphrase,
    /// The backup uses biometric-only encryption, which BrewVault cannot decode.
    #[error("biometric-only backups are not supported")]
    BiometricNotSupported,
}

/// All errors that BrewVault can encounter at runtime.
#[derive(Debug, thiserror::Error)]
pub enum TotpError {
    /// The TOTP library failed to build or generate a code, usually because
    /// the entry's secret is not valid base32.
    #[error("failed to generate TOTP code")]
    TOTPGenerationError,
    /// A database operation failed. The inner [`rusqlite::Error`] carries the
    /// original cause and can be displayed or logged directly.
    #[error("storage error: {0}")]
    StorageError(#[from] rusqlite::Error),
    /// The supplied master password is incorrect.
    #[error("wrong password")]
    WrongPassword,
    /// The new password and confirmation do not match.
    #[error("passwords do not match")]
    PasswordMismatch,
    /// The supplied password is too short (minimum 8 characters).
    #[error("password must be at least 8 characters")]
    PasswordTooShort,
    /// The supplied password is a reserved sentinel value.
    #[error("that password is reserved \u{2014} choose another")]
    ReservedPassword,
    /// An operation that requires an unlocked vault was attempted while locked.
    #[error("vault is locked")]
    VaultLocked,
    /// A backup export or import operation failed.
    #[error(transparent)]
    Backup(BackupError),
}
