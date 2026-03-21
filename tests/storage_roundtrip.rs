//! Integration test: persist entries to a real encrypted file, reopen, and verify.

use brew_vault::{
    models::totp::{Algorithm, Digits, TotpEntry},
    storage::{init_schema, insert_entry, load_entries, open_db_at},
};
use tempfile::tempdir;

fn make_entry(n: u8) -> TotpEntry {
    TotpEntry {
        id: format!("entry-{n}"),
        issuer: format!("Issuer {n}"),
        account: format!("user{n}@example.com"),
        secret: "JBSWY3DPEHPK3PXP".to_string(),
        algorithm: Algorithm::Sha1,
        digits: Digits::Six,
        period: 30,
        group: None,
        sort_order: n as u64,
    }
}

#[test]
fn roundtrip_three_entries() {
    let dir = tempdir().expect("failed to create tempdir");
    let db_path = dir.path().join("vault.db");
    let key = "integration-test-key";

    // Write phase: save 3 entries and drop the connection.
    {
        let conn = open_db_at(&db_path, key).expect("open for write failed");
        init_schema(&conn).expect("init_schema failed");
        for n in 1..=3 {
            insert_entry(&conn, &make_entry(n)).expect("insert_entry failed");
        }
    }

    // Read phase: reopen with the same key.
    let conn = open_db_at(&db_path, key).expect("reopen failed");
    let entries = load_entries(&conn).expect("load_entries failed");

    assert_eq!(entries.len(), 3, "expected 3 entries after roundtrip");

    let mut ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, ["entry-1", "entry-2", "entry-3"]);

    for entry in &entries {
        let n: u8 = entry.id.trim_start_matches("entry-").parse().unwrap();
        let expected = make_entry(n);
        assert_eq!(entry.issuer, expected.issuer);
        assert_eq!(entry.account, expected.account);
        assert_eq!(entry.secret, expected.secret);
        assert_eq!(entry.algorithm.as_str(), expected.algorithm.as_str());
        assert_eq!(entry.digits.as_i64(), expected.digits.as_i64());
        assert_eq!(entry.period, expected.period);
        assert_eq!(entry.sort_order, expected.sort_order);
    }
}

#[test]
fn wrong_key_cannot_read_entries() {
    let dir = tempdir().expect("failed to create tempdir");
    let db_path = dir.path().join("vault.db");

    {
        let conn = open_db_at(&db_path, "correct-key").expect("open for write failed");
        init_schema(&conn).expect("init_schema failed");
        insert_entry(&conn, &make_entry(1)).expect("insert_entry failed");
    }

    let conn = open_db_at(&db_path, "wrong-key").expect("open with wrong key failed");
    assert!(
        load_entries(&conn).is_err(),
        "reading with wrong key must fail"
    );
}
