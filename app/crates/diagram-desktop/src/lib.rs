//! Desktop application boundary for Diagram Studio.
//!
//! The native shell owns editor state, will coordinate commands and dispatch
//! worker jobs. R0-02 provides the eframe/egui shell prototype in [`shell`].

#![deny(missing_docs)]

use std::error::Error;
use std::io::{self, Write};

pub mod shell;

/// Working product name displayed by the desktop executable.
pub const APPLICATION_NAME: &str = "Diagram Studio";

/// Window and renderer settings for the native shell (wgpu backend, ADR-001).
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(APPLICATION_NAME)
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    }
}

/// Writes the `--version` diagnostics: application and bundled SQLite versions.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] when `output` rejects a write.
pub fn write_version(output: &mut impl Write) -> io::Result<()> {
    writeln!(output, "{APPLICATION_NAME} {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(
        output,
        "SQLite {} (bundled)",
        diagram_storage::sqlite_version()
    )
}

/// Explains why the native window could not start and what the user can check.
pub fn launch_failure_message(error: &eframe::Error) -> String {
    let hint = match error {
        eframe::Error::Wgpu(_) => {
            "The graphics adapter could not be initialised. Update the graphics driver. \
             Remote desktops and virtual machines may not expose a Direct3D 12, Vulkan \
             or Metal capable adapter."
        }
        eframe::Error::Winit(_) | eframe::Error::WinitEventLoop(_) => {
            "The operating system could not create the window or its event loop. \
             On Linux, check that DISPLAY or WAYLAND_DISPLAY points to a running display."
        }
        eframe::Error::AppCreation(_) => "The application failed while starting up.",
    };
    let mut message = format!("{APPLICATION_NAME} could not start: {error}");
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(&format!("\n  caused by: {cause}"));
        source = cause.source();
    }
    message.push('\n');
    message.push_str(hint);
    message
}
