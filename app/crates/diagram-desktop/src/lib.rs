//! Desktop application boundary for Diagram Studio.
//!
//! The native shell will own editor state, coordinate commands and dispatch
//! worker jobs. R0-01 provides the crate wiring and a console bootstrap; the
//! native eframe shell is tracked separately as R0-02.

#![deny(missing_docs)]

use std::io::{self, Write};

use diagram_core::ProjectName;

/// Working product name displayed by the desktop executable.
pub const APPLICATION_NAME: &str = "Diagram Studio";

/// Writes the console bootstrap message for `project_name` to `output`.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] when `output` rejects a write.
pub fn write_bootstrap(output: &mut impl Write, project_name: &ProjectName) -> io::Result<()> {
    writeln!(output, "{APPLICATION_NAME} - Rust workspace foundation")?;
    writeln!(output, "Example project name: {project_name}")?;
    writeln!(
        output,
        "Native GUI: next task R0-02. No project file was created."
    )
}
