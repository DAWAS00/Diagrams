//! Desktop application boundary for Diagram Studio.
//!
//! The native shell will own editor state, coordinate commands and dispatch
//! worker jobs. R0-01 provides the crate wiring and a console bootstrap; the
//! native eframe shell is tracked separately as R0-02.

#![deny(missing_docs)]

/// Working product name displayed by the desktop executable.
pub const APPLICATION_NAME: &str = "Diagram Studio";
