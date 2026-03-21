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
    conn.pragma_update(None, "key", key)?;
    Ok(conn)
}

/// Creates the `entries` table if it does not already exist.
///
/// Includes the `sort_order` column. Safe to call multiple times — uses
/// `CREATE TABLE IF NOT EXISTS`. For existing databases that already have
/// the table without `sort_order`, call [`migrate_sort_order`] afterwards.
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
        .map(|n| n > 0)
        .unwrap_or(false);

    if !has_column {
        conn.execute_batch(
            "ALTER TABLE entries ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
             UPDATE entries SET sort_order = rowid WHERE sort_order = 0;",
        )?;
    }
    Ok(())
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

/// Opens the vault database with the hardcoded key, initialises the schema,
/// and runs any pending migrations.
///
/// Returns the ready-to-use [`Connection`]. Callers are responsible for
/// storing it (typically in [`AppState`]).
pub fn open_and_init() -> Result<Connection> {
    let conn = open_db(DB_KEY)?;
    init_schema(&conn)?;
    migrate_sort_order(&conn)?;
    Ok(conn)
}

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
/// Unlike the removed `save_entry`, this uses a plain `INSERT` — it will fail
/// if an entry with the same `id` already exists.
pub fn insert_entry(conn: &Connection, entry: &TotpEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO entries
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
    )?;
    Ok(())
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

        // Two existing Work entries with sort_order 5 and 3
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
        // Should be max(5, 3) + 1 = 6
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

        let entry = make_entry(); // group: None
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
}
