use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Digits {
    Six,
    Eight,
}

impl Digits {
    pub fn as_i64(&self) -> i64 {
        match self {
            Digits::Six => 6,
            Digits::Eight => 8,
        }
    }
}

impl TryFrom<i64> for Digits {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            6 => Ok(Digits::Six),
            8 => Ok(Digits::Eight),
            _ => Err(format!("invalid digits value: {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
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

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "SHA1" => Ok(Algorithm::Sha1),
            "SHA256" => Ok(Algorithm::Sha256),
            "SHA512" => Ok(Algorithm::Sha512),
            _ => Err(format!("invalid algorithm: {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TotpEntry {
    pub id: String,
    pub issuer: String,
    pub account: String,
    pub secret: String,
    pub algorithm: Algorithm,
    pub digits: Digits,
    pub period: u64,
}
