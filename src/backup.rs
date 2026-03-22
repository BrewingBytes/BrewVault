//! Backup export and import in Aegis-compatible encrypted format.
//!
//! ## File format
//!
//! The `.brewvault` file is a JSON object that follows the Aegis vault v1
//! envelope / v3 database format:
//!
//! ```text
//! {
//!   "version": 1,
//!   "header": {
//!     "slots": [{ "type": 1, "uuid": "...", "key": "<hex>",
//!                 "key_params": { "nonce": "<hex>", "tag": "<hex>" },
//!                 "n": 32768, "r": 8, "p": 1, "salt": "<hex>" }],
//!     "params": { "nonce": "<hex>", "tag": "<hex>" }
//!   },
//!   "db": "<base64 of ciphertext>"   // AES-256-GCM encrypted JSON
//! }
//! ```
//!
//! ## Crypto summary
//!
//! 1. Argon2id(passphrase, random_salt, m=32768, t=1, p=1) → `slot_key` (32 bytes)
//! 2. Random `master_key` (32 bytes)
//! 3. AES-256-GCM(slot_key, random_nonce12) → `encrypted_master_key` (32 B ciphertext + 16 B tag)
//! 4. AES-256-GCM(master_key, random_nonce12) → `encrypted_db` (ciphertext + 16 B tag)
//!
//! Fields `nonce` and `tag` are **hex-encoded**; the `db` field is
//! **base64-encoded** (standard, no padding stripped).

use std::path::Path;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::models::error::{BackupError, TotpError};
use crate::models::totp::{Algorithm as TotpAlgorithm, Digits, TotpEntry};

// ─── Aegis JSON structures ────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct AesParams {
    nonce: String,
    tag: String,
}

#[derive(Serialize, Deserialize)]
struct Slot {
    r#type: u8,
    uuid: String,
    key: String,
    key_params: AesParams,
    n: u32,
    r: u32,
    p: u32,
    salt: String,
}

#[derive(Serialize, Deserialize)]
struct Header {
    slots: Vec<Slot>,
    params: AesParams,
}

#[derive(Serialize, Deserialize)]
struct AegisEnvelope {
    version: u32,
    header: Header,
    db: String,
}

// ─── DB content (Aegis v3 db schema) ─────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct DbEntry {
    r#type: String,
    uuid: String,
    name: String,
    issuer: String,
    note: String,
    favorite: bool,
    icon: Option<String>,
    icon_mime: Option<String>,
    info: DbEntryInfo,
    groups: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct DbEntryInfo {
    secret: String,
    algo: String,
    digits: u32,
    period: u64,
}

#[derive(Serialize, Deserialize)]
struct Db {
    version: u32,
    entries: Vec<DbEntry>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, BackupError> {
    if !s.len().is_multiple_of(2) {
        return Err(BackupError::InvalidFormat(
            "invalid hex encoding (odd length)".into(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| BackupError::InvalidFormat("invalid hex encoding".into()))
        })
        .collect()
}

/// Derive a 32-byte key from a passphrase + salt using Argon2id with explicit parameters.
fn derive_key_with_params(
    passphrase: &str,
    salt: &[u8],
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
) -> Result<[u8; 32], BackupError> {
    let params = Params::new(m_cost, t_cost, p_cost, Some(32))
        .map_err(|e| BackupError::InvalidFormat(format!("argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| BackupError::InvalidFormat(format!("argon2: {e}")))?;
    Ok(key)
}

/// Derive a 32-byte key from a passphrase + salt using Argon2id with default parameters.
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], BackupError> {
    derive_key_with_params(passphrase, salt, 32768, 1, 1)
}

/// AES-256-GCM encrypt. Returns `(nonce_bytes, ciphertext_plus_tag)`, where
/// `ciphertext_plus_tag` is `ciphertext || tag` (last 16 bytes are the tag).
fn aes_encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), BackupError> {
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| BackupError::InvalidFormat(format!("aes key: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    // aes-gcm returns ciphertext || tag (last 16 bytes are tag)
    let ct_plus_tag = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| BackupError::InvalidFormat("encryption failed".into()))?;
    Ok((nonce_bytes.to_vec(), ct_plus_tag))
}

/// AES-256-GCM decrypt. `ciphertext_with_tag` must include the 16-byte tag appended.
fn aes_decrypt(
    key: &[u8; 32],
    nonce_bytes: &[u8],
    ct_plus_tag: &[u8],
) -> Result<Vec<u8>, BackupError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| BackupError::InvalidFormat(format!("aes key: {e}")))?;
    if nonce_bytes.len() != 12 {
        return Err(BackupError::WrongPassphrase);
    }
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ct_plus_tag)
        .map_err(|_| BackupError::WrongPassphrase)
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Export `entries` to an Aegis-compatible encrypted `.brewvault` file at `path`.
pub fn export_vault(entries: &[TotpEntry], passphrase: &str, path: &Path) -> Result<(), TotpError> {
    // 1. Build db JSON
    let db_entries: Vec<DbEntry> = entries
        .iter()
        .map(|e| DbEntry {
            r#type: "totp".into(),
            uuid: e.id.clone(),
            name: e.account.clone(),
            issuer: e.issuer.clone(),
            note: String::new(),
            favorite: false,
            icon: None,
            icon_mime: None,
            info: DbEntryInfo {
                secret: e.secret.clone(),
                algo: e.algorithm.as_str().to_string(),
                digits: match e.digits {
                    Digits::Six => 6,
                    Digits::Eight => 8,
                },
                period: e.period,
            },
            groups: e
                .group
                .as_deref()
                .map(|g| vec![g.to_string()])
                .unwrap_or_default(),
        })
        .collect();

    let db = Db {
        version: 3,
        entries: db_entries,
    };
    let db_json = serde_json::to_vec(&db)
        .map_err(|e| TotpError::Backup(BackupError::InvalidFormat(e.to_string())))?;

    // 2. Generate random master key + salt
    let mut master_key = [0u8; 32];
    OsRng.fill_bytes(&mut master_key);
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    // 3. Derive slot key from passphrase
    let slot_key = derive_key(passphrase, &salt).map_err(TotpError::Backup)?;

    // 4. Encrypt master key with slot key
    let (mk_nonce, mk_ct_tag) = aes_encrypt(&slot_key, &master_key).map_err(TotpError::Backup)?;
    // split: last 16 bytes = tag, rest = ciphertext (32 bytes for 32-byte key)
    let (mk_ct, mk_tag) = mk_ct_tag.split_at(mk_ct_tag.len() - 16);

    // 5. Encrypt db JSON with master key
    let (db_nonce, db_ct_tag) = aes_encrypt(&master_key, &db_json).map_err(TotpError::Backup)?;
    let (db_ct, db_tag) = db_ct_tag.split_at(db_ct_tag.len() - 16);

    // 6. Build slot UUID
    let slot_uuid = uuid::Uuid::new_v4().to_string();

    let envelope = AegisEnvelope {
        version: 1,
        header: Header {
            slots: vec![Slot {
                r#type: 1,
                uuid: slot_uuid,
                key: hex_encode(mk_ct),
                key_params: AesParams {
                    nonce: hex_encode(&mk_nonce),
                    tag: hex_encode(mk_tag),
                },
                n: 32768,
                r: 1,
                p: 1,
                salt: hex_encode(&salt),
            }],
            params: AesParams {
                nonce: hex_encode(&db_nonce),
                tag: hex_encode(db_tag),
            },
        },
        db: B64.encode(db_ct),
    };

    let json = serde_json::to_string_pretty(&envelope)
        .map_err(|e| TotpError::Backup(BackupError::InvalidFormat(e.to_string())))?;
    std::fs::write(path, json).map_err(|e| TotpError::Backup(BackupError::Io(e)))?;

    Ok(())
}

/// Import entries from an Aegis-compatible encrypted `.brewvault` file.
///
/// Returns the decoded `Vec<TotpEntry>`. The caller is responsible for
/// calling `AppState::add_entry` for each, which handles de-duplication.
/// Maximum size (50 MB) accepted for import — guards against OOM on malformed files.
const MAX_IMPORT_BYTES: u64 = 50 * 1024 * 1024;

pub fn import_vault(path: &Path, passphrase: &str) -> Result<Vec<TotpEntry>, TotpError> {
    let file_size = std::fs::metadata(path)
        .map_err(|e| TotpError::Backup(BackupError::Io(e)))?
        .len();
    if file_size > MAX_IMPORT_BYTES {
        return Err(TotpError::Backup(BackupError::InvalidFormat(
            "file too large to be a valid .brewvault backup".into(),
        )));
    }
    let raw = std::fs::read(path).map_err(|e| TotpError::Backup(BackupError::Io(e)))?;
    let envelope: AegisEnvelope = serde_json::from_slice(&raw)
        .map_err(|e| TotpError::Backup(BackupError::InvalidFormat(e.to_string())))?;

    if envelope.version != 1 {
        return Err(TotpError::Backup(BackupError::InvalidFormat(format!(
            "unsupported version: {}",
            envelope.version
        ))));
    }

    // Find a password slot (type == 1)
    let slot = envelope
        .header
        .slots
        .iter()
        .find(|s| s.r#type == 1)
        .ok_or_else(|| {
            // Check for biometric-only (type == 2)
            if envelope.header.slots.iter().any(|s| s.r#type == 2) {
                TotpError::Backup(BackupError::BiometricNotSupported)
            } else {
                TotpError::Backup(BackupError::InvalidFormat("no supported slot found".into()))
            }
        })?;

    // Derive slot key from passphrase using per-slot Argon2 parameters
    let salt = hex_decode(&slot.salt).map_err(TotpError::Backup)?;
    let slot_key = derive_key_with_params(passphrase, &salt, slot.n, slot.r, slot.p)
        .map_err(TotpError::Backup)?;

    // Decrypt master key
    let mk_nonce = hex_decode(&slot.key_params.nonce).map_err(TotpError::Backup)?;
    let mk_ct = hex_decode(&slot.key).map_err(TotpError::Backup)?;
    let mk_tag = hex_decode(&slot.key_params.tag).map_err(TotpError::Backup)?;
    let mut mk_ct_tag = mk_ct;
    mk_ct_tag.extend_from_slice(&mk_tag);
    let master_key_vec =
        aes_decrypt(&slot_key, &mk_nonce, &mk_ct_tag).map_err(TotpError::Backup)?;
    let mut master_key = [0u8; 32];
    if master_key_vec.len() != 32 {
        return Err(TotpError::Backup(BackupError::InvalidFormat(
            "master key wrong length".into(),
        )));
    }
    master_key.copy_from_slice(&master_key_vec);

    // Decrypt db
    let db_nonce = hex_decode(&envelope.header.params.nonce).map_err(TotpError::Backup)?;
    let db_ct = B64
        .decode(&envelope.db)
        .map_err(|_| TotpError::Backup(BackupError::InvalidFormat("invalid base64 db".into())))?;
    let db_tag = hex_decode(&envelope.header.params.tag).map_err(TotpError::Backup)?;
    let mut db_ct_tag = db_ct;
    db_ct_tag.extend_from_slice(&db_tag);
    let db_json = aes_decrypt(&master_key, &db_nonce, &db_ct_tag)
        .map_err(|_| TotpError::Backup(BackupError::WrongPassphrase))?;

    let db: Db = serde_json::from_slice(&db_json)
        .map_err(|e| TotpError::Backup(BackupError::InvalidFormat(e.to_string())))?;

    // Map Aegis db entries → TotpEntry
    let mut entries = Vec::with_capacity(db.entries.len());
    for e in db.entries {
        let algorithm = TotpAlgorithm::try_from(e.info.algo.as_str()).map_err(|_| {
            TotpError::Backup(BackupError::InvalidFormat(format!(
                "unsupported algorithm: {}",
                e.info.algo
            )))
        })?;
        let digits = match e.info.digits {
            6 => Digits::Six,
            8 => Digits::Eight,
            other => {
                return Err(TotpError::Backup(BackupError::InvalidFormat(format!(
                    "unsupported digit count: {other}"
                ))));
            }
        };
        let group = e.groups.into_iter().next();
        entries.push(TotpEntry {
            id: e.uuid,
            issuer: e.issuer,
            account: e.name,
            secret: e.info.secret,
            algorithm,
            digits,
            period: e.info.period,
            group,
            sort_order: 0, // assigned by add_entry
        });
    }

    Ok(entries)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn sample_entries() -> Vec<TotpEntry> {
        vec![
            TotpEntry {
                id: "11111111-1111-1111-1111-111111111111".into(),
                issuer: "GitHub".into(),
                account: "user@example.com".into(),
                secret: "JBSWY3DPEHPK3PXP".into(),
                algorithm: TotpAlgorithm::Sha1,
                digits: Digits::Six,
                period: 30,
                group: Some("Dev".into()),
                sort_order: 1,
            },
            TotpEntry {
                id: "22222222-2222-2222-2222-222222222222".into(),
                issuer: "Google".into(),
                account: "user@gmail.com".into(),
                secret: "JBSWY3DPEHPK3PXP".into(),
                algorithm: TotpAlgorithm::Sha1,
                digits: Digits::Six,
                period: 30,
                group: None,
                sort_order: 2,
            },
        ]
    }

    #[test]
    fn test_round_trip() {
        let entries = sample_entries();
        let file = NamedTempFile::new().unwrap();
        export_vault(&entries, "correct-passphrase", file.path()).unwrap();
        let imported = import_vault(file.path(), "correct-passphrase").unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].id, entries[0].id);
        assert_eq!(imported[0].issuer, entries[0].issuer);
        assert_eq!(imported[0].secret, entries[0].secret);
        assert_eq!(imported[0].group, entries[0].group);
        assert_eq!(imported[1].id, entries[1].id);
        assert_eq!(imported[1].group, None);
    }

    #[test]
    fn test_wrong_passphrase() {
        let entries = sample_entries();
        let file = NamedTempFile::new().unwrap();
        export_vault(&entries, "correct-passphrase", file.path()).unwrap();
        let result = import_vault(file.path(), "wrong-passphrase");
        assert!(
            matches!(result, Err(TotpError::Backup(BackupError::WrongPassphrase))),
            "expected WrongPassphrase, got {result:?}"
        );
    }

    #[test]
    fn test_corrupt_file() {
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"not valid json at all").unwrap();
        let result = import_vault(file.path(), "any");
        assert!(
            matches!(
                result,
                Err(TotpError::Backup(BackupError::InvalidFormat(_)))
            ),
            "expected InvalidFormat, got {result:?}"
        );
    }

    #[test]
    fn test_empty_vault() {
        let file = NamedTempFile::new().unwrap();
        export_vault(&[], "passphrase", file.path()).unwrap();
        let imported = import_vault(file.path(), "passphrase").unwrap();
        assert_eq!(imported.len(), 0);
    }

    #[test]
    fn test_aegis_structure() {
        let entries = sample_entries();
        let file = NamedTempFile::new().unwrap();
        export_vault(&entries, "test-pw", file.path()).unwrap();
        let raw = std::fs::read_to_string(file.path()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["version"], 1);
        let slots = v["header"]["slots"].as_array().unwrap();
        assert!(!slots.is_empty());
        assert_eq!(slots[0]["type"], 1);
        // nonce + tag should be hex strings
        let nonce = v["header"]["params"]["nonce"].as_str().unwrap();
        assert_eq!(nonce.len(), 24); // 12 bytes * 2
        // db should be valid base64
        let db_b64 = v["db"].as_str().unwrap();
        assert!(B64.decode(db_b64).is_ok());
    }
}
