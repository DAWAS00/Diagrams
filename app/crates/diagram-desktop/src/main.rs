//! Console bootstrap for the initial Rust workspace.

use std::error::Error;
use std::io;

use diagram_core::ProjectName;
use diagram_desktop::write_bootstrap;

fn main() -> Result<(), Box<dyn Error>> {
    let project_name = ProjectName::new(String::from("Untitled"))?;
    write_bootstrap(&mut io::stdout().lock(), &project_name)?;
    Ok(())
}
