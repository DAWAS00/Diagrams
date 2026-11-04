//! Diagram Studio native desktop executable.

use std::io;
use std::process::ExitCode;

use diagram_desktop::shell::ShellApp;
use diagram_desktop::{APPLICATION_NAME, launch_failure_message, native_options, write_version};

const USAGE: &str = "Usage: diagram-desktop [--version]";

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    match arguments.as_slice() {
        [] => run_shell(),
        [flag] if flag == "--version" => match write_version(&mut io::stdout().lock()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Could not write version information: {error}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run_shell() -> ExitCode {
    explain_panics();
    let result = eframe::run_native(
        APPLICATION_NAME,
        native_options(),
        Box::new(|_creation_context| Ok(Box::<ShellApp>::default())),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", launch_failure_message(&error));
            ExitCode::FAILURE
        }
    }
}

/// Platform libraries (for example a missing libxkbcommon-x11 on Linux) can
/// panic inside the windowing stack instead of returning an error. Prefix the
/// standard panic report with an explanation for the user.
fn explain_panics() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!(
            "{APPLICATION_NAME} stopped because of an internal error. \
             If this happened at start-up, a required graphics or windowing \
             library may be missing. Please include the details below in a problem report."
        );
        default_hook(info);
    }));
}
