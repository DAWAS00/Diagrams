//! R1-05: schema v1 is recorded in the file, and files that are not ours,
//! empty, or from a newer version are refused without being touched.

mod common;
use std::error::Error;
use std::fs;

use common::{bytes, folder, project};
use diagram_storage::schema::{APPLICATION_ID, SCHEMA_VERSION};
use diagram_storage::{LoadError, load, peek, save};
use rusqlite::{Connection, OpenFlags};

#[test]
fn a_saved_file_should_record_the_application_id_and_version() -> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    save(&path, "Shop", &project())?;

    let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let application: i32 = connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    assert_eq!(application, APPLICATION_ID);
    assert_eq!(version, SCHEMA_VERSION);
    assert_eq!(SCHEMA_VERSION, 1);
    let info = peek(&path)?;
    assert_eq!((info.name.as_str(), info.schema_version), ("Shop", 1));
    Ok(())
}

#[test]
fn a_newer_schema_should_be_refused_and_left_byte_for_byte_unchanged() -> Result<(), Box<dyn Error>>
{
    let (_dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let writer = Connection::open(&path)?;
    writer.execute_batch("PRAGMA user_version = 2;")?;
    writer.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);

    let loaded = load(&path);
    let peeked = peek(&path);

    assert!(matches!(
        loaded,
        Err(LoadError::NewerVersion {
            found: 2,
            supported: 1
        })
    ));
    assert!(matches!(peeked, Err(LoadError::NewerVersion { .. })));
    assert_eq!(bytes(&path), before, "a refused file is never rewritten");
    Ok(())
}

#[test]
fn the_newer_version_message_should_say_what_to_do_in_words() -> Result<(), Box<dyn Error>> {
    let error = LoadError::NewerVersion {
        found: 7,
        supported: 1,
    };

    let message = error.to_string();

    assert!(message.contains("newer version"));
    assert!(message.contains('7') && message.contains('1'));
    Ok(())
}

#[test]
fn a_plain_sqlite_file_should_not_be_taken_for_a_project() -> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    let connection = Connection::open(&path)?;
    connection.execute_batch("CREATE TABLE notes (id INTEGER PRIMARY KEY, text TEXT);")?;
    connection.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);

    let loaded = load(&path);

    assert!(matches!(loaded, Err(LoadError::NotAProject { .. })));
    assert_eq!(bytes(&path), before);
    Ok(())
}

#[test]
fn a_text_file_and_an_empty_file_should_not_be_projects() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    fs::write(
        &path,
        "this is not a database, just some words.\n".repeat(20),
    )?;
    let empty = dir.path().join("empty.dgram");
    fs::write(&empty, b"")?;

    assert!(matches!(load(&path), Err(LoadError::NotAProject { .. })));
    assert!(matches!(load(&empty), Err(LoadError::NotAProject { .. })));
    assert_eq!(bytes(&empty), Vec::<u8>::new(), "an empty file stays empty");
    Ok(())
}

#[test]
fn a_missing_file_should_be_not_found_and_not_created() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;

    let loaded = load(&path);

    assert!(matches!(loaded, Err(LoadError::NotFound { .. })));
    assert!(!path.exists(), "loading never creates a file");
    assert!(common::names_in(dir.path()).is_empty());
    Ok(())
}

#[test]
fn our_application_id_with_an_unknown_version_should_be_damaged_not_migrated()
-> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let writer = Connection::open(&path)?;
    writer.execute_batch("PRAGMA user_version = 0;")?;
    writer.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);

    let loaded = load(&path);

    assert!(matches!(loaded, Err(LoadError::Damaged { .. })));
    assert_eq!(bytes(&path), before);
    Ok(())
}
