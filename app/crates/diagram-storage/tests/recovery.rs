//! R3-03 (storage half): recovery snapshots are atomic, listed newest first,
//! and recovered through the same checked load.

mod common;
use std::error::Error;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

use common::{names_in, other_project, project};
use diagram_storage::LoadError;
use diagram_storage::recovery::{
    SNAPSHOT_EXTENSION, discard, list_snapshots, recover, write_snapshot,
};

#[test]
fn the_recovered_document_should_equal_the_last_snapshot_written() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;

    let path = write_snapshot(folder.path(), "shop", "Shop", &project())?;
    let again = write_snapshot(folder.path(), "shop", "Shop (edited)", &other_project())?;

    assert_eq!(path, again, "one snapshot per project key");
    let recovered = recover(&path)?;
    assert_eq!(recovered.name, "Shop (edited)");
    assert_eq!(recovered.document, other_project());
    assert_eq!(
        names_in(folder.path()).len(),
        1,
        "the older snapshot was replaced"
    );
    Ok(())
}

#[test]
fn snapshots_should_list_newest_first_with_their_project_names() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    write_snapshot(folder.path(), "older", "Older project", &project())?;
    sleep(Duration::from_millis(1100));
    write_snapshot(folder.path(), "newer", "Newer project", &other_project())?;

    let entries = list_snapshots(folder.path())?;

    let names: Vec<String> = entries
        .iter()
        .filter_map(|entry| entry.info.as_ref().ok().cloned())
        .collect();
    assert_eq!(names, ["Newer project", "Older project"]);
    Ok(())
}

#[test]
fn a_damaged_snapshot_should_be_listed_with_its_error_and_not_hide_the_others()
-> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    write_snapshot(folder.path(), "good", "Good", &project())?;
    fs::write(
        folder.path().join(format!("bad.{SNAPSHOT_EXTENSION}")),
        "not a database at all, only words\n".repeat(30),
    )?;
    fs::write(folder.path().join("notes.txt"), "ignored: wrong extension")?;

    let entries = list_snapshots(folder.path())?;

    assert_eq!(entries.len(), 2, "the text file is not a snapshot");
    assert_eq!(entries.iter().filter(|entry| entry.info.is_ok()).count(), 1);
    assert!(
        entries
            .iter()
            .any(|entry| matches!(entry.info, Err(LoadError::NotAProject { .. })))
    );
    Ok(())
}

#[test]
fn discarding_should_remove_only_that_snapshot() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    let keep = write_snapshot(folder.path(), "keep", "Keep", &project())?;
    let drop_me = write_snapshot(folder.path(), "drop", "Drop", &other_project())?;

    discard(folder.path(), &drop_me)?;

    assert!(keep.exists());
    assert!(!drop_me.exists());
    assert_eq!(list_snapshots(folder.path())?.len(), 1);
    Ok(())
}

#[test]
fn a_missing_recovery_folder_should_list_as_empty() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;

    let entries = list_snapshots(&folder.path().join("never-created"))?;

    assert!(entries.is_empty());
    Ok(())
}

#[test]
fn a_hostile_key_should_stay_inside_the_recovery_folder() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    let inner = folder.path().join("recovery");

    let path = write_snapshot(&inner, "../../escape", "Shop", &project())?;

    assert_eq!(path.parent(), Some(inner.as_path()));
    assert!(!folder.path().join("escape").exists());
    assert_eq!(names_in(&inner).len(), 1);
    Ok(())
}

#[test]
fn a_failed_snapshot_write_should_keep_the_previous_snapshot() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    let path = write_snapshot(folder.path(), "shop", "Shop", &project())?;
    // An invalid document is refused by the save's own check.
    let mut broken = other_project();
    if let Some(customer) = common::element(&broken, "Client") {
        broken.elements.remove(&customer);
    }

    let outcome = write_snapshot(folder.path(), "shop", "Shop", &broken);

    assert!(outcome.is_err());
    assert_eq!(recover(&path)?.document, project());
    assert_eq!(names_in(folder.path()).len(), 1);
    Ok(())
}

#[test]
fn discard_should_refuse_anything_that_is_not_a_snapshot_in_the_folder()
-> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    let elsewhere = tempfile::tempdir()?;
    let outside = elsewhere
        .path()
        .join(format!("precious.{SNAPSHOT_EXTENSION}"));
    fs::write(&outside, b"not yours to delete")?;
    let notes = folder.path().join("notes.txt");
    fs::write(&notes, b"keep")?;
    let sub = folder.path().join(format!("folder.{SNAPSHOT_EXTENSION}"));
    fs::create_dir(&sub)?;

    assert!(
        discard(folder.path(), &outside).is_err(),
        "outside the folder"
    );
    assert!(discard(folder.path(), &notes).is_err(), "not a snapshot");
    assert!(
        discard(folder.path(), &sub).is_err(),
        "a folder, not a file"
    );
    assert!(outside.exists() && notes.exists() && sub.exists());
    Ok(())
}

#[test]
fn keys_that_differ_only_in_punctuation_should_keep_separate_snapshots()
-> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;

    let first = write_snapshot(folder.path(), "project:a", "First", &project())?;
    let second = write_snapshot(folder.path(), "project?a", "Second", &other_project())?;

    assert_ne!(first, second);
    assert_eq!(recover(&first)?.name, "First");
    assert_eq!(recover(&second)?.name, "Second");
    assert_eq!(recover(&first)?.document, project());
    assert_eq!(recover(&second)?.document, other_project());
    // The longest accepted key works (its temporary names stay short enough);
    // one byte more is refused.
    let longest = "k".repeat(48);
    assert!(write_snapshot(folder.path(), &longest, "x", &project()).is_ok());
    let too_long = "k".repeat(49);
    assert!(write_snapshot(folder.path(), &too_long, "x", &project()).is_err());
    Ok(())
}
