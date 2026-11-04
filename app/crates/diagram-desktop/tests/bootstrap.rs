use std::error::Error;
use std::io::{self, Write};
use std::process::Command;

use diagram_core::ProjectName;
use diagram_desktop::{APPLICATION_NAME, write_bootstrap};

const EXPECTED_UNTITLED_OUTPUT: &str = "Diagram Studio - Rust workspace foundation\n\
Example project name: Untitled\n\
Native GUI: next task R0-02. No project file was created.\n";

#[test]
fn application_name_should_be_the_working_product_name() {
    assert_eq!(APPLICATION_NAME, "Diagram Studio");
}

#[test]
fn write_bootstrap_should_print_the_three_line_message() -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();

    write_bootstrap(&mut output, &ProjectName::new(String::from("Untitled"))?)?;

    assert_eq!(String::from_utf8(output)?, EXPECTED_UNTITLED_OUTPUT);
    Ok(())
}

#[test]
fn write_bootstrap_should_print_the_project_name_unchanged() -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();

    write_bootstrap(
        &mut output,
        &ProjectName::new(String::from("  Customer نموذج  "))?,
    )?;

    assert!(String::from_utf8(output)?.contains("Example project name:   Customer نموذج  \n"));
    Ok(())
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn write_bootstrap_should_return_the_writer_error() -> Result<(), Box<dyn Error>> {
    let result = write_bootstrap(
        &mut FailingWriter,
        &ProjectName::new(String::from("Untitled"))?,
    );

    assert_eq!(
        result.map_err(|error| error.kind()),
        Err(io::ErrorKind::BrokenPipe)
    );
    Ok(())
}

#[test]
fn executable_should_print_bootstrap_and_exit_successfully() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_diagram-desktop")).output()?;

    assert!(output.status.success(), "status: {}", output.status);
    assert_eq!(String::from_utf8(output.stdout)?, EXPECTED_UNTITLED_OUTPUT);
    assert!(output.stderr.is_empty());
    Ok(())
}
