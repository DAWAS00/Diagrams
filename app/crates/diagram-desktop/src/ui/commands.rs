//! One command catalog for the toolbar, menus, command search and shortcuts (UI-06).
//!
//! Every surface that offers a command reads it from [`CATALOG`], so labels,
//! shortcuts and availability cannot drift apart. A disabled command always
//! carries the reason shown to the user.

use std::cmp::Reverse;

use eframe::egui::{InputState, Key, KeyboardShortcut, Modifiers};

/// Every command the application knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    /// Undo the last change.
    Undo,
    /// Redo the last undone change.
    Redo,
    /// Open the command and object search.
    SearchCommands,
    /// Save the project.
    Save,
    /// Export the current diagram.
    Export,
    /// Select tool: click objects to select them.
    SelectTool,
    /// Pan tool: drag to move the view.
    PanTool,
    /// Class tool: place a new class.
    AddClass,
    /// Association tool: connect two classes.
    Connect,
    /// Note tool: place a note.
    AddNote,
    /// Arrange the selected objects.
    Arrange,
    /// Create a diagram.
    NewDiagram,
    /// Show or hide the project panel (a drawer on narrow windows).
    ToggleNavigator,
    /// Show or hide the inspector (a drawer on narrow windows).
    ToggleInspector,
    /// Clear the current selection.
    ClearSelection,
    /// Show every reusable control in each state (design fixture).
    ShowComponentShowcase,
    /// Close the project and return to the welcome screen.
    CloseProject,
}

impl CommandId {
    /// Every command, in catalog order.
    pub const ALL: [Self; 17] = [
        Self::Undo,
        Self::Redo,
        Self::SearchCommands,
        Self::Save,
        Self::Export,
        Self::SelectTool,
        Self::PanTool,
        Self::AddClass,
        Self::Connect,
        Self::AddNote,
        Self::Arrange,
        Self::NewDiagram,
        Self::ToggleNavigator,
        Self::ToggleInspector,
        Self::ClearSelection,
        Self::ShowComponentShowcase,
        Self::CloseProject,
    ];

    /// This command's catalog entry.
    pub fn spec(self) -> &'static CommandSpec {
        &CATALOG[self as usize]
    }
}

/// Menu group a command belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandGroup {
    /// Project files and diagrams.
    File,
    /// Changes and selection.
    Edit,
    /// Panels and layout.
    View,
    /// Canvas tools.
    Tools,
    /// Finding commands and objects.
    Navigate,
}

impl CommandGroup {
    /// Groups in menu order.
    pub const ALL: [Self; 5] = [
        Self::File,
        Self::Edit,
        Self::View,
        Self::Tools,
        Self::Navigate,
    ];

    /// Name shown in menus and in search results.
    pub const fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Tools => "Tools",
            Self::Navigate => "Navigate",
        }
    }
}

/// Where a command appears besides the menu and the search, which list every command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    /// A button in the header.
    Header,
    /// A tool in the tool row above the canvas.
    ToolRow,
    /// A button in the diagram tab row.
    TabRow,
    /// A labeled panel button at the ends of the tool row.
    PanelToggle,
    /// Only in the menu, the search and its shortcut.
    MenuOnly,
}

/// Whether a command can run now, and if not, why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// The command can run.
    Enabled,
    /// The command cannot run; the text explains why to the user.
    Disabled(&'static str),
}

impl Availability {
    /// True when the command can run.
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// The reason a disabled command cannot run.
    pub const fn reason(self) -> Option<&'static str> {
        match self {
            Self::Enabled => None,
            Self::Disabled(reason) => Some(reason),
        }
    }
}

/// One catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    /// The command.
    pub id: CommandId,
    /// Label on buttons, menus and search results.
    pub label: &'static str,
    /// Menu group.
    pub group: CommandGroup,
    /// Keyboard shortcut, if any.
    pub shortcut: Option<KeyboardShortcut>,
    /// True when a focused text field uses the same keys (for example Ctrl+Z
    /// or Escape). The field then keeps them and the command does not run.
    pub text_field_owns_shortcut: bool,
    /// Where the command appears outside the menu and the search.
    pub placement: Placement,
}

const fn spec(
    id: CommandId,
    label: &'static str,
    group: CommandGroup,
    shortcut: Option<KeyboardShortcut>,
    text_field_owns_shortcut: bool,
    placement: Placement,
) -> CommandSpec {
    CommandSpec {
        id,
        label,
        group,
        shortcut,
        text_field_owns_shortcut,
        placement,
    }
}

const fn keys(modifiers: Modifiers, key: Key) -> Option<KeyboardShortcut> {
    Some(KeyboardShortcut::new(modifiers, key))
}

/// The catalog, in [`CommandId`] order.
pub const CATALOG: [CommandSpec; 17] = [
    spec(
        CommandId::Undo,
        "Undo",
        CommandGroup::Edit,
        keys(Modifiers::COMMAND, Key::Z),
        true,
        Placement::Header,
    ),
    spec(
        CommandId::Redo,
        "Redo",
        CommandGroup::Edit,
        keys(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z),
        true,
        Placement::Header,
    ),
    spec(
        CommandId::SearchCommands,
        "Search",
        CommandGroup::Navigate,
        keys(Modifiers::COMMAND, Key::K),
        false,
        Placement::Header,
    ),
    spec(
        CommandId::Save,
        "Save",
        CommandGroup::File,
        keys(Modifiers::COMMAND, Key::S),
        false,
        Placement::Header,
    ),
    spec(
        CommandId::Export,
        "Export",
        CommandGroup::File,
        None,
        false,
        Placement::Header,
    ),
    spec(
        CommandId::SelectTool,
        "Select",
        CommandGroup::Tools,
        None,
        false,
        Placement::ToolRow,
    ),
    spec(
        CommandId::PanTool,
        "Pan",
        CommandGroup::Tools,
        None,
        false,
        Placement::ToolRow,
    ),
    spec(
        CommandId::AddClass,
        "Class",
        CommandGroup::Tools,
        None,
        false,
        Placement::ToolRow,
    ),
    spec(
        CommandId::Connect,
        "Association",
        CommandGroup::Tools,
        None,
        false,
        Placement::ToolRow,
    ),
    spec(
        CommandId::AddNote,
        "Note",
        CommandGroup::Tools,
        None,
        false,
        Placement::ToolRow,
    ),
    spec(
        CommandId::Arrange,
        "Arrange",
        CommandGroup::Tools,
        None,
        false,
        Placement::ToolRow,
    ),
    spec(
        CommandId::NewDiagram,
        "New diagram",
        CommandGroup::File,
        None,
        false,
        Placement::TabRow,
    ),
    spec(
        CommandId::ToggleNavigator,
        "Project panel",
        CommandGroup::View,
        None,
        false,
        Placement::PanelToggle,
    ),
    spec(
        CommandId::ToggleInspector,
        "Inspector",
        CommandGroup::View,
        None,
        false,
        Placement::PanelToggle,
    ),
    spec(
        CommandId::ClearSelection,
        "Clear selection",
        CommandGroup::Edit,
        keys(Modifiers::NONE, Key::Escape),
        true,
        Placement::MenuOnly,
    ),
    spec(
        CommandId::ShowComponentShowcase,
        "Component showcase",
        CommandGroup::View,
        None,
        false,
        Placement::MenuOnly,
    ),
    spec(
        CommandId::CloseProject,
        "Close project",
        CommandGroup::File,
        None,
        false,
        Placement::MenuOnly,
    ),
];

/// The state that command availability depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandContext {
    /// Something is selected on the canvas.
    pub has_selection: bool,
    /// A project is open.
    pub has_project: bool,
    /// The history has an action to undo.
    pub can_undo: bool,
    /// The history has an action to redo.
    pub can_redo: bool,
}

/// Whether `id` can run in `context`, with the reason when it cannot.
pub fn availability(id: CommandId, context: CommandContext) -> Availability {
    use Availability::{Disabled, Enabled};
    match id {
        CommandId::Undo if context.can_undo => Enabled,
        CommandId::Undo => Disabled("Nothing to undo yet."),
        CommandId::Redo if context.can_redo => Enabled,
        CommandId::Redo => Disabled("Nothing to redo."),
        CommandId::CloseProject if context.has_project => Enabled,
        CommandId::CloseProject => Disabled("No project is open."),
        CommandId::Save => Disabled("Not available yet: this prototype cannot save."),
        CommandId::Export => Disabled("Not available yet: SVG export is planned for v0.1."),
        CommandId::PanTool => {
            Disabled("Not available yet: panning arrives with canvas scrolling and zoom.")
        }
        CommandId::AddClass => {
            Disabled("Not available yet: adding classes needs the project model.")
        }
        CommandId::Connect => Disabled("Not available yet: relationships need the project model."),
        CommandId::AddNote => Disabled("Not available yet: notes need the project model."),
        CommandId::Arrange => Disabled("Not available yet: arranging needs movable objects."),
        CommandId::NewDiagram => {
            Disabled("Not available yet: new diagrams need the project model.")
        }
        CommandId::SearchCommands
        | CommandId::SelectTool
        | CommandId::ToggleNavigator
        | CommandId::ToggleInspector
        | CommandId::ShowComponentShowcase => Enabled,
        CommandId::ClearSelection if context.has_selection => Enabled,
        CommandId::ClearSelection => Disabled("Nothing is selected."),
    }
}

/// Commands shown in `placement`, in catalog order.
pub fn placed(placement: Placement) -> impl Iterator<Item = &'static CommandSpec> {
    CATALOG
        .iter()
        .filter(move |spec| spec.placement == placement)
}

/// Commands grouped for menus, in menu order; empty groups are left out.
pub fn menu_groups() -> Vec<(CommandGroup, Vec<&'static CommandSpec>)> {
    CommandGroup::ALL
        .into_iter()
        .map(|group| {
            let specs = CATALOG.iter().filter(|spec| spec.group == group).collect();
            (group, specs)
        })
        .filter(|(_, specs): &(CommandGroup, Vec<_>)| !specs.is_empty())
        .collect()
}

/// Commands whose label or group contains `query`, ignoring case. An empty
/// query returns every command.
pub fn search(query: &str) -> Vec<&'static CommandSpec> {
    let query = query.trim().to_lowercase();
    CATALOG
        .iter()
        .filter(|spec| {
            query.is_empty()
                || spec.label.to_lowercase().contains(&query)
                || spec.group.label().to_lowercase().contains(&query)
        })
        .collect()
}

fn modifier_count(shortcut: &KeyboardShortcut) -> u8 {
    let modifiers = shortcut.modifiers;
    u8::from(modifiers.alt)
        + u8::from(modifiers.shift)
        + u8::from(modifiers.command || modifiers.ctrl || modifiers.mac_cmd)
}

/// Consumes the first catalog shortcut pressed this frame and returns its command.
///
/// More specific shortcuts are checked first, so Ctrl+Shift+Z is never read as
/// Ctrl+Z. When a text field had focus in the previous frame, shortcuts that
/// the field uses are left for the field: egui drops text focus in the same
/// frame that Escape is pressed, so the current frame's focus is not reliable.
pub fn take_shortcut(input: &mut InputState, text_had_focus: bool) -> Option<CommandId> {
    let mut candidates: Vec<(&CommandSpec, KeyboardShortcut)> = CATALOG
        .iter()
        .filter(|spec| !(text_had_focus && spec.text_field_owns_shortcut))
        .filter_map(|spec| spec.shortcut.map(|shortcut| (spec, shortcut)))
        .collect();
    candidates.sort_by_key(|(_, shortcut)| Reverse(modifier_count(shortcut)));
    candidates
        .into_iter()
        .find(|(_, shortcut)| input.consume_shortcut(shortcut))
        .map(|(spec, _)| spec.id)
}

/// Who carries out a command's effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectOwner {
    /// Editor state only: selection, panels, search, tools, windows.
    Editor,
    /// The project model, through validated commands (M1).
    ProjectModel,
    /// A background worker job such as saving or exporting (M1/M3).
    Worker,
}

/// How a command relates to undo history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoPolicy {
    /// Changes editor state only; not recorded in the project history.
    NotRecorded,
    /// One accepted edit is one undo step.
    OneStep,
    /// Moves through the history itself (Undo, Redo).
    MovesHistory,
    /// Does not change the project (save, export).
    NoProjectChange,
}

/// The contract every visible action has (plan section 5): when it can run,
/// what it changes, who owns that change, its undo policy, and the test that
/// proves its current behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionContract {
    /// What must be true before it can run.
    pub prerequisite: &'static str,
    /// What it changes.
    pub effect: &'static str,
    /// Who carries out the change.
    pub owner: EffectOwner,
    /// How it relates to undo.
    pub undo: UndoPolicy,
    /// Name of the test function that covers its current behaviour.
    pub test: &'static str,
}

/// The contract of `id`.
pub fn contract(id: CommandId) -> ActionContract {
    use EffectOwner::{Editor, ProjectModel, Worker};
    use UndoPolicy::{MovesHistory, NoProjectChange, NotRecorded, OneStep};
    let c = |prerequisite, effect, owner, undo, test| ActionContract {
        prerequisite,
        effect,
        owner,
        undo,
        test,
    };
    match id {
        CommandId::Undo => c(
            "A change to undo in the project history",
            "Restores the previous project state",
            ProjectModel,
            MovesHistory,
            "undo_and_redo_should_follow_the_project_history",
        ),
        CommandId::Redo => c(
            "An undone change to redo",
            "Re-applies the undone change",
            ProjectModel,
            MovesHistory,
            "undo_and_redo_should_follow_the_project_history",
        ),
        CommandId::SearchCommands => c(
            "None",
            "Opens the command and object search; focus returns when it closes",
            Editor,
            NotRecorded,
            "ctrl_k_should_open_search_with_the_field_focused",
        ),
        CommandId::Save => c(
            "Project storage and a project file (M1)",
            "Writes a snapshot through the save worker; dirty state follows the acknowledgment",
            Worker,
            NoProjectChange,
            "unavailable_shortcuts_should_change_nothing",
        ),
        CommandId::Export => c(
            "SVG export (v0.1, M3)",
            "Writes the current diagram as SVG through a worker",
            Worker,
            NoProjectChange,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::SelectTool => c(
            "None",
            "Makes Select the active tool",
            Editor,
            NotRecorded,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::PanTool => c(
            "Canvas scrolling and zoom (UI-07)",
            "Makes Pan the active tool",
            Editor,
            NotRecorded,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::AddClass => c(
            "The project model (M1)",
            "Creates a class and its view on the current diagram",
            ProjectModel,
            OneStep,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::Connect => c(
            "The project model (M1)",
            "Creates a relationship and its view",
            ProjectModel,
            OneStep,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::AddNote => c(
            "The project model (M1)",
            "Creates a note on the current diagram",
            ProjectModel,
            OneStep,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::Arrange => c(
            "Movable objects (M2)",
            "Moves the selected views",
            ProjectModel,
            OneStep,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::NewDiagram => c(
            "The project model (M1)",
            "Creates a diagram and opens it in a tab",
            ProjectModel,
            OneStep,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::ToggleNavigator => c(
            "None",
            "Shows or hides the project panel, or opens its drawer on narrow windows",
            Editor,
            NotRecorded,
            "hidden_panel_should_stay_hidden_after_a_narrow_phase",
        ),
        CommandId::ToggleInspector => c(
            "None",
            "Shows or hides the inspector, or opens its drawer on narrow windows",
            Editor,
            NotRecorded,
            "inspector_drawer_should_cover_only_the_canvas",
        ),
        CommandId::ClearSelection => c(
            "Something is selected",
            "Clears the selection on the current diagram",
            Editor,
            NotRecorded,
            "escape_should_clear_the_selection_when_not_typing",
        ),
        CommandId::ShowComponentShowcase => c(
            "None",
            "Opens the component showcase design fixture",
            Editor,
            NotRecorded,
            "search_should_open_the_showcase",
        ),
        CommandId::CloseProject => c(
            "A project is open",
            "Closes the project and returns to the welcome screen; asks first when work would be lost",
            Editor,
            NoProjectChange,
            "close_project_should_ask_first_when_there_are_unsaved_changes",
        ),
    }
}
