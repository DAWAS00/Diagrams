//! Saving a project: a complete new file, checked, then swapped in.
//!
//! The project is written to a temporary file **in the same folder**, in one
//! transaction. The finished file is read back through the normal checked
//! load and compared with the document. Only then does it replace the
//! target with one rename (`std::fs::rename`, which on Windows replaces an
//! existing file atomically: the target is never deleted first, so it is
//! either the old project or the new one). A failure at any step removes the
//! temporary file and leaves the previous file exactly as it was.
//!
//! Durability: the written file is flushed (`sync_all`) before the swap. The
//! standard library cannot flush the folder entry itself, so **no power-loss
//! guarantee is claimed**: the swap is atomic against crashes and errors of
//! this program, not against loss of power.
//! A process killed mid-save leaves at most a `.<name>.saving-*` file, which
//! [`remove_orphans`] deletes when the project is next opened.
//!
//! The target is replaced only if it is a Diagram Studio project of this or
//! an older format, a damaged one, or an empty file; a newer-format project
//! or any other file is refused with a typed error.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use diagram_core::model::Document;
use diagram_core::validate::{Severity, check_document};
use rusqlite::{Connection, params};

use crate::dto::ProjectDto;
use crate::error::{FailPoint, SaveError};
use crate::load::{Identity, identity, load_checked};
use crate::schema::{APPLICATION_ID, CREATE_V1, SCHEMA_VERSION};

/// A temporary file that is removed unless it was published.
struct Temporary {
    path: PathBuf,
    published: bool,
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.path);
            let mut journal = self.path.clone().into_os_string();
            journal.push("-journal");
            let _ = fs::remove_file(PathBuf::from(journal));
        }
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let name = path.file_name().map_or_else(
        || String::from("project"),
        |name| name.to_string_lossy().into_owned(),
    );
    let unique = format!(
        ".{name}.saving-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from(&unique), |parent| parent.join(&unique))
}

fn write_error(error: &rusqlite::Error) -> SaveError {
    SaveError::Write(error.to_string())
}

fn write_rows(
    connection: &mut Connection,
    dto: &ProjectDto,
    fault: Option<FailPoint>,
) -> Result<(), SaveError> {
    // Policy (ADR-002): a plain rollback journal, full sync, foreign keys on
    // before the transaction. Each saved file is a self-contained snapshot, so
    // no write-ahead log and no sidecar files are ever published.
    let mode: String = connection
        .query_row("PRAGMA journal_mode = DELETE", [], |row| row.get(0))
        .map_err(|error| write_error(&error))?;
    if !mode.eq_ignore_ascii_case("delete") {
        return Err(SaveError::Write(format!(
            "the journal mode is {mode}, not delete"
        )));
    }
    connection
        .execute_batch(&format!(
            "PRAGMA synchronous = FULL; PRAGMA application_id = {APPLICATION_ID}; \
             PRAGMA user_version = {SCHEMA_VERSION}; PRAGMA foreign_keys = ON;"
        ))
        .map_err(|error| write_error(&error))?;
    let transaction = connection
        .transaction()
        .map_err(|error| write_error(&error))?;
    transaction
        .execute_batch(CREATE_V1)
        .map_err(|error| write_error(&error))?;
    transaction
        .execute(
            "INSERT INTO meta (key, value) VALUES ('project_name', ?1), ('next_id', ?2)",
            params![dto.name, dto.next_id.to_string()],
        )
        .map_err(|error| write_error(&error))?;
    for row in &dto.elements {
        transaction
            .execute(
                "INSERT INTO elements (id, name) VALUES (?1, ?2)",
                params![id(row.id)?, row.name],
            )
            .map_err(|error| write_error(&error))?;
    }
    for row in &dto.members {
        transaction
            .execute(
                "INSERT INTO members (id, element_id, position, kind, name, visibility, \
                 type_text, multiplicity, return_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id(row.id)?,
                    id(row.element_id)?,
                    row.position,
                    row.kind,
                    row.name,
                    row.visibility,
                    row.type_text,
                    row.multiplicity,
                    row.return_type
                ],
            )
            .map_err(|error| write_error(&error))?;
    }
    for row in &dto.parameters {
        transaction
            .execute(
                "INSERT INTO parameters (member_id, position, name, type_text) \
                 VALUES (?1, ?2, ?3, ?4)",
                params![id(row.member_id)?, row.position, row.name, row.type_text],
            )
            .map_err(|error| write_error(&error))?;
    }
    for row in &dto.relations {
        transaction
            .execute(
                "INSERT INTO relations (id, kind, source, target, source_role, \
                 source_multiplicity, target_role, target_multiplicity) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id(row.id)?,
                    row.kind,
                    id(row.source)?,
                    id(row.target)?,
                    row.source_role,
                    row.source_multiplicity,
                    row.target_role,
                    row.target_multiplicity
                ],
            )
            .map_err(|error| write_error(&error))?;
    }
    for row in &dto.diagrams {
        transaction
            .execute(
                "INSERT INTO diagrams (id, position, name, notation) VALUES (?1, ?2, ?3, ?4)",
                params![id(row.id)?, row.position, row.name, row.notation],
            )
            .map_err(|error| write_error(&error))?;
    }
    for row in &dto.nodes {
        transaction
            .execute(
                "INSERT INTO node_views (id, diagram_id, position, element_id, x, y, width, \
                 height) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id(row.id)?,
                    id(row.diagram_id)?,
                    row.position,
                    id(row.element_id)?,
                    row.x,
                    row.y,
                    row.width,
                    row.height
                ],
            )
            .map_err(|error| write_error(&error))?;
    }
    for row in &dto.edges {
        transaction
            .execute(
                "INSERT INTO edge_views (id, diagram_id, position, relation_id, source_view, \
                 target_view) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id(row.id)?,
                    id(row.diagram_id)?,
                    row.position,
                    id(row.relation_id)?,
                    id(row.source_view)?,
                    id(row.target_view)?
                ],
            )
            .map_err(|error| write_error(&error))?;
    }
    if fault == Some(FailPoint::Write) {
        // Dropping the transaction rolls everything back.
        return Err(SaveError::Injected(FailPoint::Write));
    }
    transaction.commit().map_err(|error| write_error(&error))
}

/// An ID as SQLite's signed integer; larger values are not valid IDs.
fn id(value: u64) -> Result<i64, SaveError> {
    i64::try_from(value)
        .map_err(|_| SaveError::Write(format!("the ID {value} is too large to store")))
}

/// Decides whether an existing file at `path` may be replaced: a project of
/// this or an older format, a damaged project (replacing it is the repair),
/// or an empty file (such as one a file dialog just created). A project from
/// a newer version, or any other file, is refused.
fn check_target(path: &Path) -> Result<(), SaveError> {
    // A leftover journal or log next to the target would attach itself to the
    // new file and corrupt it, so refuse until it is dealt with.
    for suffix in ["-journal", "-wal", "-shm"] {
        let mut sidecar = path.as_os_str().to_owned();
        sidecar.push(suffix);
        if Path::new(&sidecar).exists() {
            return Err(SaveError::SidecarsPresent {
                path: path.to_path_buf(),
                sidecar: format!(
                    "{}{suffix}",
                    path.file_name()
                        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
                ),
            });
        }
    }
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(());
    };
    if metadata.is_dir() || metadata.len() == 0 {
        // A folder fails later at the swap with a typed error; an empty file is free to take.
        return Ok(());
    }
    // Identity comes from the file's own header bytes, so even a damaged or
    // write-ahead-log file is told apart from a foreign one without opening it.
    match identity(path) {
        Ok(Identity::Empty) => Ok(()),
        Ok(Identity::Ours { version, .. }) if version > SCHEMA_VERSION => {
            Err(SaveError::NewerTarget {
                path: path.to_path_buf(),
                found: version,
            })
        }
        Ok(Identity::Ours { .. }) => Ok(()),
        Ok(Identity::NotSqlite | Identity::Foreign) | Err(_) => Err(SaveError::ForeignTarget {
            path: path.to_path_buf(),
        }),
    }
}

/// Removes temporary files that an interrupted save of `path` left behind
/// (`.<file name>.saving-*` in the same folder). Call it when no save of this
/// project is running, for example when the project is opened. Returns how
/// many files were removed.
///
/// # Errors
///
/// Returns the I/O error if the folder cannot be read or a file cannot be removed.
pub fn remove_orphans(path: &Path) -> std::io::Result<usize> {
    let Some(file_name) = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
    else {
        return Ok(0);
    };
    let folder = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let prefix = format!(".{file_name}.saving-");
    let mut removed = 0;
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(&prefix) && entry.file_type()?.is_file() {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Saves `document` as the project `name` at `path`.
///
/// # Errors
///
/// Returns [`SaveError`] naming the step that failed. The file at `path`,
/// if there was one, is then unchanged and no temporary file remains.
pub fn save(path: &Path, name: &str, document: &Document) -> Result<(), SaveError> {
    save_with_fault(path, name, document, None)
}

/// [`save()`] with an optional injected failure, for tests of the failure paths.
///
/// # Errors
///
/// As [`save()`]; with `fault` set, fails with [`SaveError::Injected`] at that step.
#[doc(hidden)]
pub fn save_with_fault(
    path: &Path,
    name: &str,
    document: &Document,
    fault: Option<FailPoint>,
) -> Result<(), SaveError> {
    let problems: Vec<String> = check_document(document)
        .into_iter()
        .filter(|issue| issue.severity == Severity::Error)
        .map(|issue| issue.message)
        .collect();
    if !problems.is_empty() {
        return Err(SaveError::Invalid(problems));
    }
    check_target(path)?;
    if fault == Some(FailPoint::Create) {
        return Err(SaveError::Injected(FailPoint::Create));
    }
    let temporary_path = temporary_path(path);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|source| SaveError::Create {
            path: path.to_path_buf(),
            source,
        })?;
    let mut temporary = Temporary {
        path: temporary_path,
        published: false,
    };

    let dto = ProjectDto::from_document(name, document);
    {
        let mut connection =
            Connection::open(&temporary.path).map_err(|error| write_error(&error))?;
        write_rows(&mut connection, &dto, fault)?;
        connection
            .close()
            .map_err(|(_, error)| write_error(&error))?;
    }

    // Make the finished file durable before it can become the project file.
    if fault == Some(FailPoint::Flush) {
        return Err(SaveError::Injected(FailPoint::Flush));
    }
    fs::OpenOptions::new()
        .write(true)
        .open(&temporary.path)
        .and_then(|file| file.sync_all())
        .map_err(SaveError::Flush)?;

    if fault == Some(FailPoint::Verify) {
        return Err(SaveError::Injected(FailPoint::Verify));
    }
    let (read_name, read_document) =
        load_checked(&temporary.path).map_err(|error| SaveError::Verify(error.to_string()))?;
    if read_name != name || &read_document != document {
        return Err(SaveError::Verify(String::from(
            "the file reads back as a different project",
        )));
    }

    if fault == Some(FailPoint::Replace) {
        return Err(SaveError::Injected(FailPoint::Replace));
    }
    fs::rename(&temporary.path, path).map_err(|source| SaveError::Replace {
        path: path.to_path_buf(),
        source,
    })?;
    temporary.published = true;
    Ok(())
}
