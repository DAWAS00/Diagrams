//! A read-only snapshot of everything the screen shows this frame.
//!
//! [`build`] derives a [`Presentation`] from [`AppState`]. Views draw from it
//! and push [`crate::intent::UiIntent`]s; they never read or change the state
//! directly. Every label, availability and reason text comes from here (and
//! so from the command catalog), never from a view. Changes to these types
//! are additive only.

use eframe::egui::{Id, Pos2, ThemePreference, Vec2};

use crate::app_state::{AppState, BLANK_NAME_WARNING};
use crate::editor_state::{Gesture, Tool};
use crate::intent::{DraftField, Region, Target};
use crate::ui::commands::{self, Availability, CommandContext, CommandGroup, CommandId, Placement};
use crate::ui::workspace::Drawer;
use crate::viewport;

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
    /// Zoom as a whole percentage.
    pub zoom_percent: u32,
    /// Draw the grid.
    pub grid: bool,
    /// The selection rectangle being dragged, in screen points relative to
    /// the canvas origin.
    pub marquee: Option<(Pos2, Pos2)>,
    /// The tool drags use now (Hand while Space is held).
    pub tool: Tool,
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
    /// Contents for the current selection. With several classes selected
    /// it describes the primary (last selected) one.
    pub kind: InspectorKind,
    /// Number of selected objects.
    pub selection_count: usize,
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
    /// Zoom in words, such as "Zoom 100%".
    pub zoom_text: String,
    /// The grid is shown.
    pub grid: bool,
    /// Snapping is on.
    pub snap: bool,
    /// What the active tool does, such as "Hand: drag to move the view".
    pub tool_hint: &'static str,
}

/// A modal dialog that is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogView {
    /// The command and object search.
    CommandSearch,
    /// An invalid edit would be lost: Return to field or Discard edit.
    ResolveEdit {
        /// Explanation, including the field's problem.
        message: String,
    },
}

/// The command and object search contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSearchView {
    /// The search is open.
    pub open: bool,
    /// The query typed so far.
    pub query: String,
    /// Matching commands only (objects are in [`ObjectSearchView`]); a
    /// disabled one carries its reason.
    pub results: Vec<CommandView>,
}

/// One object found by the object search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectResultView {
    /// Name with its kind, such as "Class: Customer".
    pub label: String,
    /// Where it is shown, such as "Class diagram 1".
    pub location: String,
    /// Selecting the result selects this.
    pub target: Target,
}

/// The object search in the project panel (Ctrl+F).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectSearchView {
    /// The search is open.
    pub open: bool,
    /// The query typed so far.
    pub query: String,
    /// Matching objects.
    pub results: Vec<ObjectResultView>,
    /// "No objects yet" or "No objects match"; `None` when there are results.
    pub empty_text: Option<&'static str>,
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
    /// Object search.
    pub object_search: ObjectSearchView,
    /// Diagram types the Add diagram picker offers (from the notation).
    pub diagram_types: Vec<&'static str>,
    /// The region that has keyboard focus.
    pub focus_region: Region,
    /// Widget the shell should focus once no modal is open.
    pub focus_request: Option<Id>,
}

pub use crate::app_state::SAMPLE_DIAGRAM;

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

/// Hint for the active tool in the status bar.
fn tool_hint(tool: Tool) -> &'static str {
    match tool {
        Tool::Select => "Select: click to select, drag empty space to select several",
        Tool::Hand => "Hand: drag to move the view",
    }
}

/// Builds the snapshot the views draw this frame.
pub fn build(state: &AppState) -> Presentation {
    let context = state.command_context();
    let editor = &state.editor;
    let viewport = editor.viewport();
    let selection = state.selected_indices();
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
            CommandId::SelectTool => pressed(view, editor.tool == Tool::Select),
            CommandId::PanTool => pressed(view, editor.tool == Tool::Hand),
            _ => view,
        })
        .collect();
    let active_diagram = editor
        .tabs
        .get(editor.active_tab)
        .map_or(0, |tab| tab.diagram);
    let diagram_name = |index: usize| {
        state
            .diagrams
            .get(index)
            .cloned()
            .unwrap_or_else(|| SAMPLE_DIAGRAM.to_owned())
    };

    Presentation {
        header: HeaderView {
            title: format!("{} · not saved", state.project_name),
            menu,
            commands: placed(Placement::Header, context),
        },
        tabs: TabsView {
            tabs: editor
                .tabs
                .iter()
                .enumerate()
                .map(|(index, tab)| TabView {
                    label: diagram_name(tab.diagram),
                    active: index == editor.active_tab,
                })
                .collect(),
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
            diagrams: (0..state.diagrams.len())
                .map(|index| RowView {
                    label: diagram_name(index),
                    selected: index == active_diagram,
                    target: None,
                })
                .collect(),
            model: state
                .classes
                .iter()
                .enumerate()
                .map(|(index, class)| RowView {
                    label: class.display_name().to_owned(),
                    selected: selection.contains(&index),
                    target: Some(Target::Class(index)),
                })
                .collect(),
        },
        canvas: canvas(state, &selection),
        inspector: InspectorView {
            title: "Selection",
            kind: inspector_kind(state),
            selection_count: selection.len(),
        },
        status: StatusView {
            save_text: SAVE_STATE,
            selection_text: match (selection.len(), state.selected_class()) {
                (0, _) | (_, None) => String::from("No selection"),
                (1, Some(class)) => format!("Selected: {}", class.display_name()),
                (count, Some(_)) => format!("{count} selected"),
            },
            theme: state.theme,
            zoom_text: format!("Zoom {}%", viewport.zoom_percent()),
            grid: editor.preferences.grid,
            snap: editor.preferences.snap,
            tool_hint: tool_hint(editor.effective_tool()),
        },
        dialog: dialog(state),
        command_search: CommandSearchView {
            open: state.palette.is_some(),
            query: state
                .palette
                .as_ref()
                .map(|search| search.query.clone())
                .unwrap_or_default(),
            results: state
                .palette
                .as_ref()
                .map(|search| {
                    commands::search(&search.query)
                        .into_iter()
                        .filter(|spec| spec.id != CommandId::SearchCommands)
                        .map(|spec| command_view(spec.id, context))
                        .collect()
                })
                .unwrap_or_default(),
        },
        object_search: object_search(state, &diagram_name(active_diagram)),
        diagram_types: diagram_core::notation::creatable()
            .map(|notation| notation.display_name)
            .collect(),
        focus_region: editor.focus_region,
        focus_request: state.pending_focus(),
    }
}

fn canvas(state: &AppState, selection: &[usize]) -> CanvasView {
    let editor = &state.editor;
    let viewport = editor.viewport();
    let to_pos = |point: diagram_core::Point| {
        // Screen positions fit comfortably in f32.
        #[expect(clippy::cast_possible_truncation, reason = "screen points")]
        let pos = Pos2::new(point.x() as f32, point.y() as f32);
        pos
    };
    let nodes = state
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| {
            let screen = viewport::class_rect(class).and_then(|rect| viewport.rect_to_screen(rect));
            let (position, size) = match screen {
                Some(rect) => {
                    #[expect(clippy::cast_possible_truncation, reason = "screen points")]
                    let size = Vec2::new(rect.size().width() as f32, rect.size().height() as f32);
                    (to_pos(rect.origin()), size)
                }
                None => (class.position.to_pos2(), class.size()),
            };
            NodeView {
                target: Target::Class(index),
                name: class.display_name().to_owned(),
                accessible_name: format!("Class {}", class.display_name()),
                attributes: class.attributes.clone(),
                position,
                size,
                selected: selection.contains(&index),
            }
        })
        .collect();
    CanvasView {
        accessible_name: "Diagram canvas",
        nodes,
        empty_text: state.classes.is_empty().then_some(EMPTY_CANVAS),
        zoom_percent: viewport.zoom_percent(),
        grid: editor.preferences.grid,
        marquee: match editor.gesture {
            Gesture::Marquee { start, current, .. } => Some((to_pos(start), to_pos(current))),
            _ => None,
        },
        tool: editor.effective_tool(),
    }
}

/// Text shown after an invalid edit could not be committed.
pub const RESOLVE_HINT: &str = "Fix the name, or press Escape to discard the edit.";

fn inspector_kind(state: &AppState) -> InspectorKind {
    let Some(index) = state.selected else {
        return InspectorKind::Nothing {
            guidance: NOTHING_SELECTED,
        };
    };
    let Some(class) = state.classes.get(index) else {
        return InspectorKind::Nothing {
            guidance: NOTHING_SELECTED,
        };
    };
    let draft = state.editor.draft.as_ref().filter(|draft| {
        draft.field == DraftField::ClassName && draft.target == Target::Class(index)
    });
    let (text, problem) = match draft {
        Some(draft) => {
            let problem = draft.problem().map(|problem| {
                if draft.commit_refused {
                    format!("{problem} {RESOLVE_HINT}")
                } else {
                    problem.to_owned()
                }
            });
            (draft.text.clone(), problem)
        }
        None => (
            class.name.clone(),
            class
                .name
                .trim()
                .is_empty()
                .then(|| BLANK_NAME_WARNING.to_owned()),
        ),
    };
    InspectorKind::Class {
        name: FieldView {
            label: "Name",
            text,
            problem,
        },
        attributes: class.attributes.clone(),
    }
}

fn dialog(state: &AppState) -> Option<DialogView> {
    if state.palette.is_some() {
        return Some(DialogView::CommandSearch);
    }
    let draft = state.editor.draft.as_ref()?;
    state.editor.pending.as_ref()?;
    let problem = draft.problem().unwrap_or_default();
    Some(DialogView::ResolveEdit {
        message: format!(
            "The name you typed cannot be saved: {problem} Return to the field to fix it, \
             or discard the edit to keep \"{}\".",
            draft.original
        ),
    })
}

/// Text when the project has no objects to search.
pub const NO_OBJECTS: &str = "No objects yet. Add a class to find it here.";
/// Text when no object matches the query.
pub const NO_MATCHES: &str = "No objects match.";

fn object_search(state: &AppState, location: &str) -> ObjectSearchView {
    let query = state.editor.object_search.clone().unwrap_or_default();
    let needle = query.trim().to_lowercase();
    let results: Vec<ObjectResultView> = state
        .classes
        .iter()
        .enumerate()
        .filter(|(_, class)| {
            needle.is_empty() || class.display_name().to_lowercase().contains(&needle)
        })
        .map(|(index, class)| ObjectResultView {
            label: format!("Class: {}", class.display_name()),
            location: location.to_owned(),
            target: Target::Class(index),
        })
        .collect();
    let empty_text = if state.classes.is_empty() {
        Some(NO_OBJECTS)
    } else if results.is_empty() {
        Some(NO_MATCHES)
    } else {
        None
    };
    ObjectSearchView {
        open: state.editor.object_search.is_some(),
        query,
        results,
        empty_text,
    }
}
