//! Persistence and recovery boundary for Diagram Studio.
//!
//! Reserved for validated DTOs, SQLite schema versions, atomic saves and recovery.
//! A desktop worker will own connections and receive owned document snapshots;
//! this crate must not depend on widgets or interpret pointer input.
//!
//! SQLite is compiled into the application through rusqlite's `bundled`
//! feature (R0-03), so users never install a separate SQLite runtime. No file
//! format or project schema exists yet; M1 defines it.

#![deny(missing_docs)]

/// Version of the SQLite library linked into this build, for example `3.51.1`.
///
/// Shown in diagnostics so problem reports identify the exact database engine.
pub fn sqlite_version() -> &'static str {
    rusqlite::version()
}
