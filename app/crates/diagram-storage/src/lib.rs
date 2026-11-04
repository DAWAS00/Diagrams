//! Persistence and recovery boundary for Diagram Studio.
//!
//! Reserved for validated DTOs, SQLite schema versions, atomic saves and recovery.
//! A desktop worker will own connections and receive owned document snapshots;
//! this crate must not depend on widgets or interpret pointer input.
//!
//! No file format or database operations are implemented in R0-01. Bundled
//! SQLite is introduced and tested in R0-03 before M1 defines the project schema.

#![deny(missing_docs)]
