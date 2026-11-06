//! Persistence and recovery boundary for Diagram Studio.
//!
//! A project is one SQLite file (schema v1, see [`schema`]). The crate turns
//! an in-memory [`diagram_core::model::Document`] into rows ([`dto`]), saves
//! it atomically ([`save`](mod@save)), loads it with every check ([`load`](mod@load)) and keeps
//! recovery snapshots ([`recovery`]). Failures are typed ([`error`]).
//!
//! A desktop worker will own connections and receive owned document
//! snapshots; this crate must not depend on widgets or interpret pointer input.
//!
//! SQLite is compiled into the application through rusqlite's `bundled`
//! feature (R0-03), so users never install a separate SQLite runtime.

#![deny(missing_docs)]

pub mod dto;
pub mod error;
pub mod load;
mod rebuild;
pub mod recovery;
pub mod save;
pub mod schema;

pub use error::{FailPoint, LoadError, SaveError};
pub use load::{LoadedProject, ProjectInfo, load, peek};
pub use save::save;

/// Version of the SQLite library linked into this build, for example `3.51.1`.
///
/// Shown in diagnostics so problem reports identify the exact database engine.
pub fn sqlite_version() -> &'static str {
    rusqlite::version()
}
