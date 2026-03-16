use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rusqlite::{Connection, Result, params};

use crate::models::totp::{Algorithm, Digits, TotpEntry};

static DB: Mutex<Option<Connection>> = Mutex::new(None);
const DB_KEY: &str = "brew-vault-hardcoded-key";

pub fn db_path() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine data directory")
        .join("Brew Vault")
        .join("vault.db")
}

pub fn open_db(key: &str) -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("could not create data directory");
    }
    let conn = Connection::open(&path)?;
    conn.execute_batch(&format!("PRAGMA key = '{}';", key))?;
    Ok(conn)
}

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

pub fn init() -> Result<()> {
    let conn = open_db(DB_KEY)?;
    init_schema(&conn)?;
    let mut guard = DB.lock().expect("DB mutex poisoned");
    *guard = Some(conn);
    Ok(())
}

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

pub fn delete_entry(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}

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
