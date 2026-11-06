//! R1-06: a save is a complete new file, checked and then swapped in. An
//! injected failure at each step leaves the previous file byte for byte
//! intact and no temporary file behind.

mod common;
use std::error::Error;
use std::fs;

use common::{bytes, folder, names_in, other_project, project};
use diagram_storage::error::FailPoint;
use diagram_storage::save::save_with_fault;
use diagram_storage::{SaveError, load, save};

const STEPS: [FailPoint; 5] = [
    FailPoint::Create,
    FailPoint::Write,
    FailPoint::Flush,
    FailPoint::Verify,
    FailPoint::Replace,
];

#[test]
fn a_failure_at_each_step_should_keep_the_previous_file_intact() -> Result<(), Box<dyn Error>> {
    for step in STEPS {
        let (dir, path) = folder()?;
        save(&path, "Shop", &project())?;
        let before = bytes(&path);

        let outcome = save_with_fault(&path, "Shop", &other_project(), Some(step));

        assert!(
            matches!(outcome, Err(SaveError::Injected(failed)) if failed == step),
            "{step:?}: {outcome:?}"
        );
        assert_eq!(bytes(&path), before, "{step:?} changed the old file");
        assert_eq!(names_in(dir.path()), ["shop.dgram"], "{step:?} left a file");
        assert_eq!(load(&path)?.document, project(), "{step:?}");
    }
    Ok(())
}

#[test]
fn a_failed_first_save_should_leave_nothing_behind() -> Result<(), Box<dyn Error>> {
    for step in STEPS {
        let (dir, path) = folder()?;

        let outcome = save_with_fault(&path, "Shop", &project(), Some(step));

        assert!(outcome.is_err(), "{step:?}");
        assert!(!path.exists(), "{step:?} created the project file");
        assert!(names_in(dir.path()).is_empty(), "{step:?} left a file");
    }
    Ok(())
}

#[test]
fn a_successful_save_should_replace_the_file_and_leave_no_temporary_file()
-> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;

    save(&path, "Shop v2", &other_project())?;

    let loaded = load(&path)?;
    assert_eq!(loaded.name, "Shop v2");
    assert_eq!(loaded.document, other_project());
    assert_eq!(names_in(dir.path()), ["shop.dgram"]);
    Ok(())
}

#[test]
fn saving_into_a_missing_folder_should_fail_with_a_clear_cause() -> Result<(), Box<dyn Error>> {
    let (dir, _) = folder()?;
    let path = dir.path().join("no-such-folder").join("shop.dgram");

    let outcome = save(&path, "Shop", &project());

    assert!(
        matches!(outcome, Err(SaveError::Create { .. })),
        "{outcome:?}"
    );
    assert!(outcome.is_err_and(|error| error.to_string().contains("could not create")));
    Ok(())
}

#[test]
fn a_target_that_cannot_be_replaced_should_fail_typed_and_clean_up() -> Result<(), Box<dyn Error>> {
    // A folder where the project file should go: the final swap cannot succeed.
    let (dir, path) = folder()?;
    fs::create_dir(&path)?;

    let outcome = save(&path, "Shop", &project());

    assert!(
        matches!(outcome, Err(SaveError::Replace { .. })),
        "{outcome:?}"
    );
    assert_eq!(
        names_in(dir.path()),
        ["shop.dgram"],
        "only the folder remains"
    );
    assert!(path.is_dir());
    Ok(())
}

#[test]
fn an_invalid_document_should_never_be_written() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let before = bytes(&path);
    let mut broken = project();
    // A view that shows a class that does not exist.
    if let Some(customer) = common::element(&broken, "Customer") {
        broken.elements.remove(&customer);
    }

    let outcome = save(&path, "Shop", &broken);

    assert!(matches!(outcome, Err(SaveError::Invalid(_))), "{outcome:?}");
    assert!(outcome.is_err_and(|error| error.to_string().contains("breaks the project rules")));
    assert_eq!(bytes(&path), before);
    assert_eq!(names_in(dir.path()), ["shop.dgram"]);
    Ok(())
}

#[test]
fn a_save_while_another_connection_reads_the_file_should_succeed_or_fail_cleanly()
-> Result<(), Box<dyn Error>> {
    use rusqlite::{Connection, OpenFlags};
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let before = bytes(&path);
    // Another reader holds the file open with a read transaction.
    let reader = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    reader.execute_batch("BEGIN; SELECT count(*) FROM elements;")?;

    let outcome = save(&path, "Shop", &other_project());

    match outcome {
        Ok(()) => {
            reader.execute_batch("COMMIT;")?;
            drop(reader);
            assert_eq!(load(&path)?.document, other_project());
        }
        Err(error) => {
            assert!(matches!(error, SaveError::Replace { .. }), "{error:?}");
            drop(reader);
            assert_eq!(bytes(&path), before, "a refused swap leaves the old file");
        }
    }
    assert_eq!(
        names_in(dir.path()),
        ["shop.dgram"],
        "no temporary file remains"
    );
    Ok(())
}

#[test]
fn a_project_from_a_newer_version_should_never_be_overwritten() -> Result<(), Box<dyn Error>> {
    use rusqlite::Connection;
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let writer = Connection::open(&path)?;
    writer.execute_batch("PRAGMA user_version = 9;")?;
    writer.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);

    let outcome = save(&path, "Shop", &other_project());

    assert!(
        matches!(outcome, Err(SaveError::NewerTarget { found: 9, .. })),
        "{outcome:?}"
    );
    assert_eq!(bytes(&path), before);
    assert_eq!(names_in(dir.path()), ["shop.dgram"]);
    Ok(())
}

#[test]
fn a_file_that_is_not_a_project_should_not_be_replaced_silently() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    fs::write(&path, "my shopping list, not a database\n".repeat(10))?;
    let before = bytes(&path);

    let outcome = save(&path, "Shop", &project());

    assert!(
        matches!(outcome, Err(SaveError::ForeignTarget { .. })),
        "{outcome:?}"
    );
    assert_eq!(bytes(&path), before);
    assert_eq!(names_in(dir.path()), ["shop.dgram"]);
    Ok(())
}

#[test]
fn an_empty_file_and_a_damaged_project_should_be_replaceable() -> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    fs::write(&path, b"")?;
    save(&path, "Shop", &project())?;
    assert_eq!(load(&path)?.document, project());

    // Damage the project (our header, cut-off body): saving repairs it.
    let whole = bytes(&path);
    fs::write(&path, &whole[..whole.len() / 2])?;
    assert!(load(&path).is_err());
    save(&path, "Shop", &other_project())?;
    assert_eq!(load(&path)?.document, other_project());
    Ok(())
}

#[test]
fn leftovers_of_an_interrupted_save_should_be_removed_without_touching_the_project()
-> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let before = bytes(&path);
    // What a killed process leaves behind, and files that are not ours.
    fs::write(dir.path().join(".shop.dgram.saving-4242-0"), b"half a file")?;
    fs::write(dir.path().join(".shop.dgram.saving-4242-1"), b"another")?;
    fs::write(
        dir.path().join(".other.dgram.saving-1-0"),
        b"belongs to another project",
    )?;
    fs::write(dir.path().join("notes.txt"), b"keep")?;

    let removed = diagram_storage::save::remove_orphans(&path)?;

    assert_eq!(removed, 2);
    assert_eq!(bytes(&path), before);
    assert_eq!(
        load(&path)?.document,
        project(),
        "the old project still loads"
    );
    assert_eq!(
        names_in(dir.path()),
        [".other.dgram.saving-1-0", "notes.txt", "shop.dgram"]
    );
    Ok(())
}

#[test]
fn a_saved_file_should_be_a_single_self_contained_file() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;

    assert_eq!(
        names_in(dir.path()),
        ["shop.dgram"],
        "no -journal, -wal or -shm"
    );
    use rusqlite::{Connection, OpenFlags};
    let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mode: String = connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    assert_eq!(mode.to_lowercase(), "delete");
    Ok(())
}

#[test]
fn a_foreign_database_in_wal_mode_should_not_be_replaced_or_touched() -> Result<(), Box<dyn Error>>
{
    use rusqlite::Connection;
    let (dir, path) = folder()?;
    let foreign = Connection::open(&path)?;
    foreign.execute_batch("CREATE TABLE notes (text TEXT);")?;
    let _mode: String = foreign.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    foreign.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);
    let names_before = names_in(dir.path());

    let outcome = save(&path, "Shop", &project());

    assert!(
        matches!(outcome, Err(SaveError::ForeignTarget { .. })),
        "{outcome:?}"
    );
    assert_eq!(bytes(&path), before);
    assert_eq!(
        names_in(dir.path()),
        names_before,
        "no leftovers or sidecars"
    );
    Ok(())
}

#[test]
fn a_truncated_foreign_database_should_not_be_taken_for_a_damaged_project()
-> Result<(), Box<dyn Error>> {
    use rusqlite::Connection;
    let (_dir, path) = folder()?;
    let foreign = Connection::open(&path)?;
    foreign.execute_batch("CREATE TABLE notes (text TEXT); INSERT INTO notes VALUES ('a');")?;
    foreign.close().map_err(|(_, error)| error)?;
    let whole = bytes(&path);
    fs::write(&path, &whole[..whole.len().min(150)])?;
    let before = bytes(&path);

    let outcome = save(&path, "Shop", &project());

    assert!(
        matches!(outcome, Err(SaveError::ForeignTarget { .. })),
        "{outcome:?}"
    );
    assert_eq!(bytes(&path), before);
    Ok(())
}

#[test]
fn a_journal_or_log_next_to_the_target_should_stop_the_save_untouched() -> Result<(), Box<dyn Error>>
{
    for suffix in ["-journal", "-wal", "-shm"] {
        let (dir, path) = folder()?;
        save(&path, "Shop", &project())?;
        let sidecar = dir.path().join(format!("shop.dgram{suffix}"));
        fs::write(&sidecar, [1_u8])?;
        let before = bytes(&path);
        let names_before = names_in(dir.path());

        let outcome = save(&path, "Shop", &other_project());

        assert!(
            matches!(outcome, Err(SaveError::SidecarsPresent { .. })),
            "{suffix}: {outcome:?}"
        );
        assert!(outcome.is_err_and(|error| error.to_string().contains("journal")));
        assert_eq!(bytes(&path), before, "{suffix}");
        assert_eq!(
            names_in(dir.path()),
            names_before,
            "{suffix}: nothing added or removed"
        );
    }
    Ok(())
}
