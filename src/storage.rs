//! Persistent, encrypted storage for BrewVault TOTP entries.
//!
//! All vault data is stored in a SQLCipher-encrypted SQLite database. The
//! module exposes a process-wide singleton connection ([`DB`]) that is
//! initialised once at startup via [`init`] and then accessed through the
//! [`with_db`] helper.
//!
//! # Encryption
//! The database key is currently hardcoded (`DB_KEY`). A master-password flow
//! is deferred to v2.

use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rusqlite::{Connection, Result, params};

use crate::models::totp::{Algorithm, Digits, TotpEntry};

/// Process-wide SQLCipher connection, initialised by [`init`].
static DB: Mutex<Option<Connection>> = Mutex::new(None);

/// Hardcoded encryption key used until a master-password flow is introduced.
const DB_KEY: &str = "brew-vault-hardcoded-key";

/// Returns the platform-appropriate path to the vault database file.
///
/// On macOS this resolves to
/// `~/Library/Application Support/Brew Vault/vault.db`.
pub fn db_path() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine data directory")
        .join("Brew Vault")
        .join("vault.db")
}

/// Opens (or creates) the SQLCipher database at [`db_path`] using `key`.
///
/// The parent directory is created automatically if it does not exist.
/// Returns an error if the file cannot be opened or if the key pragma fails.
pub fn open_db(key: &str) -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("could not create data directory");
    }
    let conn = Connection::open(&path)?;
    conn.execute_batch(&format!("PRAGMA key = '{}';", key))?;
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
            period    INTEGER NOT NULL DEFAULT 30
        );",
    )
}

/// Initialises the global database connection.
///
/// Opens the vault file with [`DB_KEY`], runs [`init_schema`], and stores the
/// connection in the [`DB`] static. Must be called once before any other
/// storage function is used (typically at the top of `main`).
pub fn init() -> Result<()> {
    let conn = open_db(DB_KEY)?;
    init_schema(&conn)?;
    let mut guard = DB.lock().expect("DB mutex poisoned");
    *guard = Some(conn);
    Ok(())
}

/// Loads all TOTP entries from the database.
///
/// Returns a [`Vec`] of [`TotpEntry`] in insertion order.
pub fn load_entries(conn: &Connection) -> Result<Vec<TotpEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, issuer, account, secret, algorithm, digits, period FROM entries",
    )?;
    let entries = stmt.query_map([], |row| {
        let algorithm_str: String = row.get(4)?;
        let digits_i64: i64 = row.get(5)?;
        let period: i64 = row.get(6)?;

        let algorithm = Algorithm::try_from(algorithm_str.as_str())
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, e.into()))?;
        let digits = Digits::try_from(digits_i64)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Integer, e.into()))?;

        Ok(TotpEntry {
            id: row.get(0)?,
            issuer: row.get(1)?,
            account: row.get(2)?,
            secret: row.get(3)?,
            algorithm,
            digits,
            period: period as u64,
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
        "INSERT OR REPLACE INTO entries (id, issuer, account, secret, algorithm, digits, period)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.id,
            entry.issuer,
            entry.account,
            entry.secret,
            entry.algorithm.as_str(),
            entry.digits.as_i64(),
            entry.period as i64,
        ],
    )?;
    Ok(())
}

/// Deletes the entry with the given `id`. No-ops silently if not found.
pub fn delete_entry(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}

/// Locks the global [`DB`] mutex and passes the connection to `f`.
///
/// # Panics
/// Panics if [`init`] has not been called or if the mutex is poisoned.
pub fn with_db<F, T>(f: F) -> T
where
    F: FnOnce(&Connection) -> T,
{
    let guard: MutexGuard<Option<Connection>> = DB.lock().expect("DB mutex poisoned");
    let conn = guard.as_ref().expect("DB not initialized");
    f(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory DB");
        conn.execute_batch(&format!("PRAGMA key = '{}';", "test-key"))
            .expect("PRAGMA key failed");
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
        }
    }

    #[test]
    fn test_init_schema_is_idempotent() {
        let conn = open_memory_db();
        init_schema(&conn).expect("first init_schema failed");
        init_schema(&conn).expect("second init_schema failed");
    }

    #[test]
    fn test_save_and_load_entry() {
        let conn = open_memory_db();
        init_schema(&conn).expect("init_schema failed");

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
        let conn = open_memory_db();
        init_schema(&conn).expect("init_schema failed");

        let entry = make_entry();
        save_entry(&conn, &entry).expect("save_entry failed");
        delete_entry(&conn, &entry.id).expect("delete_entry failed");

        let entries = load_entries(&conn).expect("load_entries failed");
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_wrong_key_fails() {
        let path = std::env::temp_dir().join("brew-vault-test-wrong-key.db");
        // Clean up any leftover file from a previous run.
        let _ = std::fs::remove_file(&path);

        {
            let conn = Connection::open(&path).expect("open for write failed");
            conn.execute_batch("PRAGMA key = 'correct-key';")
                .expect("set key failed");
            init_schema(&conn).expect("init_schema failed");
            let entry = make_entry();
            save_entry(&conn, &entry).expect("save_entry failed");
        }

        {
            let conn = Connection::open(&path).expect("open for read failed");
            conn.execute_batch("PRAGMA key = 'wrong-key';")
                .expect("PRAGMA key failed");
            let result = load_entries(&conn);
            assert!(result.is_err(), "expected error with wrong key");
        }

        let _ = std::fs::remove_file(&path);
    }
}
