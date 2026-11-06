//! Typed causes for saving and loading, each with a message in words.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Where a save can fail. Tests inject one of these to prove that the
/// previous file survives a failure at that step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailPoint {
    /// Creating the temporary file.
    Create,
    /// Writing the project into the temporary file.
    Write,
    /// Flushing the finished temporary file to disk.
    Flush,
    /// Checking the finished temporary file.
    Verify,
    /// Replacing the target with the finished file.
    Replace,
}

impl FailPoint {
    /// The step in words.
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Create => "creating the temporary file",
            Self::Write => "writing the project",
            Self::Flush => "flushing the written file to disk",
            Self::Verify => "checking the written file",
            Self::Replace => "replacing the project file",
        }
    }
}

/// Why a save failed. The previous file is untouched in every case.
#[derive(Debug, Error)]
pub enum SaveError {
    /// The temporary file could not be created (missing folder, no permission).
    #[error("could not create a file next to {path}: {source}")]
    Create {
        /// The project path being saved.
        path: PathBuf,
        /// The operating system's reason.
        #[source]
        source: io::Error,
    },
    /// The path holds a project saved by a newer version; replacing it would destroy data.
    #[error(
        "{path} was saved by a newer version of Diagram Studio (file format {found}); \
         saving over it would destroy data this version cannot read"
    )]
    NewerTarget {
        /// The project path being saved.
        path: PathBuf,
        /// The format found in the existing file.
        found: u32,
    },
    /// The path holds some other file; it is not replaced silently.
    #[error("{path} exists and is not a Diagram Studio project, so it was not replaced")]
    ForeignTarget {
        /// The project path being saved.
        path: PathBuf,
    },
    /// SQLite sidecar files (`-journal`, `-wal`, `-shm`) sit next to the target;
    /// a new file swapped in would inherit them.
    #[error(
        "{path} has an unfinished SQLite journal next to it ({sidecar});          open and close the project in the program that wrote it, or remove that file, then save again"
    )]
    SidecarsPresent {
        /// The project path being saved.
        path: PathBuf,
        /// The file name found.
        sidecar: String,
    },
    /// The file could not be flushed to disk.
    #[error("could not flush the saved file to disk: {0}")]
    Flush(io::Error),
    /// The document breaks the project rules, so it is not written at all.
    #[error("the project cannot be saved because it breaks the project rules: {}", .0.join("; "))]
    Invalid(Vec<String>),
    /// SQLite refused a write.
    #[error("could not write the project: {0}")]
    Write(String),
    /// The finished file did not read back as the same project.
    #[error("the saved file did not pass its check: {0}")]
    Verify(String),
    /// The finished file could not replace the old one (for example it is open elsewhere).
    #[error("could not replace {path}: {source}")]
    Replace {
        /// The project path being saved.
        path: PathBuf,
        /// The operating system's reason.
        #[source]
        source: io::Error,
    },
    /// A test asked for a failure at this step.
    #[error("injected failure while {}", .0.describe())]
    Injected(FailPoint),
}

/// Why a project could not be loaded. Nothing is opened or rewritten.
#[derive(Debug, Error)]
pub enum LoadError {
    /// There is no file at the path.
    #[error("there is no file at {path}")]
    NotFound {
        /// The path that was asked for.
        path: PathBuf,
    },
    /// The file is not a Diagram Studio project.
    #[error("{path} is not a Diagram Studio project file")]
    NotAProject {
        /// The path that was asked for.
        path: PathBuf,
    },
    /// The file was saved by a newer version.
    #[error(
        "this project was saved by a newer version of Diagram Studio (file format {found}); \
         this version reads up to format {supported}"
    )]
    NewerVersion {
        /// The version in the file.
        found: u32,
        /// The newest version this build reads.
        supported: u32,
    },
    /// The file is damaged or cut short.
    #[error("the project file is damaged: {reason}")]
    Damaged {
        /// What was found.
        reason: String,
    },
    /// The file reads, but its content breaks the project rules.
    #[error("the project file has invalid content: {}", .0.join("; "))]
    Invalid(Vec<String>),
}
