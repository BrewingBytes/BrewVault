use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Digits {
    Six,
    Eight,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
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
