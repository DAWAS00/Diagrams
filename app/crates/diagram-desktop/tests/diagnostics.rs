use std::error::Error;
use std::io::{self, Write};
use std::process::Command;

use diagram_desktop::{APPLICATION_NAME, launch_failure_message, native_options, write_version};

const EXECUTABLE: &str = env!("CARGO_BIN_EXE_diagram-desktop");

#[test]
fn application_name_should_be_the_working_product_name() {
    assert_eq!(APPLICATION_NAME, "Diagram Studio");
}

#[test]
fn native_options_should_use_wgpu_and_titled_window() {
    let options = native_options();

    assert_eq!(options.renderer, eframe::Renderer::Wgpu);
    assert_eq!(options.viewport.title.as_deref(), Some(APPLICATION_NAME));
    assert_eq!(options.viewport.inner_size, Some([1280.0, 800.0].into()));
}

#[test]
fn write_version_should_name_app_and_bundled_sqlite() -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();

    write_version(&mut output)?;

    let expected = format!(
        "Diagram Studio {}\nSQLite {} (bundled)\n",
        env!("CARGO_PKG_VERSION"),
        diagram_storage::sqlite_version()
    );
    assert_eq!(String::from_utf8(output)?, expected);
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
fn write_version_should_return_the_writer_error() {
    let result = write_version(&mut FailingWriter);

    assert_eq!(
        result.map_err(|error| error.kind()),
        Err(io::ErrorKind::BrokenPipe)
    );
}

#[test]
fn launch_failure_message_should_include_error_cause_and_hint() {
    let cause = io::Error::other("font atlas unavailable");
    let error = eframe::Error::AppCreation(Box::new(cause));

    let message = launch_failure_message(&error);

    assert!(message.starts_with("Diagram Studio could not start: "));
    assert!(message.contains("font atlas unavailable"));
    assert!(message.ends_with("The application failed while starting up."));
}

#[test]
fn executable_version_flag_should_print_diagnostics_without_a_window() -> Result<(), Box<dyn Error>>
{
    let output = Command::new(EXECUTABLE).arg("--version").output()?;

    assert!(output.status.success(), "status: {}", output.status);
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with("Diagram Studio "));
    assert!(stdout.contains("(bundled)"));
    Ok(())
}

#[test]
fn executable_should_reject_unknown_arguments_with_usage() -> Result<(), Box<dyn Error>> {
    let output = Command::new(EXECUTABLE).arg("--bogus").output()?;

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)?.contains("Usage: diagram-desktop"));
    Ok(())
}

/// Without any display server the window cannot open; the user must get an
/// explanation and a failing exit code instead of a panic.
#[cfg(target_os = "linux")]
#[test]
fn executable_should_explain_missing_display() -> Result<(), Box<dyn Error>> {
    let output = Command::new(EXECUTABLE)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.starts_with("Diagram Studio could not start: "),
        "{stderr}"
    );
    assert!(stderr.contains("DISPLAY or WAYLAND_DISPLAY"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    Ok(())
}
