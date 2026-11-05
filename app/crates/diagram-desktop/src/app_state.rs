//! The application state the shell shows and the dispatcher changes.
//!
//! The preview classes are prototype data owned by the desktop crate. They
//! exercise layout, selection and text editing before M1 binds the core
//! model, commands and history. Only [`crate::dispatch::apply`] changes this
//! state in response to the person; views read it through
//! [`crate::presentation::build`].

use eframe::egui::{Id, ThemePreference, Vec2};

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
    /// Index into [`Self::classes`] of the selected class, if any.
    pub selected: Option<usize>,
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

    /// The currently selected class, if the selection index is valid.
    pub fn selected_class(&self) -> Option<&PreviewClass> {
        self.selected.and_then(|index| self.classes.get(index))
    }

    /// Selects the class at `index`; out-of-range indices clear the selection.
    pub fn select(&mut self, index: usize) {
        self.selected = (index < self.classes.len()).then_some(index);
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// The state that decides which commands are available.
    pub fn command_context(&self) -> CommandContext {
        CommandContext {
            has_selection: self.selected_class().is_some(),
            selection_count: usize::from(self.selected_class().is_some()),
            has_content: !self.classes.is_empty(),
            tab_count: 1,
            can_zoom_in: false,
            can_zoom_out: false,
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
