//! TOTP code generation and countdown helpers.
//!
//! The public surface is intentionally small: [`generate_code`] produces the
//! current OTP for a [`TotpEntry`], and [`seconds_remaining`] reports how many
//! seconds are left before the code rotates — useful for driving a countdown
//! indicator in the UI.
//!
//! The private [`generate_code_at`] function accepts an explicit timestamp so
//! it can be exercised with RFC 6238 test vectors without sleeping.

use chrono::Utc;
use totp_rs::TOTP;

use crate::models::{
    error::TotpError,
    totp::{Algorithm, Digits, TotpEntry},
};

/// Formats a raw TOTP digit string for display.
///
/// Inserts a space at the midpoint so the code is easier to read at a glance.
///
/// # Examples
///
/// ```text
/// "123456"   → "123 456"
/// "12345678" → "1234 5678"
/// ```
pub fn format_code(code: &str) -> String {
    let mid = code.len() / 2;
    format!("{} {}", &code[..mid], &code[mid..])
}

/// Derives up to two uppercase initials from an issuer name.
///
/// - Multi-word names use the first letter of each of the first two words:
///   `"Brewing Bytes"` → `"BB"`.
/// - Single-word names use the first two characters: `"GitHub"` → `"GI"`,
///   `"X"` → `"X"`.
pub fn initials(issuer: &str) -> String {
    let mut words = issuer.split_whitespace();
    match (words.next(), words.next()) {
        (Some(a), Some(b)) => format!(
            "{}{}",
            a.chars()
                .next()
                .unwrap_or_default()
                .to_uppercase()
                .next()
                .unwrap_or_default(),
            b.chars()
                .next()
                .unwrap_or_default()
                .to_uppercase()
                .next()
                .unwrap_or_default(),
        ),
        (Some(a), None) => a.chars().take(2).flat_map(|c| c.to_uppercase()).collect(),
        _ => String::new(),
    }
}

/// Returns the number of seconds remaining in the current TOTP window for `entry`.
///
/// The value counts down from `entry.period` to 1, resetting at each window boundary.
/// Useful for driving a countdown UI so the user knows when the code will rotate.
pub fn seconds_remaining(entry: &TotpEntry) -> u8 {
    let period = entry.period;
    (period - (Utc::now().timestamp() as u64 % period)) as u8
}

/// Normalizes a base32 secret by stripping trailing `=` padding characters.
///
/// `totp_rs` decodes base32 with `padding: false`, so passing `=` characters
/// causes decode failures. Keys exported from some apps include padding;
/// stripping it makes them accepted.
pub fn normalize_secret(secret: &str) -> String {
    secret.trim_end_matches('=').to_string()
}

/// Returns `true` if `secret` is a non-empty, valid base32-encoded string
/// that can be used as a TOTP secret.
pub fn is_valid_secret(secret: &str) -> bool {
    !secret.is_empty()
        && totp_rs::Secret::Encoded(normalize_secret(secret))
            .to_bytes()
            .is_ok()
}

/// Generates a TOTP code for `entry` at a specific Unix timestamp.
///
/// `time` is a Unix timestamp in seconds and determines which 30-second window
/// is used. This is the testable core of [`generate_code`].
///
/// # Errors
///
/// Returns [`TotpError::TOTPGenerationError`] if `entry.secret` is not valid base32
/// or if the underlying TOTP construction fails.
fn generate_code_at(entry: &TotpEntry, time: u64) -> Result<String, TotpError> {
    let secret_bytes = totp_rs::Secret::Encoded(normalize_secret(&entry.secret))
        .to_bytes()
        .map_err(|_| TotpError::TOTPGenerationError)?;

    let digits = match entry.digits {
        Digits::Six => 6,
        Digits::Eight => 8,
    };

    let totp = TOTP::new_unchecked(
        match entry.algorithm {
            Algorithm::Sha1 => totp_rs::Algorithm::SHA1,
            Algorithm::Sha256 => totp_rs::Algorithm::SHA256,
            Algorithm::Sha512 => totp_rs::Algorithm::SHA512,
        },
        digits,
        1,
        entry.period,
        secret_bytes,
    );

    Ok(totp.generate(time))
}

/// Generates the current TOTP code for a vault entry.
///
/// The secret, algorithm, digit count, and period are taken from `entry`.
/// The secret must be a base32-encoded string (the format used by authenticator apps
/// and most TOTP QR codes). Returns the raw digit string with no formatting
/// (e.g. `"123456"` for 6-digit codes).
///
/// # Errors
///
/// Returns [`TotpError::TOTPGenerationError`] if `entry.secret` is not valid base32
/// or if the underlying TOTP construction fails.
pub fn generate_code(entry: &TotpEntry) -> Result<String, TotpError> {
    generate_code_at(entry, Utc::now().timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::totp::{Algorithm, Digits, TotpEntry};
    use chrono::Utc;
    use std::{thread, time::Duration};

    fn make_entry(algorithm: Algorithm, digits: Digits, secret: &str) -> TotpEntry {
        TotpEntry {
            id: "test".to_string(),
            issuer: "ACME".to_string(),
            account: "user@example.com".to_string(),
            secret: secret.to_string(),
            algorithm,
            digits,
            period: 30,
            group: None,
        }
    }

    // RFC 6238 SHA-1 vector:
    // secret = "12345678901234567890", T = floor(1111111109 / 30) = 0x23523EC, expected = "07081804"
    #[test]
    fn test_generate_code_sha1() {
        let entry = make_entry(
            Algorithm::Sha1,
            Digits::Eight,
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        );
        let result = generate_code_at(&entry, 1111111109).unwrap();
        assert_eq!(result, "07081804");
    }

    // RFC 6238 SHA-256 vector:
    // secret = "12345678901234567890123456789012", T = 0x23523EC, expected = "68084774"
    #[test]
    fn test_generate_code_sha256() {
        let entry = make_entry(
            Algorithm::Sha256,
            Digits::Eight,
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZA",
        );
        let result = generate_code_at(&entry, 1111111109).unwrap();
        assert_eq!(result, "68084774");
    }

    // 6-digit code must come back as "XXX XXX"
    #[test]
    fn test_code_format() {
        let entry = make_entry(
            Algorithm::Sha1,
            Digits::Six,
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        );
        let result = generate_code_at(&entry, 1111111109).unwrap();
        assert_eq!(result.len(), 6);
        assert!(result.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_invalid_secret_returns_error() {
        let entry = make_entry(Algorithm::Sha1, Digits::Six, "not-base32!!!");
        assert!(generate_code(&entry).is_err());
    }

    #[test]
    fn test_seconds_remaining_range() {
        let entry = make_entry(
            Algorithm::Sha1,
            Digits::Six,
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        );

        // Verify the value decrements by 1 each second.
        let first = seconds_remaining(&entry);
        thread::sleep(Duration::from_secs(1));
        let second = seconds_remaining(&entry);
        assert_eq!(
            second,
            (first - 1) % 30,
            "value should advance by 1 per second"
        );

        // Wait for the next 30-second boundary (:00 or :30).
        let now = Utc::now().timestamp();
        let secs_to_boundary = 30 - (now % 30);
        thread::sleep(Duration::from_secs(secs_to_boundary as u64));

        // At the boundary the full window is remaining.
        assert_eq!(
            seconds_remaining(&entry),
            30,
            "at :00/:30 boundary, 30 seconds should remain"
        );

        // One second in, one second has elapsed.
        thread::sleep(Duration::from_secs(1));
        assert_eq!(
            seconds_remaining(&entry),
            29,
            "one second after boundary, 29 seconds should remain"
        );
    }

    #[test]
    fn test_format_code_six_digits() {
        assert_eq!(format_code("123456"), "123 456");
    }

    #[test]
    fn test_format_code_eight_digits() {
        assert_eq!(format_code("12345678"), "1234 5678");
    }

    #[test]
    fn test_initials_single_word() {
        assert_eq!(initials("GitHub"), "GI");
    }

    #[test]
    fn test_initials_single_char() {
        assert_eq!(initials("X"), "X");
    }

    #[test]
    fn test_initials_two_words() {
        assert_eq!(initials("Brewing Bytes"), "BB");
    }

    #[test]
    fn test_initials_empty() {
        assert_eq!(initials(""), "");
    }

    #[test]
    fn test_is_valid_secret_with_padding() {
        // Some apps export keys with trailing '=' padding; stripping makes them valid.
        let key = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"; // 32 chars, 20 bytes — valid without padding
        let padded = format!("{}======", key); // extra '=' that would break totp_rs
        assert!(is_valid_secret(&padded));
    }

    #[test]
    fn test_generate_code_padded_secret() {
        // Key with trailing '=' padding — normalize_secret strips them before decoding.
        let key = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let padded = format!("{}======", key);
        let entry = make_entry(Algorithm::Sha1, Digits::Six, &padded);
        let code = generate_code(&entry).unwrap();
        assert!(!code.is_empty());
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_normalize_secret_strips_padding() {
        assert_eq!(
            normalize_secret("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ======"),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        );
    }

    #[test]
    fn test_normalize_secret_no_op_without_padding() {
        let key = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        assert_eq!(normalize_secret(key), key);
    }

    #[test]
    fn test_normalize_secret_various_lengths() {
        // Keys with any number of trailing '=' chars should be stripped cleanly.
        for pads in [0usize, 1, 2, 3, 4, 5, 6] {
            let base = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
            let key = format!("{}{}", base, "=".repeat(pads));
            assert_eq!(
                normalize_secret(&key),
                base,
                "failed for {pads} padding chars"
            );
        }
    }

    #[test]
    fn test_generate_code_short_secret() {
        // 16-char key decodes to only 10 bytes (80 bits), below the RFC 4226 minimum.
        // We use new_unchecked so real-world short keys still produce codes.
        let entry = make_entry(Algorithm::Sha1, Digits::Six, "I65VU7K5ZQL7WB4E");
        let code = generate_code(&entry).unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_is_valid_secret_invalid_chars_still_rejected() {
        assert!(!is_valid_secret("not-base32!!!"));
    }
}
