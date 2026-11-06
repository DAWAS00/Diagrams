//! Recovery snapshots: autosaved copies of unsaved work.
//!
//! A snapshot is an ordinary project file in a recovery folder, written with
//! the same atomic save, so a crash while writing leaves the previous
//! snapshot intact. Recovering one uses the same checked load.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use diagram_core::model::Document;

use crate::error::{LoadError, SaveError};
use crate::load::{LoadedProject, load, peek};
use crate::save::save;

/// File extension of a recovery snapshot.
pub const SNAPSHOT_EXTENSION: &str = "dgrecovery";

/// One entry of the recovery folder.
#[derive(Debug)]
pub struct SnapshotEntry {
    /// Where the snapshot file is.
    pub path: PathBuf,
    /// When it was last written, if the system says.
    pub modified: Option<SystemTime>,
    /// The project name, or why the snapshot cannot be read.
    pub info: Result<String, LoadError>,
}

/// Most bytes a recovery key may have.
/// Kept small so the snapshot name, its temporary name and the `-journal`
/// name all stay far below the 255-character file name limit.
const MAX_KEY_BYTES: usize = 48;

/// A file name for `key` that is unique per key: a readable prefix of its
/// letters and digits, then the **whole key in hexadecimal**, so two different
/// keys (such as `project:a` and `project?a`) can never share a snapshot.
fn file_stem(key: &str) -> String {
    let readable: String = key
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(24)
        .collect();
    let readable = if readable.is_empty() {
        String::from("project")
    } else {
        readable
    };
    let encoded: String = key.bytes().map(|byte| format!("{byte:02x}")).collect();
    format!("{readable}-{encoded}")
}

/// Writes (or atomically replaces) the snapshot for `key`, which identifies
/// the open project (a stable identity such as "untitled-1", not its name).
/// Different keys always get different files.
///
/// # Errors
///
/// Returns the [`SaveError`] of the underlying save; the previous snapshot
/// for `key`, if any, is then unchanged.
pub fn write_snapshot(
    folder: &Path,
    key: &str,
    name: &str,
    document: &Document,
) -> Result<PathBuf, SaveError> {
    if key.len() > MAX_KEY_BYTES {
        return Err(SaveError::Write(format!(
            "the recovery key is longer than {MAX_KEY_BYTES} bytes"
        )));
    }
    fs::create_dir_all(folder).map_err(|source| SaveError::Create {
        path: folder.to_path_buf(),
        source,
    })?;
    let path = folder.join(format!("{}.{SNAPSHOT_EXTENSION}", file_stem(key)));
    save(&path, name, document)?;
    Ok(path)
}

/// Lists the snapshots in `folder`, newest first. A damaged snapshot is
/// listed with its error and never hides the others.
///
/// # Errors
///
/// Returns the I/O error if the folder exists but cannot be read; a missing
/// folder is an empty list.
pub fn list_snapshots(folder: &Path) -> io::Result<Vec<SnapshotEntry>> {
    let reader = match fs::read_dir(folder) {
        Ok(reader) => reader,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut entries = Vec::new();
    for item in reader {
        let path = item?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some(SNAPSHOT_EXTENSION) {
            continue;
        }
        let modified = fs::metadata(&path).and_then(|meta| meta.modified()).ok();
        let info = peek(&path).map(|info| info.name);
        entries.push(SnapshotEntry {
            path,
            modified,
            info,
        });
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.modified));
    Ok(entries)
}

/// Loads a snapshot with the normal checks.
///
/// # Errors
///
/// As [`load()`](crate::load::load).
pub fn recover(path: &Path) -> Result<LoadedProject, LoadError> {
    load(path)
}

/// Removes a snapshot. Only a regular snapshot file directly inside
/// `folder` is removed: a path elsewhere, a folder, or a link is refused.
///
/// # Errors
///
/// Returns an I/O error if the path is not such a snapshot or cannot be removed.
pub fn discard(folder: &Path, path: &Path) -> io::Result<()> {
    let refuse = |why: &str| io::Error::new(io::ErrorKind::PermissionDenied, why.to_owned());
    let inside =
        path.parent().map(fs::canonicalize).transpose()? == Some(fs::canonicalize(folder)?);
    if !inside {
        return Err(refuse("the file is not in the recovery folder"));
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some(SNAPSHOT_EXTENSION) {
        return Err(refuse("the file is not a recovery snapshot"));
    }
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(refuse("only a regular snapshot file is removed"));
    }
    fs::remove_file(path)
}
