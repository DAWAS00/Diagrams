//! Command and object search (UI-06): the first surface built on the command
//! catalog. It lists matching commands with their shortcut or the reason they
//! are unavailable, and matching objects by name.

use eframe::egui;

use super::commands::{self, CommandId};

/// One search result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteEntry {
    /// A command from the catalog.
    Command(CommandId),
    /// An object, by its index in the caller's list of names.
    Object(usize),
}

/// Search state while the palette is open.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaletteState {
    /// Text typed into the search field.
    pub query: String,
    /// Index of the highlighted result; Enter runs it.
    pub highlighted: usize,
    /// Widget that had keyboard focus before the palette opened.
    pub restore_focus: Option<egui::Id>,
    /// The search field has already been given focus.
    pub focused_once: bool,
    /// Move the highlight to the first available result in the next frame
    /// (set when the palette opens and whenever the query changes).
    pub highlight_pending: bool,
}

impl PaletteState {
    /// Opens a palette that returns focus to `restore_focus` when it closes.
    pub fn new(restore_focus: Option<egui::Id>) -> Self {
        Self {
            restore_focus,
            highlight_pending: true,
            ..Self::default()
        }
    }
}

/// Matching results for `query`: commands first (excluding the search
/// command itself), then objects whose name contains the query.
pub fn entries(query: &str, object_names: &[&str]) -> Vec<PaletteEntry> {
    let needle = query.trim().to_lowercase();
    let commands = commands::search(query)
        .into_iter()
        .filter(|spec| spec.id != CommandId::SearchCommands)
        .map(|spec| PaletteEntry::Command(spec.id));
    let objects = object_names
        .iter()
        .enumerate()
        .filter(|(_, name)| needle.is_empty() || name.to_lowercase().contains(&needle))
        .map(|(index, _)| PaletteEntry::Object(index));
    commands.chain(objects).collect()
}

/// Visible and accessible text of a result row, such as "File: Save" or
/// "Class: Customer". The prefix keeps rows distinct from toolbar buttons.
pub fn entry_label(entry: PaletteEntry, object_names: &[&str]) -> String {
    match entry {
        PaletteEntry::Command(id) => {
            let spec = id.spec();
            format!("{}: {}", spec.group.label(), spec.label)
        }
        PaletteEntry::Object(index) => {
            let name = object_names.get(index).copied().unwrap_or_default();
            format!("Class: {name}")
        }
    }
}

/// Moves the highlight with the arrow keys, staying within `count` results.
pub fn move_highlight(highlighted: usize, count: usize, down: bool, up: bool) -> usize {
    if count == 0 {
        return 0;
    }
    let current = highlighted.min(count - 1);
    match (down, up) {
        (true, false) => (current + 1).min(count - 1),
        (false, true) => current.saturating_sub(1),
        _ => current,
    }
}

/// Index of the first result that can run, so Enter does something useful;
/// 0 when none can.
pub fn first_available(
    results: &[PaletteEntry],
    available: impl Fn(PaletteEntry) -> bool,
) -> usize {
    results
        .iter()
        .position(|entry| available(*entry))
        .unwrap_or(0)
}
