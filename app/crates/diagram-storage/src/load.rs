//! Loading a project file: read-only, checked, and never rewritten.
//!
//! The file is opened read-only, so loading cannot create, change or
//! "repair" anything. Before any table is read it must carry our
//! `application_id` and a schema version this build understands; then SQLite's
//! integrity check runs; then every row is rebuilt through the core's rules
//! (see [`crate::dto`]).

use std::fs;
use std::io;
use std::path::Path;

use diagram_core::model::Document;
use rusqlite::{Connection, ErrorCode, OpenFlags, Row};

use crate::dto::{
    DiagramRow, EdgeRow, ElementRow, MemberRow, NodeRow, ParameterRow, ProjectDto, RelationRow,
};
use crate::error::LoadError;
use crate::schema::{APPLICATION_ID, SCHEMA_VERSION};

/// A project read from a file.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedProject {
    /// The project name.
    pub name: String,
    /// The validated document.
    pub document: Document,
}

/// What can be told about a project file without loading all of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInfo {
    /// The project name.
    pub name: String,
    /// The schema version of the file.
    pub schema_version: u32,
}

/// Most rows read from any one table; a larger file is refused as damaged.
const MAX_ROWS: usize = 1_000_000;

fn damaged(error: &rusqlite::Error) -> LoadError {
    LoadError::Damaged {
        reason: error.to_string(),
    }
}

/// Maps a SQLite failure: "not a database" means the file is not ours.
fn map_sqlite(path: &Path, error: &rusqlite::Error) -> LoadError {
    match error {
        rusqlite::Error::SqliteFailure(failure, _) if failure.code == ErrorCode::NotADatabase => {
            LoadError::NotAProject {
                path: path.to_path_buf(),
            }
        }
        other => damaged(other),
    }
}

/// Who a file says it belongs to, from its first 100 bytes alone (no SQLite
/// open, so nothing is created next to it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Identity {
    /// Zero bytes.
    Empty,
    /// Not a SQLite file, or too short to be one.
    NotSqlite,
    /// A SQLite file that is not a Diagram Studio project.
    Foreign,
    /// A Diagram Studio project: its schema version and whether it is in
    /// write-ahead-log mode (this program never writes that).
    Ours {
        /// The `user_version` stored in the header.
        version: u32,
        /// Write-ahead-log mode.
        wal: bool,
    },
}

/// Reads the SQLite header fields that identify a project.
pub(crate) fn identity(path: &Path) -> io::Result<Identity> {
    use std::io::Read;
    let mut header = [0_u8; 100];
    let mut file = fs::File::open(path)?;
    let mut filled = 0;
    while filled < header.len() {
        let count = file.read(&mut header[filled..])?;
        if count == 0 {
            break;
        }
        filled += count;
    }
    if filled == 0 {
        return Ok(Identity::Empty);
    }
    if filled < header.len() || &header[..16] != b"SQLite format 3\0" {
        return Ok(Identity::NotSqlite);
    }
    let application = i32::from_be_bytes([header[68], header[69], header[70], header[71]]);
    if application != APPLICATION_ID {
        return Ok(Identity::Foreign);
    }
    Ok(Identity::Ours {
        version: u32::from_be_bytes([header[60], header[61], header[62], header[63]]),
        // Bytes 18 and 19 are the write and read format versions: 2 means write-ahead log.
        wal: header[18] == 2 || header[19] == 2,
    })
}

/// Refuses what is not a rollback-journal project, **before SQLite opens
/// it**: opening a write-ahead-log file, even read-only, creates `-wal` and
/// `-shm` files next to it.
fn check_header(path: &Path) -> Result<(), LoadError> {
    match identity(path) {
        Err(error) => Err(LoadError::Damaged {
            reason: error.to_string(),
        }),
        Ok(Identity::Empty | Identity::NotSqlite | Identity::Foreign) => {
            Err(LoadError::NotAProject {
                path: path.to_path_buf(),
            })
        }
        Ok(Identity::Ours { wal: true, .. }) => Err(LoadError::Damaged {
            reason: String::from(
                "the file uses a write-ahead journal; this program writes delete-mode files only",
            ),
        }),
        Ok(Identity::Ours { .. }) => Ok(()),
    }
}

/// Opens `path` read-only and checks that it is a project of a known version.
fn open_checked(path: &Path) -> Result<(Connection, u32), LoadError> {
    match fs::metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(LoadError::NotFound {
                path: path.to_path_buf(),
            });
        }
        Err(error) => {
            return Err(LoadError::Damaged {
                reason: error.to_string(),
            });
        }
    }
    check_header(path)?;
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| map_sqlite(path, &error))?;
    // A project file is untrusted input: no trusted-schema functions, no writes.
    connection
        .execute_batch("PRAGMA trusted_schema = OFF; PRAGMA query_only = ON;")
        .map_err(|error| map_sqlite(path, &error))?;
    // Everything below, up to the last row, is read in one read transaction,
    // so a writer committing in between cannot give a mixed document.
    connection
        .execute_batch("BEGIN")
        .map_err(|error| map_sqlite(path, &error))?;
    let application: i32 = connection
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .map_err(|error| map_sqlite(path, &error))?;
    if application != APPLICATION_ID {
        return Err(LoadError::NotAProject {
            path: path.to_path_buf(),
        });
    }
    let version: u32 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| map_sqlite(path, &error))?;
    if version > SCHEMA_VERSION {
        return Err(LoadError::NewerVersion {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    if version != SCHEMA_VERSION {
        return Err(LoadError::Damaged {
            reason: format!("the file format number {version} is not one this program wrote"),
        });
    }
    // This program only writes rollback-journal files. A write-ahead-log file
    // would need sidecar files just to be read, so it is refused untouched.
    let mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .map_err(|error| map_sqlite(path, &error))?;
    if !mode.eq_ignore_ascii_case("delete") {
        return Err(LoadError::Damaged {
            reason: format!(
                "the journal mode is {mode}; this program writes delete-mode files only"
            ),
        });
    }
    Ok((connection, version))
}

fn integer(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

fn small(row: &Row<'_>, index: usize) -> rusqlite::Result<u32> {
    let value: i64 = row.get(index)?;
    u32::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

fn read_meta(connection: &Connection, key: &str) -> Result<Option<String>, LoadError> {
    let mut statement = connection
        .prepare("SELECT value FROM meta WHERE key = ?1")
        .map_err(|error| damaged(&error))?;
    let mut rows = statement.query([key]).map_err(|error| damaged(&error))?;
    match rows.next().map_err(|error| damaged(&error))? {
        Some(row) => row
            .get::<_, String>(0)
            .map(Some)
            .map_err(|error| damaged(&error)),
        None => Ok(None),
    }
}

/// Reads the project name and version without loading the rest.
///
/// # Errors
///
/// Returns the same typed causes as [`load()`] for a missing, foreign, newer
/// or damaged file.
pub fn peek(path: &Path) -> Result<ProjectInfo, LoadError> {
    let (connection, schema_version) = open_checked(path)?;
    let name = read_meta(&connection, "project_name")?.ok_or_else(|| LoadError::Damaged {
        reason: String::from("the project name is missing"),
    })?;
    Ok(ProjectInfo {
        name,
        schema_version,
    })
}

fn query<T>(
    connection: &Connection,
    sql: &str,
    map: impl Fn(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, LoadError> {
    let limited = format!("{sql} LIMIT {}", MAX_ROWS + 1);
    let mut statement = connection
        .prepare(&limited)
        .map_err(|error| damaged(&error))?;
    let rows = statement
        .query_map([], |row| map(row))
        .map_err(|error| damaged(&error))?;
    let rows = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| damaged(&error))?;
    if rows.len() > MAX_ROWS {
        return Err(LoadError::Damaged {
            reason: format!("a table has more than {MAX_ROWS} rows"),
        });
    }
    Ok(rows)
}

fn read_dto(connection: &Connection) -> Result<ProjectDto, LoadError> {
    let name = read_meta(connection, "project_name")?.ok_or_else(|| LoadError::Damaged {
        reason: String::from("the project name is missing"),
    })?;
    let next_id = read_meta(connection, "next_id")?
        .and_then(|text| text.parse::<u64>().ok())
        .ok_or_else(|| LoadError::Damaged {
            reason: String::from("the ID counter is missing or not a number"),
        })?;
    Ok(ProjectDto {
        name,
        next_id,
        elements: query(
            connection,
            "SELECT id, name FROM elements ORDER BY id",
            |row| {
                Ok(ElementRow {
                    id: integer(row, 0)?,
                    name: row.get(1)?,
                })
            },
        )?,
        members: query(
            connection,
            "SELECT id, element_id, position, kind, name, visibility, type_text, multiplicity, \
             return_type FROM members ORDER BY element_id, position",
            |row| {
                Ok(MemberRow {
                    id: integer(row, 0)?,
                    element_id: integer(row, 1)?,
                    position: small(row, 2)?,
                    kind: row.get(3)?,
                    name: row.get(4)?,
                    visibility: row.get(5)?,
                    type_text: row.get(6)?,
                    multiplicity: row.get(7)?,
                    return_type: row.get(8)?,
                })
            },
        )?,
        parameters: query(
            connection,
            "SELECT member_id, position, name, type_text FROM parameters \
             ORDER BY member_id, position",
            |row| {
                Ok(ParameterRow {
                    member_id: integer(row, 0)?,
                    position: small(row, 1)?,
                    name: row.get(2)?,
                    type_text: row.get(3)?,
                })
            },
        )?,
        relations: query(
            connection,
            "SELECT id, kind, source, target, source_role, source_multiplicity, target_role, \
             target_multiplicity FROM relations ORDER BY id",
            |row| {
                Ok(RelationRow {
                    id: integer(row, 0)?,
                    kind: row.get(1)?,
                    source: integer(row, 2)?,
                    target: integer(row, 3)?,
                    source_role: row.get(4)?,
                    source_multiplicity: row.get(5)?,
                    target_role: row.get(6)?,
                    target_multiplicity: row.get(7)?,
                })
            },
        )?,
        diagrams: query(
            connection,
            "SELECT id, position, name, notation FROM diagrams ORDER BY position",
            |row| {
                Ok(DiagramRow {
                    id: integer(row, 0)?,
                    position: small(row, 1)?,
                    name: row.get(2)?,
                    notation: row.get(3)?,
                })
            },
        )?,
        nodes: query(
            connection,
            "SELECT id, diagram_id, position, element_id, x, y, width, height FROM node_views \
             ORDER BY diagram_id, position",
            |row| {
                Ok(NodeRow {
                    id: integer(row, 0)?,
                    diagram_id: integer(row, 1)?,
                    position: small(row, 2)?,
                    element_id: integer(row, 3)?,
                    x: row.get(4)?,
                    y: row.get(5)?,
                    width: row.get(6)?,
                    height: row.get(7)?,
                })
            },
        )?,
        edges: query(
            connection,
            "SELECT id, diagram_id, position, relation_id, source_view, target_view FROM \
             edge_views ORDER BY diagram_id, position",
            |row| {
                Ok(EdgeRow {
                    id: integer(row, 0)?,
                    diagram_id: integer(row, 1)?,
                    position: small(row, 2)?,
                    relation_id: integer(row, 3)?,
                    source_view: integer(row, 4)?,
                    target_view: integer(row, 5)?,
                })
            },
        )?,
    })
}

/// Loads and validates the project at `path`. The file is never written.
///
/// # Errors
///
/// Returns [`LoadError`]: not found, not a project, saved by a newer
/// version, damaged, or invalid content (with the reasons).
pub fn load(path: &Path) -> Result<LoadedProject, LoadError> {
    let (name, document) = load_checked(path)?;
    Ok(LoadedProject { name, document })
}

/// The checked load used by [`load()`] and by the save's read-back check.
pub(crate) fn load_checked(path: &Path) -> Result<(String, Document), LoadError> {
    let (connection, _) = open_checked(path)?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| map_sqlite(path, &error))?;
    if integrity != "ok" {
        return Err(LoadError::Damaged {
            reason: format!("SQLite's integrity check reported: {integrity}"),
        });
    }
    let dto = read_dto(&connection)?;
    dto.into_document()
}
