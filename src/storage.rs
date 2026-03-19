//! Persistent, encrypted storage for BrewVault TOTP entries.
//!
//! All vault data is stored in a SQLCipher-encrypted SQLite database.
//! Callers obtain a [`Connection`] via [`open_and_init`] and store it in
//! [`AppState`]; there is no process-wide singleton here.
//!
//! # Encryption
//! The database key is currently hardcoded (`DB_KEY`). A master-password flow
//! is deferred to v2.

use std::path::PathBuf;

use rusqlite::{Connection, Result, params};

use crate::models::totp::{Algorithm, Digits, TotpEntry};

/// Hardcoded encryption key used until a master-password flow is introduced.
const DB_KEY: &str = "brew-vault-hardcoded-key";

/// Returns the platform-appropriate path to the vault database file.
///
/// Prefers `dirs::data_dir()` (e.g. `~/Library/Application Support` on macOS),
/// falling back to `dirs::home_dir()` if the former is unavailable.
/// Returns `None` only when neither directory can be resolved.
pub fn db_path() -> Option<PathBuf> {
    let base = dirs::data_dir().or_else(dirs::home_dir)?;
    Some(base.join("Brew Vault").join("vault.db"))
}

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
    // Use pragma_update so the key is passed as a bound parameter, not
    // interpolated into SQL — prevents injection if the key ever comes from
    // user input.
    conn.pragma_update(None, "key", key)?;
    Ok(conn)
}

/// Creates the `entries` table if it does not already exist.
///
/// Safe to call multiple times — uses `CREATE TABLE IF NOT EXISTS`.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id        TEXT PRIMARY KEY,
            issuer    TEXT NOT NULL,
            account   TEXT NOT NULL,
            secret    TEXT NOT NULL,
            algorithm TEXT NOT NULL DEFAULT 'SHA1',
            digits    INTEGER NOT NULL DEFAULT 6,
            period    INTEGER NOT NULL DEFAULT 30,
            `group`   TEXT DEFAULT NULL
        );",
    )
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

/// Opens the vault database with the hardcoded key and initialises the schema.
///
/// Returns the ready-to-use [`Connection`]. Callers are responsible for
/// storing it (typically in [`AppState`]).
pub fn open_and_init() -> Result<Connection> {
    let conn = open_db(DB_KEY)?;
    init_schema(&conn)?;
    Ok(conn)
}

/// Loads all TOTP entries from the database.
///
/// Returns a [`Vec`] of [`TotpEntry`] with no guaranteed ordering.
pub fn load_entries(conn: &Connection) -> Result<Vec<TotpEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, issuer, account, secret, algorithm, digits, period, `group` FROM entries",
    )?;
    let entries = stmt
        .query_map([], |row| {
            let algorithm_str: String = row.get(4)?;
            let digits_i64: i64 = row.get(5)?;
            let period: i64 = row.get(6)?;
            let group: Option<String> = row.get(7)?;

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
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(entries)
}

/// Inserts or replaces a TOTP entry in the database.
///
/// Uses `INSERT OR REPLACE` so calling this with an existing `entry.id`
/// acts as an upsert.
pub fn save_entry(conn: &Connection, entry: &TotpEntry) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO entries (id, issuer, account, secret, algorithm, digits, period, `group`)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            entry.id,
            entry.issuer,
            entry.account,
            entry.secret,
            entry.algorithm.as_str(),
            entry.digits.as_i64(),
            entry.period as i64,
            entry.group.as_deref(),
        ],
    )?;
    Ok(())
}

/// Deletes the entry with the given `id`. No-ops silently if not found.
pub fn delete_entry(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}

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
        }
    }

    #[test]
    fn test_init_schema_is_idempotent() {
        let conn = test_db();
        init_schema(&conn).expect("second init_schema failed");
    }

    #[test]
    fn test_save_and_load_entry() {
        let conn = test_db();

        let entry = make_entry();
        save_entry(&conn, &entry).expect("save_entry failed");

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
    }

    #[test]
    fn test_delete_entry() {
        let conn = test_db();

        let entry = make_entry();
        save_entry(&conn, &entry).expect("save_entry failed");
        delete_entry(&conn, &entry.id).expect("delete_entry failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_upsert_replaces_existing() {
        let conn = test_db();

        let entry = make_entry();
        save_entry(&conn, &entry).expect("first save failed");

        let updated = TotpEntry {
            issuer: "Updated Corp".to_string(),
            account: "updated@example.com".to_string(),
            ..make_entry()
        };
        save_entry(&conn, &updated).expect("upsert failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 1, "upsert must not add a duplicate row");
        assert_eq!(entries[0].issuer, "Updated Corp");
        assert_eq!(entries[0].account, "updated@example.com");
    }

    #[test]
    fn test_group_none_round_trips() {
        let conn = test_db();

        let entry = make_entry(); // group: None
        save_entry(&conn, &entry).expect("save_entry failed");

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
        save_entry(&conn, &entry).expect("save_entry failed");

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
            save_entry(&conn, &entry).expect("save_entry failed");
        }

        {
            let conn = Connection::open(&path).expect("open for read failed");
            conn.pragma_update(None, "key", "wrong-key")
                .expect("PRAGMA key failed");
            let result = load_entries(&conn);
            assert!(result.is_err(), "expected error with wrong key");
        }
    }
}
