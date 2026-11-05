//! A read-only snapshot of everything the screen shows this frame.
//!
//! [`build`] derives a [`Presentation`] from [`AppState`]. Views draw from it
//! and push [`crate::intent::UiIntent`]s; they never read or change the state
//! directly. Every label, availability and reason text comes from here (and
//! so from the command catalog), never from a view. Changes to these types
//! are additive only.

use eframe::egui::{Id, Pos2, ThemePreference, Vec2};

use crate::app_state::{AppState, BLANK_NAME_WARNING};
use crate::intent::{Region, Target};
use crate::ui::commands::{self, Availability, CommandContext, CommandGroup, CommandId, Placement};
use crate::ui::workspace::Drawer;

/// One command as a control shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandView {
    /// The command a control for this view runs.
    pub id: CommandId,
    /// Visible label and accessible name.
    pub label: String,
    /// Shortcut as the user reads it, such as "Ctrl+Z".
    pub shortcut_text: Option<String>,
    /// Whether it can run now, with the reason when it cannot.
    pub availability: Availability,
    /// `None` for a plain action; `Some(pressed)` for a tool or toggle that
    /// shows a pressed state.
    pub active: Option<bool>,
}

impl CommandView {
    /// True when the command can run.
    pub fn is_enabled(&self) -> bool {
        self.availability.is_enabled()
    }
}

/// A group of commands in the application menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuGroupView {
    /// The group.
    pub group: CommandGroup,
    /// Group label.
    pub label: &'static str,
    /// Commands in catalog order.
    pub commands: Vec<CommandView>,
}

/// Header: menu, project title and header commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderView {
    /// Project name with its save state, such as "Sample project · not saved".
    pub title: String,
    /// The application menu: every catalog command, grouped.
    pub menu: Vec<MenuGroupView>,
    /// Header buttons in visual order.
    pub commands: Vec<CommandView>,
}

/// One diagram tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabView {
    /// Tab label.
    pub label: String,
    /// The tab is the active one.
    pub active: bool,
}

/// Diagram tab row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsView {
    /// Open tabs in order.
    pub tabs: Vec<TabView>,
    /// Buttons after the tabs.
    pub commands: Vec<CommandView>,
}

/// Tool row: panel toggles at the ends, tools in between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolsView {
    /// The project-panel toggle.
    pub navigator_toggle: CommandView,
    /// Tools in order.
    pub tools: Vec<CommandView>,
    /// The inspector toggle.
    pub inspector_toggle: CommandView,
}

/// One row in a navigator list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowView {
    /// Row label.
    pub label: String,
    /// The row is selected.
    pub selected: bool,
    /// What selecting the row selects, if anything.
    pub target: Option<Target>,
}

/// Project panel: diagrams and model lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigatorView {
    /// Panel title.
    pub title: &'static str,
    /// Diagrams in the project.
    pub diagrams: Vec<RowView>,
    /// Model elements.
    pub model: Vec<RowView>,
}

/// One class box on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeView {
    /// What clicking the node selects.
    pub target: Target,
    /// Visible name (with a placeholder for blank names).
    pub name: String,
    /// Accessible name, such as "Class Customer".
    pub accessible_name: String,
    /// Attribute lines.
    pub attributes: Vec<String>,
    /// Top-left corner relative to the canvas origin, in screen points.
    pub position: Pos2,
    /// Size in screen points.
    pub size: Vec2,
    /// The node is selected.
    pub selected: bool,
}

/// The diagram canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasView {
    /// Accessible name of the canvas.
    pub accessible_name: &'static str,
    /// Nodes in drawing order.
    pub nodes: Vec<NodeView>,
    /// Text shown when there is nothing to draw.
    pub empty_text: Option<&'static str>,
}

/// A text field bound to a draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldView {
    /// Visible label and accessible name.
    pub label: &'static str,
    /// Current text.
    pub text: String,
    /// Warning or error shown beside the field.
    pub problem: Option<String>,
}

/// What the inspector shows for the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorKind {
    /// Nothing is selected.
    Nothing {
        /// Guidance text.
        guidance: &'static str,
    },
    /// One class is selected.
    Class {
        /// The Name field.
        name: FieldView,
        /// Attribute lines.
        attributes: Vec<String>,
    },
}

/// The inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorView {
    /// Panel title.
    pub title: &'static str,
    /// Contents for the current selection.
    pub kind: InspectorKind,
}

/// The status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusView {
    /// Save state in words.
    pub save_text: &'static str,
    /// Selection summary, such as "Selected: Customer".
    pub selection_text: String,
    /// Current theme preference.
    pub theme: ThemePreference,
}

/// A modal dialog that is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogView {
    /// The command and object search.
    CommandSearch,
}

/// The command and object search contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSearchView {
    /// The search is open.
    pub open: bool,
    /// The query typed so far.
    pub query: String,
}

/// Everything the screen shows this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Presentation {
    /// Header.
    pub header: HeaderView,
    /// Diagram tabs.
    pub tabs: TabsView,
    /// Tool row.
    pub tools: ToolsView,
    /// Project panel.
    pub navigator: NavigatorView,
    /// Canvas.
    pub canvas: CanvasView,
    /// Inspector.
    pub inspector: InspectorView,
    /// Status bar.
    pub status: StatusView,
    /// The open modal dialog, if any.
    pub dialog: Option<DialogView>,
    /// Command search.
    pub command_search: CommandSearchView,
    /// The region that has keyboard focus.
    pub focus_region: Region,
    /// Widget the shell should focus once no modal is open.
    pub focus_request: Option<Id>,
}

/// Name of the only diagram until the project model exists.
pub const SAMPLE_DIAGRAM: &str = "Class diagram 1";

/// Text shown on an empty canvas.
pub const EMPTY_CANVAS: &str =
    "This diagram is empty. Adding classes arrives with the project model.";

/// Inspector guidance when nothing is selected.
pub const NOTHING_SELECTED: &str =
    "Nothing selected. Select a class on the canvas or in the Model list.";

/// Status text for the save state of the prototype.
pub const SAVE_STATE: &str = "Prototype · nothing is saved";

/// The command as a control shows it in `context`.
pub fn command_view(id: CommandId, context: CommandContext) -> CommandView {
    let spec = id.spec();
    CommandView {
        id,
        label: spec.label.to_owned(),
        shortcut_text: spec.shortcut.as_ref().map(commands::shortcut_text),
        availability: commands::availability(id, context),
        active: None,
    }
}

fn pressed(mut view: CommandView, pressed: bool) -> CommandView {
    view.active = Some(pressed);
    view
}

fn placed(placement: Placement, context: CommandContext) -> Vec<CommandView> {
    commands::placed(placement)
        .map(|spec| command_view(spec.id, context))
        .collect()
}

/// Builds the snapshot the views draw this frame.
pub fn build(state: &AppState) -> Presentation {
    let context = state.command_context();
    let menu = commands::menu_groups()
        .into_iter()
        .map(|(group, specs)| MenuGroupView {
            group,
            label: group.label(),
            commands: specs
                .into_iter()
                .map(|spec| command_view(spec.id, context))
                .collect(),
        })
        .collect();
    let tools = placed(Placement::ToolRow, context)
        .into_iter()
        .map(|view| match view.id {
            // Select is the only tool so far, so it is always the active one.
            CommandId::SelectTool => pressed(view, true),
            _ => view,
        })
        .collect();
    let selected_name = state
        .selected_class()
        .map(|class| class.display_name().to_owned());

    Presentation {
        header: HeaderView {
            title: format!("{} · not saved", state.project_name),
            menu,
            commands: placed(Placement::Header, context),
        },
        tabs: TabsView {
            tabs: vec![TabView {
                label: SAMPLE_DIAGRAM.to_owned(),
                active: true,
            }],
            commands: placed(Placement::TabRow, context),
        },
        tools: ToolsView {
            navigator_toggle: pressed(
                command_view(CommandId::ToggleNavigator, context),
                state.panel_shown(Drawer::Navigator),
            ),
            tools,
            inspector_toggle: pressed(
                command_view(CommandId::ToggleInspector, context),
                state.panel_shown(Drawer::Inspector),
            ),
        },
        navigator: NavigatorView {
            title: "Project",
            diagrams: vec![RowView {
                label: SAMPLE_DIAGRAM.to_owned(),
                selected: true,
                target: None,
            }],
            model: state
                .classes
                .iter()
                .enumerate()
                .map(|(index, class)| RowView {
                    label: class.display_name().to_owned(),
                    selected: state.selected == Some(index),
                    target: Some(Target::Class(index)),
                })
                .collect(),
        },
        canvas: CanvasView {
            accessible_name: "Diagram canvas",
            nodes: state
                .classes
                .iter()
                .enumerate()
                .map(|(index, class)| NodeView {
                    target: Target::Class(index),
                    name: class.display_name().to_owned(),
                    accessible_name: format!("Class {}", class.display_name()),
                    attributes: class.attributes.clone(),
                    position: class.position.to_pos2(),
                    size: class.size(),
                    selected: state.selected == Some(index),
                })
                .collect(),
            empty_text: state.classes.is_empty().then_some(EMPTY_CANVAS),
        },
        inspector: InspectorView {
            title: "Selection",
            kind: match state.selected_class() {
                None => InspectorKind::Nothing {
                    guidance: NOTHING_SELECTED,
                },
                Some(class) => InspectorKind::Class {
                    name: FieldView {
                        label: "Name",
                        text: class.name.clone(),
                        problem: class
                            .name
                            .trim()
                            .is_empty()
                            .then(|| BLANK_NAME_WARNING.to_owned()),
                    },
                    attributes: class.attributes.clone(),
                },
            },
        },
        status: StatusView {
            save_text: SAVE_STATE,
            selection_text: match selected_name {
                Some(name) => format!("Selected: {name}"),
                None => String::from("No selection"),
            },
            theme: state.theme,
        },
        dialog: state.palette.as_ref().map(|_| DialogView::CommandSearch),
        command_search: CommandSearchView {
            open: state.palette.is_some(),
            query: state
                .palette
                .as_ref()
                .map(|search| search.query.clone())
                .unwrap_or_default(),
        },
        focus_region: Region::Board,
        focus_request: state.pending_focus(),
    }
}
