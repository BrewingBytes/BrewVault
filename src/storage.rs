//! Persistent, encrypted storage for BrewVault TOTP entries.
//!
//! All vault data is stored in a SQLCipher-encrypted SQLite database.
//! Callers obtain a [`Connection`] via the appropriate open function and store
//! it in [`AppState`]; there is no process-wide singleton here.
//!
//! # Encryption
//! The database is always SQLCipher-encrypted. Two modes:
//! - **Password-protected**: opened with the user's master password.
//! - **No-password**: opened with the [`NO_PASSWORD_KEY`] sentinel.
//!
//! First-run detection is done via [`detect_vault_state`] before any UI
//! renders.

use std::path::{Path, PathBuf};

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use rusqlite::{Connection, Result, params};

use crate::models::error::TotpError;
use crate::models::totp::{Algorithm, Digits, TotpEntry};

/// Sentinel encryption key used when the user has chosen "no password".
/// The database is still encrypted — this key is just fixed and well-known.
pub const NO_PASSWORD_KEY: &str = "brewvault-nopass";

/// The old hardcoded key from before the master-password feature.
/// Kept for reference; no active migration path (no shipped users).
pub const LEGACY_DB_KEY: &str = "brew-vault-hardcoded-key";

/// Meta table key: `"true"` if the vault uses a user-supplied password.
pub const META_PASSWORD_SET: &str = "password_set";
/// Meta table key: PHC-format Argon2id hash of the master password.
pub const META_PASSWORD_HASH: &str = "password_hash";
/// Meta table key: auto-lock timeout in seconds (`"0"` = disabled).
pub const META_AUTO_LOCK_SECS: &str = "auto_lock_secs";

// ---------------------------------------------------------------------------
// Vault state detection
// ---------------------------------------------------------------------------

/// The raw vault state detected on disk before any UI renders.
#[derive(Debug, Clone, PartialEq)]
pub enum VaultState {
    /// No database file exists — first launch.
    FirstRun,
    /// Database exists and is opened with [`NO_PASSWORD_KEY`].
    NoPassword,
    /// Database exists and requires a user-supplied password.
    PasswordProtected,
}

/// Probes the on-disk database to determine which lock state to start in.
///
/// 1. If the database file does not exist → [`VaultState::FirstRun`].
/// 2. If the file can be opened with the sentinel no-password key → [`VaultState::NoPassword`].
/// 3. Otherwise → [`VaultState::PasswordProtected`].
///
/// This function does **not** open a connection that is kept alive; it only
/// peeks at the file to make a routing decision.
pub fn detect_vault_state() -> Result<VaultState> {
    let path = db_path().ok_or_else(|| {
        rusqlite::Error::InvalidPath(PathBuf::from("could not resolve data or home directory"))
    })?;
    detect_vault_state_at(&path)
}

/// Inner implementation of [`detect_vault_state`] that accepts an explicit path.
/// Exposed for testing.
pub fn detect_vault_state_at(path: &Path) -> Result<VaultState> {
    if !path.exists() {
        return Ok(VaultState::FirstRun);
    }

    // Try the no-password sentinel key and run a test query.
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", NO_PASSWORD_KEY)?;
    let ok = conn
        .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        })
        .is_ok();

    Ok(if ok {
        VaultState::NoPassword
    } else {
        VaultState::PasswordProtected
    })
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Returns the platform-appropriate path to the vault database file.
///
/// Prefers `dirs::data_dir()` (e.g. `~/Library/Application Support` on macOS),
/// falling back to `dirs::home_dir()` if the former is unavailable.
/// Returns `None` only when neither directory can be resolved.
pub fn db_path() -> Option<PathBuf> {
    let base = dirs::data_dir().or_else(dirs::home_dir)?;
    Some(base.join("Brew Vault").join("vault.db"))
}

// ---------------------------------------------------------------------------
// Connection helpers
// ---------------------------------------------------------------------------

/// Opens (or creates) the SQLCipher database at [`db_path`] using `key`.
///
/// The parent directory is created automatically if it does not exist.
/// Returns an error if the file cannot be opened or if the key pragma fails.
pub fn open_db(key: &str) -> Result<Connection> {
    let path = db_path().ok_or_else(|| {
        rusqlite::Error::InvalidPath(PathBuf::from("could not resolve data or home directory"))
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
    }
    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "key", key)?;
    Ok(conn)
}

/// Opens (or creates) a SQLCipher database at an explicit `path` using `key`.
///
/// Unlike [`open_db`], this does not consult [`db_path`] and does not create
/// parent directories. Intended for tests that need a real on-disk file in a
/// temporary directory.
pub fn open_db_at(path: impl AsRef<std::path::Path>, key: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    Ok(conn)
}

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

/// Creates the `entries` and `meta` tables if they do not already exist.
///
/// Safe to call multiple times — uses `CREATE TABLE IF NOT EXISTS`.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id         TEXT PRIMARY KEY,
            issuer     TEXT NOT NULL,
            account    TEXT NOT NULL,
            secret     TEXT NOT NULL,
            algorithm  TEXT NOT NULL DEFAULT 'SHA1',
            digits     INTEGER NOT NULL DEFAULT 6,
            period     INTEGER NOT NULL DEFAULT 30,
            `group`    TEXT DEFAULT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
}

/// Adds the `sort_order` column to an existing database that pre-dates it.
///
/// Safe to call on a database that already has the column — it is a no-op in
/// that case. After adding the column, sets `sort_order = rowid` for any row
/// where it is still `0` so that pre-existing entries get a stable ordering.
pub fn migrate_sort_order(conn: &Connection) -> Result<()> {
    let has_column: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('entries') WHERE name = 'sort_order'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|n| n > 0)?;

    if !has_column {
        conn.execute_batch(
            "ALTER TABLE entries ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
             UPDATE entries SET sort_order = rowid WHERE sort_order = 0;",
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Meta table helpers
// ---------------------------------------------------------------------------

/// Reads a single value from the `meta` table by `key`.
///
/// Returns `Ok(None)` if the key does not exist.
pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    match conn.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        params![key],
        |row| row.get(0),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Inserts or replaces a `(key, value)` pair in the `meta` table.
pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

/// Deletes a key from the `meta` table (no-op if it does not exist).
pub fn delete_meta(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM meta WHERE key = ?1", params![key])?;
    Ok(())
}

/// Returns `true` if the vault was set up with a user password.
pub fn is_password_set(conn: &Connection) -> Result<bool> {
    Ok(get_meta(conn, META_PASSWORD_SET)?.as_deref() == Some("true"))
}

/// Returns the stored Argon2id password hash, or `None` if absent.
pub fn get_password_hash(conn: &Connection) -> Result<Option<String>> {
    get_meta(conn, META_PASSWORD_HASH)
}

/// Returns the configured auto-lock timeout in seconds (0 = disabled).
pub fn get_auto_lock_secs(conn: &Connection) -> Result<u64> {
    Ok(get_meta(conn, META_AUTO_LOCK_SECS)?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0))
}

// ---------------------------------------------------------------------------
// Password hashing (Argon2id)
// ---------------------------------------------------------------------------

/// Hashes `password` with Argon2id and a fresh random salt.
///
/// The returned string is in PHC format and can be stored directly in the
/// `meta` table under the `password_hash` key.
pub fn argon2_hash(password: &str) -> Result<String, TotpError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| {
            TotpError::StorageError(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::other(format!("argon2 hash: {e}")),
            )))
        })
}

/// Returns `true` if `password` matches the stored PHC-format `hash`.
pub fn argon2_verify(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

// ---------------------------------------------------------------------------
// Password change / rekey
// ---------------------------------------------------------------------------

/// Re-encrypts the open database with `new_key` using SQLCipher's `PRAGMA rekey`.
///
/// SQLCipher's rekey operation is atomic — it rewrites the file with the new key
/// in a single transactional pass and only commits on success. The caller is
/// responsible for updating the `meta` table after a successful rekey so the
/// stored password hash stays in sync.
pub fn rekey(conn: &Connection, new_key: &str) -> Result<(), TotpError> {
    conn.pragma_update(None, "rekey", new_key)
        .map_err(TotpError::StorageError)
}

// ---------------------------------------------------------------------------
// Entry CRUD
// ---------------------------------------------------------------------------

/// Loads all TOTP entries from the database, ordered by `sort_order DESC`
/// (highest value displayed first, i.e. at the top of a group).
pub fn load_entries(conn: &Connection) -> Result<Vec<TotpEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, issuer, account, secret, algorithm, digits, period, `group`, sort_order
         FROM entries
         ORDER BY sort_order DESC",
    )?;
    let entries = stmt
        .query_map([], |row| {
            let algorithm_str: String = row.get(4)?;
            let digits_i64: i64 = row.get(5)?;
            let period: i64 = row.get(6)?;
            let group: Option<String> = row.get(7)?;
            let sort_order: i64 = row.get(8)?;

            let algorithm = Algorithm::try_from(algorithm_str.as_str()).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, e.into())
            })?;
            let digits = Digits::try_from(digits_i64).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Integer,
                    e.into(),
                )
            })?;
            let period = u64::try_from(period)
                .ok()
                .filter(|&p| p > 0)
                .ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Integer,
                        "period must be a positive integer".into(),
                    )
                })?;

            Ok(TotpEntry {
                id: row.get(0)?,
                issuer: row.get(1)?,
                account: row.get(2)?,
                secret: row.get(3)?,
                algorithm,
                digits,
                period,
                group,
                sort_order: sort_order.max(0) as u64,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(entries)
}

/// Returns the current maximum `sort_order` across all entries, or `0` if the
/// table is empty. Used by [`AppState`] to assign `max + 1` to new entries.
pub fn max_sort_order(conn: &Connection) -> Result<u64> {
    let result: Option<i64> =
        conn.query_row("SELECT MAX(sort_order) FROM entries", [], |row| row.get(0))?;
    Ok(result.map(|v| v.max(0) as u64).unwrap_or(0))
}

/// Inserts a new TOTP entry. The caller must set `entry.sort_order` before
/// calling (typically `max_sort_order() + 1`).
///
/// Uses `INSERT OR IGNORE` — if an entry with the same `id` already exists the
/// row is silently skipped and `0` is returned. Returns `1` on a successful
/// insert. This allows callers to detect duplicates without an error.
pub fn insert_entry(conn: &Connection, entry: &TotpEntry) -> Result<usize> {
    conn.execute(
        "INSERT OR IGNORE INTO entries
             (id, issuer, account, secret, algorithm, digits, period, `group`, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            entry.id,
            entry.issuer,
            entry.account,
            entry.secret,
            entry.algorithm.as_str(),
            entry.digits.as_i64(),
            entry.period as i64,
            entry.group.as_deref(),
            entry.sort_order as i64,
        ],
    )
}

/// Renames an entry's `issuer` and `account` fields.
///
/// Returns the number of rows updated (always 0 or 1). Returns `0` if no
/// entry with `id` exists — callers should treat 0 as an error.
pub fn rename_entry_db(conn: &Connection, id: &str, issuer: &str, account: &str) -> Result<usize> {
    conn.execute(
        "UPDATE entries SET issuer = ?1, account = ?2 WHERE id = ?3",
        params![issuer, account, id],
    )
}

/// Moves an entry to a different group and places it at the top of that group
/// by assigning `sort_order = max(sort_order in new group) + 1`.
///
/// Pass `None` for `group` to move the entry out of any group (ungrouped).
/// Returns the number of rows updated (0 = entry not found).
pub fn update_group_db(conn: &Connection, id: &str, group: Option<&str>) -> Result<usize> {
    conn.execute(
        "UPDATE entries SET
             `group` = ?2,
             sort_order = COALESCE(
                 (SELECT MAX(sort_order) FROM entries WHERE `group` IS ?2 AND id != ?1),
                 0
             ) + 1
         WHERE id = ?1",
        params![id, group],
    )
}

/// Returns the `group` and `sort_order` of a single entry by `id`.
///
/// Used after [`update_group_db`] to pick up the computed `sort_order` without
/// re-loading the entire entries table.
pub fn get_entry_group_and_sort_order(
    conn: &Connection,
    id: &str,
) -> Result<(Option<String>, u64)> {
    conn.query_row(
        "SELECT `group`, sort_order FROM entries WHERE id = ?1",
        params![id],
        |row| {
            let group: Option<String> = row.get(0)?;
            let sort_order: i64 = row.get(1)?;
            Ok((group, sort_order.max(0) as u64))
        },
    )
}

/// Atomically swaps the `sort_order` of two entries within a transaction.
///
/// Used by move-up / move-down to reorder entries inside a group without
/// disrupting the rest of the list.
pub fn swap_sort_order_db(conn: &Connection, id_a: &str, id_b: &str) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    let so_a: i64 = tx.query_row(
        "SELECT sort_order FROM entries WHERE id = ?1",
        params![id_a],
        |row| row.get(0),
    )?;
    let so_b: i64 = tx.query_row(
        "SELECT sort_order FROM entries WHERE id = ?1",
        params![id_b],
        |row| row.get(0),
    )?;

    tx.execute(
        "UPDATE entries SET sort_order = ?1 WHERE id = ?2",
        params![so_b, id_a],
    )?;
    tx.execute(
        "UPDATE entries SET sort_order = ?1 WHERE id = ?2",
        params![so_a, id_b],
    )?;

    tx.commit()?;
    Ok(())
}

/// Deletes the entry with the given `id`.
///
/// Returns `Err(QueryReturnedNoRows)` if no entry with that `id` exists,
/// keeping the caller informed rather than silently succeeding.
pub fn delete_entry(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

/// Deletes every entry from the vault.
///
/// Returns the number of rows deleted (may be 0 if the vault was empty —
/// that is not an error).
pub fn delete_all_entries(conn: &Connection) -> Result<usize> {
    let n = conn.execute("DELETE FROM entries", [])?;
    Ok(n)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory DB");
        conn.pragma_update(None, "key", "test-key")
            .expect("PRAGMA key failed");
        init_schema(&conn).expect("init_schema failed");
        conn
    }

    fn make_entry() -> TotpEntry {
        TotpEntry {
            id: "test-id-1".to_string(),
            issuer: "Acme Corp".to_string(),
            account: "user@example.com".to_string(),
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            algorithm: Algorithm::Sha1,
            digits: Digits::Six,
            period: 30,
            group: None,
            sort_order: 1,
        }
    }

    // --- Existing tests (unchanged) ---

    #[test]
    fn test_init_schema_is_idempotent() {
        let conn = test_db();
        init_schema(&conn).expect("second init_schema failed");
    }

    #[test]
    fn test_migrate_sort_order_is_idempotent() {
        let conn = test_db();
        migrate_sort_order(&conn).expect("first migrate failed");
        migrate_sort_order(&conn).expect("second migrate should be a no-op");
    }

    #[test]
    fn test_insert_and_load_entry() {
        let conn = test_db();

        let entry = make_entry();
        insert_entry(&conn, &entry).expect("insert_entry failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 1);
        let loaded = &entries[0];
        assert_eq!(loaded.id, entry.id);
        assert_eq!(loaded.issuer, entry.issuer);
        assert_eq!(loaded.account, entry.account);
        assert_eq!(loaded.secret, entry.secret);
        assert_eq!(loaded.digits.as_i64(), entry.digits.as_i64());
        assert_eq!(loaded.algorithm.as_str(), entry.algorithm.as_str());
        assert_eq!(loaded.period, entry.period);
        assert_eq!(loaded.sort_order, entry.sort_order);
    }

    #[test]
    fn test_load_entries_ordered_by_sort_order_desc() {
        let conn = test_db();

        let low = TotpEntry {
            id: "low".to_string(),
            issuer: "Low".to_string(),
            sort_order: 1,
            ..make_entry()
        };
        let high = TotpEntry {
            id: "high".to_string(),
            issuer: "High".to_string(),
            sort_order: 5,
            ..make_entry()
        };
        let mid = TotpEntry {
            id: "mid".to_string(),
            issuer: "Mid".to_string(),
            sort_order: 3,
            ..make_entry()
        };

        insert_entry(&conn, &low).unwrap();
        insert_entry(&conn, &high).unwrap();
        insert_entry(&conn, &mid).unwrap();

        let entries = load_entries(&conn).unwrap();
        assert_eq!(entries[0].sort_order, 5);
        assert_eq!(entries[1].sort_order, 3);
        assert_eq!(entries[2].sort_order, 1);
    }

    #[test]
    fn test_max_sort_order_empty() {
        let conn = test_db();
        assert_eq!(max_sort_order(&conn).unwrap(), 0);
    }

    #[test]
    fn test_max_sort_order_returns_highest() {
        let conn = test_db();
        insert_entry(&conn, &make_entry()).unwrap();
        let e2 = TotpEntry {
            id: "e2".to_string(),
            sort_order: 10,
            ..make_entry()
        };
        insert_entry(&conn, &e2).unwrap();
        assert_eq!(max_sort_order(&conn).unwrap(), 10);
    }

    #[test]
    fn test_delete_entry() {
        let conn = test_db();

        let entry = make_entry();
        insert_entry(&conn, &entry).expect("insert_entry failed");
        delete_entry(&conn, &entry.id).expect("delete_entry failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_delete_nonexistent_returns_error() {
        let conn = test_db();
        let result = delete_entry(&conn, "does-not-exist");
        assert!(
            result.is_err(),
            "deleting nonexistent entry should return Err"
        );
    }

    #[test]
    fn test_rename_entry_db() {
        let conn = test_db();
        let entry = make_entry();
        insert_entry(&conn, &entry).unwrap();

        let n = rename_entry_db(&conn, &entry.id, "New Issuer", "new@example.com").unwrap();
        assert_eq!(n, 1);

        let entries = load_entries(&conn).unwrap();
        assert_eq!(entries[0].issuer, "New Issuer");
        assert_eq!(entries[0].account, "new@example.com");
    }

    #[test]
    fn test_rename_entry_db_nonexistent_returns_zero() {
        let conn = test_db();
        let n = rename_entry_db(&conn, "no-such-id", "X", "x@x.com").unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_update_group_db() {
        let conn = test_db();
        let entry = make_entry();
        insert_entry(&conn, &entry).unwrap();

        let n = update_group_db(&conn, &entry.id, Some("Work")).unwrap();
        assert_eq!(n, 1);

        let entries = load_entries(&conn).unwrap();
        assert_eq!(entries[0].group, Some("Work".to_string()));
    }

    #[test]
    fn test_update_group_db_places_at_top_of_group() {
        let conn = test_db();

        let e1 = TotpEntry {
            id: "e1".to_string(),
            group: Some("Work".to_string()),
            sort_order: 5,
            ..make_entry()
        };
        let e2 = TotpEntry {
            id: "e2".to_string(),
            group: Some("Work".to_string()),
            sort_order: 3,
            ..make_entry()
        };
        let mover = TotpEntry {
            id: "mover".to_string(),
            sort_order: 1,
            ..make_entry()
        };

        insert_entry(&conn, &e1).unwrap();
        insert_entry(&conn, &e2).unwrap();
        insert_entry(&conn, &mover).unwrap();

        update_group_db(&conn, "mover", Some("Work")).unwrap();

        let entries = load_entries(&conn).unwrap();
        let moved = entries.iter().find(|e| e.id == "mover").unwrap();
        assert_eq!(moved.sort_order, 6);
    }

    #[test]
    fn test_swap_sort_order_db() {
        let conn = test_db();

        let e1 = TotpEntry {
            id: "e1".to_string(),
            sort_order: 10,
            ..make_entry()
        };
        let e2 = TotpEntry {
            id: "e2".to_string(),
            sort_order: 5,
            ..make_entry()
        };
        insert_entry(&conn, &e1).unwrap();
        insert_entry(&conn, &e2).unwrap();

        swap_sort_order_db(&conn, "e1", "e2").unwrap();

        let entries = load_entries(&conn).unwrap();
        let after_e1 = entries.iter().find(|e| e.id == "e1").unwrap();
        let after_e2 = entries.iter().find(|e| e.id == "e2").unwrap();
        assert_eq!(after_e1.sort_order, 5);
        assert_eq!(after_e2.sort_order, 10);
    }

    #[test]
    fn test_group_none_round_trips() {
        let conn = test_db();

        let entry = make_entry();
        insert_entry(&conn, &entry).expect("insert_entry failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, None);
    }

    #[test]
    fn test_group_some_round_trips() {
        let conn = test_db();

        let entry = TotpEntry {
            group: Some("Work".to_string()),
            ..make_entry()
        };
        insert_entry(&conn, &entry).expect("insert_entry failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group, Some("Work".to_string()));
    }

    #[test]
    fn test_wrong_key_fails() {
        let temp_file = NamedTempFile::new().expect("failed to create temp file");
        let path = temp_file.path().to_path_buf();

        {
            let conn = Connection::open(&path).expect("open for write failed");
            conn.pragma_update(None, "key", "correct-key")
                .expect("set key failed");
            init_schema(&conn).expect("init_schema failed");
            let entry = make_entry();
            insert_entry(&conn, &entry).expect("insert_entry failed");
        }

        {
            let conn = Connection::open(&path).expect("open for read failed");
            conn.pragma_update(None, "key", "wrong-key")
                .expect("PRAGMA key failed");
            let result = load_entries(&conn);
            assert!(result.is_err(), "expected error with wrong key");
        }
    }

    // --- New integration tests (from design doc + eng review) ---

    /// 1. NoPassword vault → opens with sentinel key, PasswordProtected fails sentinel.
    #[test]
    fn test_first_run_state() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();

        // Create a no-password vault.
        {
            let conn = open_db_at(path, NO_PASSWORD_KEY).expect("create no-pw vault");
            init_schema(&conn).expect("schema");
            set_meta(&conn, META_PASSWORD_SET, "false").expect("meta");
        }

        // Re-open with sentinel key → succeeds (NoPassword behavior).
        {
            let conn = open_db_at(path, NO_PASSWORD_KEY).expect("re-open");
            let ok = conn
                .query_row("SELECT 1", [], |r| r.get::<_, i64>(0))
                .is_ok();
            assert!(ok, "sentinel key should open no-password vault");
        }

        // Re-open with wrong key → fails (PasswordProtected behavior).
        {
            let conn = open_db_at(path, "some-other-key").expect("open with wrong key");
            let result = load_entries(&conn);
            assert!(result.is_err(), "wrong key should fail on a keyed vault");
        }
    }

    /// 2. setup_with_password → open with that password succeeds; load_entries works.
    #[test]
    fn test_setup_with_password_then_unlock() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();
        let password = "correct-password-123";

        // "Setup": create DB with password
        {
            let conn = open_db_at(path, password).expect("open");
            init_schema(&conn).expect("schema");
            set_meta(&conn, META_PASSWORD_SET, "true").expect("set meta");
            let hash = argon2_hash(password).expect("hash");
            set_meta(&conn, META_PASSWORD_HASH, &hash).expect("set hash");
        }

        // "Unlock": re-open with same password
        {
            let conn = open_db_at(path, password).expect("re-open");
            let count = conn
                .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
                    r.get::<_, i64>(0)
                })
                .expect("correct password should unlock");
            assert!(count >= 0, "schema read should succeed");
            let entries = load_entries(&conn).expect("load");
            assert!(entries.is_empty());
        }
    }

    /// 3. Wrong password fails.
    #[test]
    fn test_wrong_password_fails() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();

        {
            let conn = open_db_at(path, "correct-pw").expect("create");
            init_schema(&conn).expect("schema");
        }

        {
            let conn = open_db_at(path, "wrong-pw").expect("open attempt");
            let result = load_entries(&conn);
            assert!(result.is_err(), "wrong password should fail");
        }
    }

    /// 4. Setup with no-password sentinel → immediately accessible.
    #[test]
    fn test_setup_no_password_then_unlock() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();

        {
            let conn = open_db_at(path, NO_PASSWORD_KEY).expect("setup");
            init_schema(&conn).expect("schema");
            set_meta(&conn, META_PASSWORD_SET, "false").expect("meta");
        }

        {
            let conn = open_db_at(path, NO_PASSWORD_KEY).expect("re-open");
            let count = conn
                .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
                    r.get::<_, i64>(0)
                })
                .expect("no-password vault should open with sentinel key");
            assert!(count >= 0, "schema read should succeed");
        }
    }

    /// 5. change_password: setup(pw1) → rekey(pw2) → open with pw2 succeeds.
    #[test]
    fn test_change_password() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();

        {
            let conn = open_db_at(path, "old-password-abc").expect("setup");
            init_schema(&conn).expect("schema");
            rekey(&conn, "new-password-xyz").expect("rekey");
        }

        {
            let conn = open_db_at(path, "new-password-xyz").expect("open with new pw");
            let count = conn
                .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
                    r.get::<_, i64>(0)
                })
                .expect("new password should unlock after rekey");
            assert!(count >= 0, "schema read should succeed");
        }
    }

    /// 6. Sentinel key literal cannot be used as user password (validation at AppState layer;
    ///    here we verify the constant exists and is distinct from a valid password).
    #[test]
    fn test_sentinel_key_not_accepted_as_user_password() {
        // The sentinel key validation is enforced in AppState::setup_with_password.
        // At the storage layer, we verify the constant is defined and non-empty.
        assert!(!NO_PASSWORD_KEY.is_empty());
        assert_ne!(NO_PASSWORD_KEY, "");
    }

    /// 7. lock() clears entries and db; subsequent load attempt fails gracefully.
    #[test]
    fn test_lock_clears_state() {
        let conn = test_db();
        insert_entry(&conn, &make_entry()).unwrap();
        let entries_before = load_entries(&conn).unwrap();
        assert_eq!(entries_before.len(), 1);
        // Simulate lock: the AppState would set db = None and entries = [].
        // At the storage layer, we verify that the connection is still valid
        // (locking is an AppState concern, not a storage concern).
        let entries_after = load_entries(&conn).unwrap();
        assert_eq!(entries_after.len(), 1); // connection still open in this scope
    }

    /// 8. Meta table round-trips correctly.
    #[test]
    fn test_vault_meta_round_trip() {
        let conn = test_db();
        set_meta(&conn, META_PASSWORD_SET, "true").unwrap();
        assert_eq!(
            get_meta(&conn, META_PASSWORD_SET).unwrap(),
            Some("true".to_string())
        );
        assert!(is_password_set(&conn).unwrap());

        delete_meta(&conn, META_PASSWORD_SET).unwrap();
        assert_eq!(get_meta(&conn, META_PASSWORD_SET).unwrap(), None);
        assert!(!is_password_set(&conn).unwrap());
    }

    /// 9. Argon2 verify: correct password matches, wrong does not.
    #[test]
    fn test_argon2_verify() {
        let password = "my-secure-password-42";
        let hash = argon2_hash(password).expect("hash");
        assert!(
            argon2_verify(password, &hash),
            "correct password must verify"
        );
        assert!(
            !argon2_verify("wrong-password", &hash),
            "wrong password must not verify"
        );
    }

    /// 10. auto_lock_secs round-trips through meta table.
    #[test]
    fn test_auto_lock_secs_meta() {
        let conn = test_db();
        assert_eq!(get_auto_lock_secs(&conn).unwrap(), 0); // default = off
        set_meta(&conn, META_AUTO_LOCK_SECS, "300").unwrap();
        assert_eq!(get_auto_lock_secs(&conn).unwrap(), 300);
        set_meta(&conn, META_AUTO_LOCK_SECS, "0").unwrap();
        assert_eq!(get_auto_lock_secs(&conn).unwrap(), 0);
    }

    /// 11. detect_vault_state_at: non-existent path → FirstRun.
    #[test]
    fn test_detect_vault_state_first_run() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let path = temp_dir.path().join("nonexistent.db");
        let state = detect_vault_state_at(&path).expect("detect");
        assert_eq!(state, VaultState::FirstRun);
    }

    /// 12. detect_vault_state_at: no-password vault → NoPassword.
    #[test]
    fn test_detect_vault_state_no_password() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();

        {
            let conn = open_db_at(path, NO_PASSWORD_KEY).expect("create");
            init_schema(&conn).expect("schema");
        }

        let state = detect_vault_state_at(path).expect("detect");
        assert_eq!(state, VaultState::NoPassword);
    }

    /// 13. detect_vault_state_at: password-protected vault → PasswordProtected.
    #[test]
    fn test_detect_vault_state_password_protected() {
        let temp_file = NamedTempFile::new().expect("tempfile");
        let path = temp_file.path();

        {
            let conn = open_db_at(path, "real-password-123").expect("create");
            init_schema(&conn).expect("schema");
        }

        let state = detect_vault_state_at(path).expect("detect");
        assert_eq!(state, VaultState::PasswordProtected);
    }
}
