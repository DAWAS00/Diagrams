//! R0-03: bundled SQLite compiles, links and round-trips data through a real
//! temporary database file without a separately installed SQLite runtime.

use std::error::Error;
use std::fs;

use rusqlite::{Connection, OpenFlags, ffi};

#[test]
fn linked_sqlite_should_be_the_bundled_build() {
    assert_eq!(rusqlite::version_number(), ffi::SQLITE_VERSION_NUMBER);
    assert!(diagram_storage::sqlite_version().starts_with("3."));
}

#[test]
fn temporary_database_should_write_close_reopen_read_and_delete() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("smoke.sqlite3");

    let writer = Connection::open(&path)?;
    writer.execute_batch("CREATE TABLE smoke (id INTEGER PRIMARY KEY, name TEXT NOT NULL);")?;
    writer.execute(
        "INSERT INTO smoke (name) VALUES (?1), (?2)",
        ["Customer", "  Customer نموذج  "],
    )?;
    writer.close().map_err(|(_, error)| error)?;

    let reader = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = reader.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    let names = reader
        .prepare("SELECT name FROM smoke ORDER BY id")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    reader.close().map_err(|(_, error)| error)?;

    assert_eq!(integrity, "ok");
    assert_eq!(names, ["Customer", "  Customer نموذج  "]);

    let leftovers = fs::read_dir(directory.path())?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        leftovers,
        ["smoke.sqlite3"],
        "journal or WAL file left behind"
    );

    // Deleting succeeds on Windows only when every handle has been released.
    fs::remove_file(&path)?;
    assert!(!path.exists());
    directory.close()?;
    Ok(())
}
