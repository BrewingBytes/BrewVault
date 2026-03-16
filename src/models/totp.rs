//! Core data types for a TOTP vault entry.
//!
//! [`TotpEntry`] is the central record stored in the vault database and passed
//! through the UI. [`Algorithm`] and [`Digits`] are its enum fields; both
//! provide SQL-friendly conversions so that [`crate::storage`] can serialise
//! and deserialise them without depending on serde.

use serde::{Deserialize, Serialize};

/// Number of digits in a generated TOTP code.
///
/// Most services use six digits; eight is offered by a small number of
/// providers for extra security.
#[derive(Serialize, Deserialize, Clone)]
pub enum Digits {
    /// Standard 6-digit code (e.g. `"123456"`).
    Six,
    /// Extended 8-digit code (e.g. `"12345678"`).
    Eight,
}

impl Digits {
    /// Returns the numeric digit count as `i64` for SQL storage.
    pub fn as_i64(&self) -> i64 {
        match self {
            Digits::Six => 6,
            Digits::Eight => 8,
        }
    }
}

impl TryFrom<i64> for Digits {
    type Error = String;

    /// Converts an integer read from the database back into a [`Digits`] variant.
    ///
    /// Returns `Err` for any value other than `6` or `8`.
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            6 => Ok(Digits::Six),
            8 => Ok(Digits::Eight),
            _ => Err(format!("invalid digits value: {}", value)),
        }
    }
}

/// HMAC algorithm used to derive the TOTP code.
///
/// Matches the `algorithm` field in the [`otpauth` URI scheme](https://github.com/google/google-authenticator/wiki/Key-Uri-Format).
/// SHA-1 is the default and most widely supported; SHA-256 and SHA-512 offer
/// stronger hashing but require the service to also use them.
#[derive(Serialize, Deserialize, Clone)]
pub enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    /// Returns the canonical uppercase string used in the database and in
    /// `otpauth` URIs (e.g. `"SHA1"`, `"SHA256"`, `"SHA512"`).
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::Sha1 => "SHA1",
            Algorithm::Sha256 => "SHA256",
            Algorithm::Sha512 => "SHA512",
        }
    }
}

impl TryFrom<&str> for Algorithm {
    type Error = String;

    /// Converts a string read from the database back into an [`Algorithm`] variant.
    ///
    /// Accepts `"SHA1"`, `"SHA256"`, and `"SHA512"`. Returns `Err` for any
    /// other value.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "SHA1" => Ok(Algorithm::Sha1),
            "SHA256" => Ok(Algorithm::Sha256),
            "SHA512" => Ok(Algorithm::Sha512),
            _ => Err(format!("invalid algorithm: {}", value)),
        }
    }
}

/// A single TOTP credential stored in the vault.
///
/// Each entry maps directly to one row in the `entries` table. The `secret`
/// field holds a base32-encoded TOTP secret as provided by the service (the
/// same string encoded in a `otpauth://` QR code).
#[derive(Serialize, Deserialize, Clone)]
pub struct TotpEntry {
    /// Unique identifier for this entry (UUID v4).
    pub id: String,
    /// Service or organisation name (e.g. `"GitHub"`).
    pub issuer: String,
    /// User account identifier within the service (e.g. `"user@example.com"`).
    pub account: String,
    /// Base32-encoded TOTP secret shared with the service.
    pub secret: String,
    /// HMAC algorithm used to generate codes.
    pub algorithm: Algorithm,
    /// Number of digits in each generated code.
    pub digits: Digits,
    /// TOTP window duration in seconds (typically `30`).
    pub period: u64,
}
