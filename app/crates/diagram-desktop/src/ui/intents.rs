//! Typed user intentions (UI-06).
//!
//! Widgets never change editor or project state directly. They queue a
//! [`UiIntent`]; after the frame is drawn, the shell's coordinator applies the
//! queue in order. Text fields are the exception: they edit a draft in place,
//! and committing drafts as model commands arrives with M1/M2 (UI-14).

use eframe::egui::{Id, ThemePreference};

use super::commands::CommandId;

/// Something the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiIntent {
    /// Run a catalog command if it is available. `origin` is the control it
    /// came from; a drawer or window it opens returns focus there.
    Run {
        /// The command.
        command: CommandId,
        /// The control that asked, if any.
        origin: Option<Id>,
    },
    /// Select a class on the current diagram.
    SelectClass(usize),
    /// Select a class from the project panel, opening a diagram that shows it.
    RevealClass(usize),
    /// Show a diagram in its tab, opening a tab if needed.
    OpenDiagram(usize),
    /// Make an open tab active.
    ActivateTab(usize),
    /// Close an open tab (model elements are untouched).
    CloseTab(usize),
    /// Change the theme preference.
    SetTheme(ThemePreference),
    /// Close the open drawer.
    CloseDrawer,
    /// Close the component showcase.
    CloseShowcase,
}

impl UiIntent {
    /// A command run that did not come from a particular control.
    pub const fn run(command: CommandId) -> Self {
        Self::Run {
            command,
            origin: None,
        }
    }
}
