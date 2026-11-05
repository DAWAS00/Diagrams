//! The application state the shell shows and the dispatcher changes.
//!
//! The preview classes are prototype data owned by the desktop crate. They
//! exercise layout, selection and text editing before M1 binds the core
//! model, commands and history. Only [`crate::dispatch::apply`] changes this
//! state in response to the person; views read it through
//! [`crate::presentation::build`].

use eframe::egui::{Id, ThemePreference, Vec2};

use diagram_canvas::{MAX_ZOOM, MIN_ZOOM};

use crate::editor_state::{EditorState, Gesture, ShellSupport, TabState, Tool};
use crate::ui::commands::CommandContext;
use crate::ui::palette::PaletteState;
use crate::ui::showcase::ShowcaseState;
use crate::ui::welcome::EntryState;
use crate::ui::workspace::{Drawer, LayoutMode, PanelPreferences};

/// Width of a sample class box, in canvas points.
pub const CLASS_WIDTH: f32 = 180.0;
/// Height of a class box's name compartment.
pub const CLASS_HEADER_HEIGHT: f32 = 32.0;
/// Height of one attribute line.
pub const ATTRIBUTE_HEIGHT: f32 = 20.0;

/// Name of the sample project's diagram.
pub const SAMPLE_DIAGRAM: &str = "Class diagram 1";

/// Name of the second fixture diagram used to exercise tabs.
pub const SECOND_DIAGRAM: &str = "Class diagram 2 (design fixture)";

/// Name of the built-in sample project.
pub const SAMPLE_PROJECT: &str = "Sample project";

/// Message shown when a class name has no visible characters.
pub const BLANK_NAME_WARNING: &str = "A class name must contain a non-whitespace character.";

/// A class box drawn on the prototype canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewClass {
    /// Class name, edited through the inspector Name field.
    pub name: String,
    /// Attribute lines displayed below the name.
    pub attributes: Vec<String>,
    /// Top-left corner, relative to the canvas origin.
    pub position: Vec2,
}

impl PreviewClass {
    /// Name used in the canvas and accessibility tree; blank names get a placeholder.
    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            "Unnamed class"
        } else {
            &self.name
        }
    }

    /// Size of the class box: the name compartment plus one line per attribute.
    pub fn size(&self) -> Vec2 {
        let rows = self.attributes.len().max(1) as f32;
        Vec2::new(
            CLASS_WIDTH,
            CLASS_HEADER_HEIGHT + rows * ATTRIBUTE_HEIGHT + 8.0,
        )
    }
}

/// The sample classes shown by the sample project.
pub fn sample_classes() -> Vec<PreviewClass> {
    let class = |name: &str, attributes: &[&str], x, y| PreviewClass {
        name: name.to_owned(),
        attributes: attributes.iter().map(|a| (*a).to_owned()).collect(),
        position: Vec2::new(x, y),
    };
    vec![
        class("Customer", &["id: CustomerId", "name: String"], 48.0, 48.0),
        class("Order", &["id: OrderId", "total: Money"], 300.0, 96.0),
    ]
}

/// Editor and application state.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    /// Classes shown on the canvas, in drawing order.
    pub classes: Vec<PreviewClass>,
    /// The primary selected class: the last one added to [`Self::selection`].
    /// Change it only through the selection methods, which keep both in step.
    pub selected: Option<usize>,
    /// Every selected class, in the order selected.
    pub selection: Vec<usize>,
    /// Diagrams in the project (design fixture names until M1).
    pub diagrams: Vec<String>,
    /// Tool, gesture, tabs, viewport, preferences, drafts and searches.
    pub editor: EditorState,
    /// What the running shell draws and sends.
    pub support: ShellSupport,
    /// Light, dark or follow the operating system. Light is the default.
    pub theme: ThemePreference,
    /// The command and object search, while it is open.
    pub palette: Option<PaletteState>,
    /// Which docked panels the user wants; kept across window sizes.
    pub panels: PanelPreferences,
    /// The open drawer on narrow windows, if any.
    pub drawer: Option<Drawer>,
    /// Docked or narrow, decided from the window width each frame.
    pub layout: LayoutMode,
    /// Widget to focus when the open drawer closes (usually its toggle).
    pub(crate) drawer_return: Option<Id>,
    /// The welcome or new-project screen, while one is shown instead of the workspace.
    pub entry: Option<EntryState>,
    /// Name of the open project, shown in the header.
    pub project_name: String,
    /// The component showcase window, while it is open.
    pub showcase: Option<ShowcaseState>,
    /// Widget to focus when the showcase closes.
    pub(crate) showcase_return: Option<Id>,
    /// Widget to focus at the start of the next frame, after a dialog closes.
    pub(crate) pending_focus: Option<Id>,
    /// Theme preference currently installed in the egui context.
    pub(crate) applied_theme: Option<ThemePreference>,
    /// Whether a text field (not just any widget) had keyboard focus at the
    /// end of the last frame.
    pub(crate) text_had_focus: bool,
}

/// The name the shell and its tests have always used for [`AppState`].
pub type ShellState = AppState;

impl Default for AppState {
    fn default() -> Self {
        Self {
            classes: sample_classes(),
            selected: None,
            selection: Vec::new(),
            diagrams: vec![String::from(SAMPLE_DIAGRAM)],
            editor: EditorState::default(),
            support: ShellSupport::default(),
            theme: ThemePreference::Light,
            palette: None,
            panels: PanelPreferences::default(),
            drawer: None,
            layout: LayoutMode::Docked,
            drawer_return: None,
            entry: None,
            project_name: String::from(SAMPLE_PROJECT),
            showcase: None,
            showcase_return: None,
            pending_focus: None,
            applied_theme: None,
            text_had_focus: false,
        }
    }
}

impl AppState {
    /// State at application start: the welcome screen.
    pub fn welcome() -> Self {
        Self {
            entry: Some(EntryState::default()),
            ..Self::default()
        }
    }

    /// The sample project with a second diagram tab open (design fixture),
    /// for tab switching before the project model exists.
    pub fn with_second_diagram() -> Self {
        let mut state = Self::default();
        state.diagrams.push(String::from(SECOND_DIAGRAM));
        state.editor.tabs.push(TabState::new(1));
        state
    }

    /// The currently selected class, if the selection index is valid.
    pub fn selected_class(&self) -> Option<&PreviewClass> {
        self.selected.and_then(|index| self.classes.get(index))
    }

    /// Selects only the class at `index`; out-of-range indices clear the selection.
    pub fn select(&mut self, index: usize) {
        self.set_selection(vec![index]);
    }

    /// Adds the class at `index` to the selection, or removes it if selected.
    pub fn toggle_selected(&mut self, index: usize) {
        let mut selection = self.selection.clone();
        if let Some(position) = selection.iter().position(|&item| item == index) {
            selection.remove(position);
        } else {
            selection.push(index);
        }
        self.set_selection(selection);
    }

    /// Replaces the selection; invalid and repeated indices are dropped.
    pub fn set_selection(&mut self, indices: Vec<usize>) {
        let mut selection: Vec<usize> = Vec::with_capacity(indices.len());
        for index in indices {
            if index < self.classes.len() && !selection.contains(&index) {
                selection.push(index);
            }
        }
        self.selected = selection.last().copied();
        self.selection = selection;
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.set_selection(Vec::new());
    }

    /// Indices of the selected classes. A primary selection set without the
    /// selection methods still counts.
    pub fn selected_indices(&self) -> Vec<usize> {
        match self.selected {
            Some(index) if !self.selection.contains(&index) => vec![index],
            _ => self.selection.clone(),
        }
    }

    /// Escape has something to cancel.
    pub fn can_cancel(&self) -> bool {
        self.editor.pending.is_some()
            || self.editor.draft.is_some()
            || self.editor.gesture != Gesture::Idle
            || self.editor.tool != Tool::Select
            || self.selected_class().is_some()
    }

    /// The state that decides which commands are available.
    pub fn command_context(&self) -> CommandContext {
        let selection_count = self.selected_indices().len();
        let zoom = self.editor.viewport().zoom();
        CommandContext {
            has_selection: selection_count > 0,
            selection_count,
            has_content: !self.classes.is_empty(),
            tab_count: self.editor.tabs.len(),
            can_zoom_in: zoom < MAX_ZOOM,
            can_zoom_out: zoom > MIN_ZOOM,
            can_cancel: self.can_cancel(),
            support: self.support,
        }
    }

    /// Whether a panel's toggle shows as pressed: docked and visible, or its
    /// drawer is open on a narrow window.
    pub fn panel_shown(&self, which: Drawer) -> bool {
        match (self.layout, which) {
            (LayoutMode::Docked, Drawer::Navigator) => self.panels.navigator_visible,
            (LayoutMode::Docked, Drawer::Inspector) => self.panels.inspector_visible,
            (LayoutMode::Narrow, _) => self.drawer == Some(which),
        }
    }

    /// Widget the shell should focus once no modal is open, if any.
    pub fn pending_focus(&self) -> Option<Id> {
        self.pending_focus
    }
}
