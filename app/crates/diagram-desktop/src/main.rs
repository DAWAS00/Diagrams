//! Console bootstrap for the initial Rust workspace.

use std::error::Error;
use std::io::{self, Write};

use diagram_core::ProjectName;
use diagram_desktop::APPLICATION_NAME;

fn main() -> Result<(), Box<dyn Error>> {
    let project_name = ProjectName::new(String::from("Untitled"))?;
    let mut output = io::stdout().lock();

    writeln!(output, "{APPLICATION_NAME} - Rust workspace foundation")?;
    writeln!(output, "Example project name: {project_name}")?;
    writeln!(
        output,
        "Native GUI: next task R0-02. No project file was created."
    )?;
    Ok(())
}
