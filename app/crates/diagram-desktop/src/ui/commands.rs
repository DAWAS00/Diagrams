//! One command catalog for the toolbar, menus, command search and shortcuts (UI-06).
//!
//! Every surface that offers a command reads it from [`CATALOG`], so labels,
//! shortcuts and availability cannot drift apart. A disabled command always
//! carries the reason shown to the user. Shortcuts are dispatched by
//! [`take_shortcut_in`] in the priority order of the functional spec §8:
//! modal → IME or text field → focused region → application command.

use std::cmp::Reverse;

use eframe::egui::{Event, ImeEvent, InputState, Key, KeyboardShortcut, ModifierNames, Modifiers};

use crate::editor_state::ShellSupport;
use crate::intent::{InputContext, Region};

/// Every command the application knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    /// Undo the last change.
    Undo,
    /// Redo the last undone change.
    Redo,
    /// Open the command search.
    SearchCommands,
    /// Save the project.
    Save,
    /// Export the current diagram as SVG.
    Export,
    /// Select tool: click objects to select them.
    SelectTool,
    /// Hand tool: drag to move the view (decision D3; formerly "Pan").
    HandTool,
    /// Class tool: place a new class.
    AddClass,
    /// Relationship tool: connect two classes.
    Connect,
    /// Arrange menu for the selected views.
    Arrange,
    /// Add a diagram to the project.
    NewDiagram,
    /// Show or hide the project panel (a drawer on narrow windows).
    ToggleNavigator,
    /// Show or hide the inspector (a drawer on narrow windows).
    ToggleInspector,
    /// Clear the current selection.
    ClearSelection,
    /// Show every reusable control in each state (design fixture).
    ShowComponentShowcase,
    /// Create a new project.
    NewProject,
    /// Open a project file.
    OpenProject,
    /// Save the project to a chosen file.
    SaveAs,
    /// Close the project.
    CloseProject,
    /// Review a recovered project snapshot.
    RecoverProject,
    /// Rename the active diagram.
    RenameDiagram,
    /// Close the active diagram tab; the diagram stays in the project.
    CloseTab,
    /// Switch to the next diagram tab.
    NextTab,
    /// Switch to the previous diagram tab.
    PreviousTab,
    /// Delete the active diagram and its views.
    DeleteDiagram,
    /// Select every view in the active diagram.
    SelectAll,
    /// Remove the selected views from this diagram only.
    RemoveView,
    /// Rename the selected model element.
    Rename,
    /// Edit the position and size of the selected views.
    EditGeometry,
    /// Edit the appearance of the selected views.
    EditAppearance,
    /// Add an attribute to the selected class.
    AddAttribute,
    /// Add an operation to the selected class.
    AddOperation,
    /// Move the selected member up.
    MoveMemberUp,
    /// Move the selected member down.
    MoveMemberDown,
    /// Remove the selected member.
    RemoveMember,
    /// Edit the ends (roles, multiplicities) of the selected relationship.
    EditRelationEnds,
    /// Show an existing model element in another diagram.
    ShowInDiagram,
    /// Create an independent copy of the selected class.
    DuplicateModel,
    /// Delete the selected model element from every diagram.
    DeleteModelEverywhere,
    /// Choose the relationship kind (association or generalization).
    RelationshipKind,
    /// Align left edges.
    AlignLeft,
    /// Align horizontal centers.
    AlignCenter,
    /// Align right edges.
    AlignRight,
    /// Align top edges.
    AlignTop,
    /// Align vertical middles.
    AlignMiddle,
    /// Align bottom edges.
    AlignBottom,
    /// Equal horizontal gaps.
    DistributeHorizontally,
    /// Equal vertical gaps.
    DistributeVertically,
    /// Zoom in.
    ZoomIn,
    /// Zoom out.
    ZoomOut,
    /// Zoom to 100%.
    ActualSize,
    /// Fit the whole diagram in the view.
    FitAll,
    /// Fit the selection in the view.
    FitSelection,
    /// Show or hide the grid.
    ToggleGrid,
    /// Turn snapping to the grid on or off.
    ToggleSnap,
    /// Open the validation issues drawer.
    ShowIssues,
    /// Open preferences.
    Preferences,
    /// Find an object by name.
    FindObject,
    /// Reveal the selected element's view in the active diagram.
    Reveal,
    /// Move keyboard focus to the next workspace region.
    NextRegion,
    /// Move keyboard focus to the previous workspace region.
    PreviousRegion,
    /// Open the context menu for the focused object.
    ContextMenu,
    /// Help and keyboard shortcuts.
    HelpShortcuts,
    /// Cancel the innermost thing in progress (Escape): a dialog or draft,
    /// then a gesture or tool, then the selection, one layer per press.
    Cancel,
}

impl CommandId {
    /// Every command, in catalog order.
    pub const ALL: [Self; 64] = [
        Self::Undo,
        Self::Redo,
        Self::SearchCommands,
        Self::Save,
        Self::Export,
        Self::SelectTool,
        Self::HandTool,
        Self::AddClass,
        Self::Connect,
        Self::Arrange,
        Self::NewDiagram,
        Self::ToggleNavigator,
        Self::ToggleInspector,
        Self::ClearSelection,
        Self::ShowComponentShowcase,
        Self::NewProject,
        Self::OpenProject,
        Self::SaveAs,
        Self::CloseProject,
        Self::RecoverProject,
        Self::RenameDiagram,
        Self::CloseTab,
        Self::NextTab,
        Self::PreviousTab,
        Self::DeleteDiagram,
        Self::SelectAll,
        Self::RemoveView,
        Self::Rename,
        Self::EditGeometry,
        Self::EditAppearance,
        Self::AddAttribute,
        Self::AddOperation,
        Self::MoveMemberUp,
        Self::MoveMemberDown,
        Self::RemoveMember,
        Self::EditRelationEnds,
        Self::ShowInDiagram,
        Self::DuplicateModel,
        Self::DeleteModelEverywhere,
        Self::RelationshipKind,
        Self::AlignLeft,
        Self::AlignCenter,
        Self::AlignRight,
        Self::AlignTop,
        Self::AlignMiddle,
        Self::AlignBottom,
        Self::DistributeHorizontally,
        Self::DistributeVertically,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::ActualSize,
        Self::FitAll,
        Self::FitSelection,
        Self::ToggleGrid,
        Self::ToggleSnap,
        Self::ShowIssues,
        Self::Preferences,
        Self::FindObject,
        Self::Reveal,
        Self::NextRegion,
        Self::PreviousRegion,
        Self::ContextMenu,
        Self::HelpShortcuts,
        Self::Cancel,
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
    /// Classes, members and relationships (shared model).
    Model,
    /// Panels, zoom and layout.
    View,
    /// Canvas tools and arranging.
    Tools,
    /// Finding commands, objects and regions.
    Navigate,
    /// Help and shortcuts.
    Help,
}

impl CommandGroup {
    /// Groups in menu order.
    pub const ALL: [Self; 7] = [
        Self::File,
        Self::Edit,
        Self::Model,
        Self::View,
        Self::Tools,
        Self::Navigate,
        Self::Help,
    ];

    /// Name shown in menus and in search results.
    pub const fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::Model => "Model",
            Self::View => "View",
            Self::Tools => "Tools",
            Self::Navigate => "Navigate",
            Self::Help => "Help",
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

/// Where a command's shortcut works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// Anywhere outside a modal (unless a text field owns the keys).
    Application,
    /// Only while the board has focus, nobody is typing or composing text
    /// and no modal is open. Single letters, digits, Delete and Ctrl+A.
    Canvas,
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
    /// Keyboard shortcut shown to the user, if any.
    pub shortcut: Option<KeyboardShortcut>,
    /// Further keys that run the command (for example Ctrl+Shift+P for search,
    /// or Ctrl+= for zoom in). They are not shown separately.
    pub alternate_shortcuts: &'static [KeyboardShortcut],
    /// True when a focused text field uses the same keys (for example Ctrl+Z
    /// or Escape). The field then keeps them and the command does not run.
    pub text_field_owns_shortcut: bool,
    /// Where the shortcut works.
    pub scope: ShortcutScope,
    /// Where the command appears outside the menu and the search.
    pub placement: Placement,
    /// Other words command search accepts for this command; results always
    /// show the canonical label.
    pub aliases: &'static [&'static str],
}

impl CommandSpec {
    /// Every key combination that runs the command.
    pub fn all_shortcuts(&self) -> impl Iterator<Item = KeyboardShortcut> + '_ {
        self.shortcut
            .into_iter()
            .chain(self.alternate_shortcuts.iter().copied())
    }

    const fn keys(self, modifiers: Modifiers, key: Key) -> Self {
        Self {
            shortcut: Some(KeyboardShortcut::new(modifiers, key)),
            ..self
        }
    }

    const fn also(self, alternate_shortcuts: &'static [KeyboardShortcut]) -> Self {
        Self {
            alternate_shortcuts,
            ..self
        }
    }

    const fn text_owns(self) -> Self {
        Self {
            text_field_owns_shortcut: true,
            ..self
        }
    }

    const fn canvas(self) -> Self {
        Self {
            scope: ShortcutScope::Canvas,
            ..self
        }
    }

    const fn aka(self, aliases: &'static [&'static str]) -> Self {
        Self { aliases, ..self }
    }
}

const fn spec(
    id: CommandId,
    label: &'static str,
    group: CommandGroup,
    placement: Placement,
) -> CommandSpec {
    CommandSpec {
        id,
        label,
        group,
        shortcut: None,
        alternate_shortcuts: &[],
        text_field_owns_shortcut: false,
        scope: ShortcutScope::Application,
        placement,
        aliases: &[],
    }
}

const CTRL: Modifiers = Modifiers::COMMAND;
const CTRL_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);
const SHIFT: Modifiers = Modifiers::SHIFT;
const NONE: Modifiers = Modifiers::NONE;

use CommandGroup as G;
use CommandId as C;
use Placement as P;

/// The catalog, in [`CommandId`] order.
pub const CATALOG: [CommandSpec; 64] = [
    spec(C::Undo, "Undo", G::Edit, P::Header)
        .keys(CTRL, Key::Z)
        .text_owns(),
    spec(C::Redo, "Redo", G::Edit, P::Header)
        .keys(CTRL_SHIFT, Key::Z)
        .text_owns(),
    // D1: Ctrl+K stays and Ctrl+Shift+P is added.
    spec(C::SearchCommands, "Search", G::Navigate, P::Header)
        .keys(CTRL, Key::K)
        .also(&[KeyboardShortcut::new(CTRL_SHIFT, Key::P)])
        .aka(&["command palette", "commands", "actions"]),
    spec(C::Save, "Save", G::File, P::Header).keys(CTRL, Key::S),
    spec(C::Export, "Export", G::File, P::Header)
        .keys(CTRL_SHIFT, Key::E)
        .aka(&["svg", "export svg", "image"]),
    spec(C::SelectTool, "Select", G::Tools, P::ToolRow)
        .keys(NONE, Key::V)
        .canvas()
        .aka(&["pointer", "arrow"]),
    // D3: "Pan" is now "Hand".
    spec(C::HandTool, "Hand", G::Tools, P::ToolRow)
        .keys(NONE, Key::H)
        .canvas()
        .aka(&["pan", "move view", "scroll"]),
    spec(C::AddClass, "Class", G::Tools, P::ToolRow)
        .keys(NONE, Key::C)
        .canvas()
        .aka(&["add class", "new class", "create class"]),
    spec(C::Connect, "Association", G::Tools, P::ToolRow)
        .keys(NONE, Key::R)
        .canvas()
        .aka(&["relationship", "connect", "link", "edge"]),
    spec(C::Arrange, "Arrange", G::Tools, P::ToolRow).aka(&["align", "distribute", "layout"]),
    spec(C::NewDiagram, "New diagram", G::File, P::TabRow)
        .keys(CTRL_SHIFT, Key::N)
        .aka(&["add diagram", "diagram"]),
    spec(C::ToggleNavigator, "Project panel", G::View, P::PanelToggle).aka(&[
        "navigator",
        "sidebar",
        "model list",
    ]),
    spec(C::ToggleInspector, "Inspector", G::View, P::PanelToggle).aka(&["properties", "details"]),
    spec(C::ClearSelection, "Clear selection", G::Edit, P::MenuOnly).aka(&["deselect"]),
    spec(
        C::ShowComponentShowcase,
        "Component showcase",
        G::View,
        P::MenuOnly,
    )
    .aka(&["design fixture", "controls"]),
    spec(C::NewProject, "New project", G::File, P::MenuOnly).keys(CTRL, Key::N),
    spec(C::OpenProject, "Open project", G::File, P::MenuOnly)
        .keys(CTRL, Key::O)
        .aka(&["open file", "load"]),
    spec(C::SaveAs, "Save As", G::File, P::MenuOnly)
        .keys(CTRL_SHIFT, Key::S)
        .aka(&["save copy"]),
    spec(C::CloseProject, "Close project", G::File, P::MenuOnly).aka(&["quit project"]),
    spec(C::RecoverProject, "Review recovery", G::File, P::MenuOnly)
        .aka(&["recover", "restore", "autosave"]),
    spec(C::RenameDiagram, "Rename diagram", G::File, P::MenuOnly),
    spec(C::CloseTab, "Close tab", G::File, P::MenuOnly)
        .keys(CTRL, Key::W)
        .aka(&["close diagram tab"]),
    spec(C::NextTab, "Next tab", G::Navigate, P::MenuOnly)
        .keys(CTRL, Key::Tab)
        .aka(&["switch diagram", "next diagram"]),
    spec(C::PreviousTab, "Previous tab", G::Navigate, P::MenuOnly)
        .keys(CTRL_SHIFT, Key::Tab)
        .aka(&["switch diagram", "previous diagram"]),
    spec(C::DeleteDiagram, "Delete diagram", G::File, P::MenuOnly),
    spec(C::SelectAll, "Select all", G::Edit, P::MenuOnly)
        .keys(CTRL, Key::A)
        .text_owns()
        .canvas(),
    spec(C::RemoveView, "Remove from diagram", G::Edit, P::MenuOnly)
        .keys(NONE, Key::Delete)
        .text_owns()
        .canvas()
        .aka(&["remove view", "delete"]),
    spec(C::Rename, "Rename", G::Edit, P::MenuOnly)
        .keys(NONE, Key::F2)
        .canvas()
        .aka(&["rename class", "edit name"]),
    spec(C::EditGeometry, "Position and size", G::Edit, P::MenuOnly)
        .aka(&["move", "resize", "geometry", "nudge"]),
    spec(C::EditAppearance, "Appearance", G::Edit, P::MenuOnly)
        .aka(&["style", "color", "fill", "border"]),
    spec(C::AddAttribute, "Add attribute", G::Model, P::MenuOnly).aka(&[
        "attribute",
        "field",
        "property",
        "member",
    ]),
    spec(C::AddOperation, "Add operation", G::Model, P::MenuOnly).aka(&[
        "operation",
        "method",
        "function",
        "member",
    ]),
    spec(C::MoveMemberUp, "Move member up", G::Model, P::MenuOnly)
        .aka(&["reorder", "attribute up"]),
    spec(C::MoveMemberDown, "Move member down", G::Model, P::MenuOnly)
        .aka(&["reorder", "attribute down"]),
    spec(C::RemoveMember, "Remove member", G::Model, P::MenuOnly)
        .aka(&["delete attribute", "delete operation"]),
    spec(
        C::EditRelationEnds,
        "Relationship ends",
        G::Model,
        P::MenuOnly,
    )
    .aka(&["multiplicity", "role", "ends"]),
    spec(C::ShowInDiagram, "Show in diagram", G::Model, P::MenuOnly).aka(&[
        "reuse",
        "add existing",
        "place existing",
    ]),
    spec(C::DuplicateModel, "Duplicate model", G::Model, P::MenuOnly).aka(&[
        "copy",
        "clone",
        "duplicate",
    ]),
    spec(
        C::DeleteModelEverywhere,
        "Delete model everywhere",
        G::Model,
        P::MenuOnly,
    )
    .aka(&["delete", "remove model"]),
    spec(
        C::RelationshipKind,
        "Relationship kind",
        G::Tools,
        P::MenuOnly,
    )
    .aka(&["generalization", "inheritance", "association"]),
    spec(C::AlignLeft, "Align left", G::Tools, P::MenuOnly).aka(&["arrange"]),
    spec(C::AlignCenter, "Align center", G::Tools, P::MenuOnly).aka(&["arrange"]),
    spec(C::AlignRight, "Align right", G::Tools, P::MenuOnly).aka(&["arrange"]),
    spec(C::AlignTop, "Align top", G::Tools, P::MenuOnly).aka(&["arrange"]),
    spec(C::AlignMiddle, "Align middle", G::Tools, P::MenuOnly).aka(&["arrange"]),
    spec(C::AlignBottom, "Align bottom", G::Tools, P::MenuOnly).aka(&["arrange"]),
    spec(
        C::DistributeHorizontally,
        "Distribute horizontally",
        G::Tools,
        P::MenuOnly,
    )
    .aka(&["arrange", "spacing"]),
    spec(
        C::DistributeVertically,
        "Distribute vertically",
        G::Tools,
        P::MenuOnly,
    )
    .aka(&["arrange", "spacing"]),
    spec(C::ZoomIn, "Zoom in", G::View, P::MenuOnly)
        .keys(CTRL, Key::Plus)
        .also(&[KeyboardShortcut::new(CTRL, Key::Equals)])
        .aka(&["magnify", "larger"]),
    spec(C::ZoomOut, "Zoom out", G::View, P::MenuOnly)
        .keys(CTRL, Key::Minus)
        .aka(&["smaller"]),
    spec(C::ActualSize, "Actual size", G::View, P::MenuOnly)
        .keys(CTRL, Key::Num0)
        .aka(&["100%", "reset zoom", "zoom"]),
    spec(C::FitAll, "Fit all", G::View, P::MenuOnly)
        .keys(SHIFT, Key::Num1)
        .also(&[KeyboardShortcut::new(SHIFT, Key::Exclamationmark)])
        .canvas()
        .aka(&["fit", "zoom to fit", "show all"]),
    spec(C::FitSelection, "Fit selection", G::View, P::MenuOnly)
        .keys(SHIFT, Key::Num2)
        .canvas()
        .aka(&["fit", "zoom to selection"]),
    spec(C::ToggleGrid, "Grid", G::View, P::MenuOnly)
        .keys(NONE, Key::G)
        .canvas()
        .aka(&["show grid"]),
    spec(C::ToggleSnap, "Snap to grid", G::View, P::MenuOnly)
        .keys(SHIFT, Key::G)
        .canvas()
        .aka(&["snap"]),
    spec(C::ShowIssues, "Validation issues", G::View, P::MenuOnly).aka(&[
        "issues",
        "problems",
        "errors",
        "warnings",
        "validation",
    ]),
    spec(C::Preferences, "Preferences", G::View, P::MenuOnly)
        .keys(CTRL, Key::Comma)
        .aka(&["settings", "options", "theme", "text scale"]),
    spec(C::FindObject, "Find object", G::Navigate, P::MenuOnly)
        .keys(CTRL, Key::F)
        .text_owns()
        .aka(&["find", "search objects", "model search"]),
    spec(C::Reveal, "Reveal in diagram", G::Navigate, P::MenuOnly)
        .aka(&["reveal", "go to", "locate"]),
    spec(C::NextRegion, "Next region", G::Navigate, P::MenuOnly)
        .keys(NONE, Key::F6)
        .aka(&["focus", "panel"]),
    spec(
        C::PreviousRegion,
        "Previous region",
        G::Navigate,
        P::MenuOnly,
    )
    .keys(SHIFT, Key::F6)
    .aka(&["focus", "panel"]),
    spec(C::ContextMenu, "Context menu", G::Navigate, P::MenuOnly)
        .keys(SHIFT, Key::F10)
        .aka(&["actions menu", "right click"]),
    spec(C::HelpShortcuts, "Help and shortcuts", G::Help, P::MenuOnly)
        .keys(NONE, Key::F1)
        .aka(&["keyboard", "shortcuts", "help", "keys"]),
    spec(C::Cancel, "Cancel", G::Edit, P::MenuOnly)
        .keys(NONE, Key::Escape)
        .text_owns()
        .aka(&["escape", "stop", "deselect"]),
];

/// The state that command availability depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandContext {
    /// Something is selected on the canvas.
    pub has_selection: bool,
    /// Number of selected views.
    pub selection_count: usize,
    /// The active diagram has at least one view.
    pub has_content: bool,
    /// Number of open diagram tabs.
    pub tab_count: usize,
    /// Zoom is below the maximum.
    pub can_zoom_in: bool,
    /// Zoom is above the minimum.
    pub can_zoom_out: bool,
    /// Escape has something to cancel: a dialog, draft, gesture, tool or selection.
    pub can_cancel: bool,
    /// What the running shell draws and sends.
    pub support: ShellSupport,
}

/// Explains every command that changes the project model.
pub const NEEDS_MODEL: &str = "Not available yet: this needs the project model (milestone M1).";

/// Whether `id` can run in `context`, with the reason when it cannot.
pub fn availability(id: CommandId, context: CommandContext) -> Availability {
    use Availability::{Disabled, Enabled};
    match id {
        CommandId::Undo => Disabled("Not available yet: undo needs the project history."),
        CommandId::Redo => Disabled("Not available yet: redo needs the project history."),
        CommandId::Save => Disabled("Not available yet: this prototype cannot save."),
        CommandId::SaveAs => {
            Disabled("Not available yet: Save As arrives with project storage (R1-06).")
        }
        CommandId::Export => Disabled("Not available yet: SVG export is planned for v0.1."),
        CommandId::OpenProject => {
            Disabled("Not available yet: opening project files arrives with project storage.")
        }
        CommandId::RecoverProject => {
            Disabled("Not available yet: recovery arrives with project storage.")
        }
        CommandId::NewProject | CommandId::CloseProject => Disabled(
            "Not available yet: replacing the open project needs the unsaved-changes dialog.",
        ),
        CommandId::AddClass => {
            Disabled("Not available yet: adding classes needs the project model.")
        }
        CommandId::Connect | CommandId::RelationshipKind | CommandId::EditRelationEnds => {
            Disabled("Not available yet: relationships need the project model.")
        }
        CommandId::Arrange
        | CommandId::AlignLeft
        | CommandId::AlignCenter
        | CommandId::AlignRight
        | CommandId::AlignTop
        | CommandId::AlignMiddle
        | CommandId::AlignBottom
        | CommandId::DistributeHorizontally
        | CommandId::DistributeVertically => {
            Disabled("Not available yet: arranging needs movable objects.")
        }
        CommandId::NewDiagram | CommandId::RenameDiagram | CommandId::DeleteDiagram => {
            Disabled("Not available yet: new diagrams need the project model.")
        }
        CommandId::AddAttribute
        | CommandId::AddOperation
        | CommandId::MoveMemberUp
        | CommandId::MoveMemberDown
        | CommandId::RemoveMember => {
            Disabled("Not available yet: editing members needs the project model.")
        }
        CommandId::ShowInDiagram
        | CommandId::DuplicateModel
        | CommandId::DeleteModelEverywhere
        | CommandId::RemoveView
        | CommandId::EditGeometry
        | CommandId::EditAppearance => Disabled(NEEDS_MODEL),
        CommandId::ShowIssues => {
            Disabled("Not available yet: validation issues need the project model.")
        }
        CommandId::CloseTab if context.tab_count > 1 => Enabled,
        CommandId::CloseTab => Disabled(
            "Not available yet: the last tab stays open until the no-diagram screen exists.",
        ),
        CommandId::NextTab | CommandId::PreviousTab if context.tab_count > 1 => Enabled,
        CommandId::NextTab | CommandId::PreviousTab => Disabled("Only one diagram tab is open."),
        CommandId::SelectAll if context.has_content => Enabled,
        CommandId::SelectAll => Disabled("This diagram has nothing to select."),
        CommandId::Rename if !context.support.draft_events => Disabled(
            "Not available yet: F2 moves focus to the Name field in the new inspector. \
             Use the Name field in the inspector.",
        ),
        CommandId::Rename if context.selection_count == 1 => Enabled,
        CommandId::Rename => Disabled("Select one class to rename it."),
        CommandId::Reveal | CommandId::FitSelection if context.has_selection => Enabled,
        CommandId::Reveal => Disabled("Select an object to reveal it."),
        CommandId::FitSelection => Disabled("Select something to fit it in the view."),
        CommandId::ZoomIn if context.can_zoom_in => Enabled,
        CommandId::ZoomIn => Disabled("The view is already at the largest zoom, 400%."),
        CommandId::ZoomOut if context.can_zoom_out => Enabled,
        CommandId::ZoomOut => Disabled("The view is already at the smallest zoom, 10%."),
        CommandId::ActualSize | CommandId::FitAll => Enabled,
        CommandId::ToggleGrid | CommandId::ToggleSnap if context.support.grid => Enabled,
        CommandId::ToggleGrid | CommandId::ToggleSnap => {
            Disabled("Not available yet: the grid is drawn by the new canvas view (A-5).")
        }
        CommandId::Preferences => Disabled(
            "Not available yet: the preferences window is part of the new screens (UI-09). \
             Change the theme in the status bar.",
        ),
        CommandId::FindObject if context.support.object_search => Enabled,
        CommandId::FindObject => Disabled(
            "Not available yet: object search is part of the new project panel (A-4). \
             Use Search to find classes by name.",
        ),
        CommandId::NextRegion | CommandId::PreviousRegion if context.support.region_focus => {
            Enabled
        }
        CommandId::NextRegion | CommandId::PreviousRegion => Disabled(
            "Not available yet: moving between regions arrives with the new workspace (A-10).",
        ),
        CommandId::ContextMenu | CommandId::HelpShortcuts => {
            Disabled("Not available yet: this surface is part of the new screens (UI-09).")
        }
        CommandId::SearchCommands
        | CommandId::SelectTool
        | CommandId::HandTool
        | CommandId::ToggleNavigator
        | CommandId::ToggleInspector
        | CommandId::ShowComponentShowcase => Enabled,
        CommandId::ClearSelection if context.has_selection => Enabled,
        CommandId::ClearSelection => Disabled("Nothing is selected."),
        CommandId::Cancel if context.can_cancel => Enabled,
        CommandId::Cancel => Disabled("Nothing to cancel."),
    }
}

/// The tool-row command for a notation tool.
pub const fn tool_command(tool: diagram_core::notation::ToolKind) -> CommandId {
    use diagram_core::notation::ToolKind;
    match tool {
        ToolKind::Select => CommandId::SelectTool,
        ToolKind::Hand => CommandId::HandTool,
        ToolKind::Class => CommandId::AddClass,
        ToolKind::Relationship => CommandId::Connect,
        ToolKind::Arrange => CommandId::Arrange,
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

/// Commands whose label, group or search alias contains `query`, ignoring
/// case. An empty query returns every command.
pub fn search(query: &str) -> Vec<&'static CommandSpec> {
    let query = query.trim().to_lowercase();
    CATALOG
        .iter()
        .filter(|spec| {
            query.is_empty()
                || spec.label.to_lowercase().contains(&query)
                || spec.group.label().to_lowercase().contains(&query)
                || spec.aliases.iter().any(|alias| alias.contains(&query))
        })
        .collect()
}

/// A shortcut as the user reads it, such as "Ctrl+Shift+Z" (or "Cmd" names on
/// macOS), matching egui's own formatting on Windows and Linux.
pub fn shortcut_text(shortcut: &KeyboardShortcut) -> String {
    shortcut.format(&ModifierNames::NAMES, cfg!(target_os = "macos"))
}

fn modifier_count(shortcut: &KeyboardShortcut) -> u8 {
    let modifiers = shortcut.modifiers;
    u8::from(modifiers.alt)
        + u8::from(modifiers.shift)
        + u8::from(modifiers.command || modifiers.ctrl || modifiers.mac_cmd)
}

/// Whether `spec`'s shortcut may fire in `context` (spec §8 input dispatch).
///
/// - A modal owns the keyboard; only Escape-style keys it handles itself pass.
/// - A focused text field or an IME composition keeps every key a text field
///   uses, and every canvas-only key.
/// - Canvas-only keys need the board to have focus.
pub fn shortcut_allowed(spec: &CommandSpec, context: &InputContext) -> bool {
    if context.modal_open {
        return false;
    }
    let typing = context.text_had_focus || context.ime_composing;
    if typing && spec.text_field_owns_shortcut {
        return false;
    }
    match spec.scope {
        ShortcutScope::Application => true,
        ShortcutScope::Canvas => !typing && context.focused_region == Region::Board,
    }
}

/// Consumes the first catalog shortcut pressed this frame that may fire in
/// `context`, and returns its command. Each key event is consumed at most
/// once, and more specific shortcuts are checked first, so Ctrl+Shift+Z is
/// never read as Ctrl+Z and Shift+G never as G.
pub fn take_shortcut_in(input: &mut InputState, context: &InputContext) -> Option<CommandId> {
    take_first(
        input,
        CATALOG
            .iter()
            .filter(|spec| shortcut_allowed(spec, context)),
    )
}

/// Consumes the most specific pressed shortcut among `specs`.
fn take_first<'a>(
    input: &mut InputState,
    specs: impl Iterator<Item = &'a CommandSpec>,
) -> Option<CommandId> {
    let mut candidates: Vec<(&CommandSpec, KeyboardShortcut)> = specs
        .flat_map(|spec| spec.all_shortcuts().map(move |shortcut| (spec, shortcut)))
        .collect();
    candidates.sort_by_key(|(_, shortcut)| Reverse(modifier_count(shortcut)));
    candidates
        .into_iter()
        .find(|(_, shortcut)| input.consume_shortcut(shortcut))
        .map(|(spec, _)| spec.id)
}

/// True when this frame's events show an input method composing text.
pub fn ime_composing(input: &InputState) -> bool {
    input.events.iter().any(
        |event| matches!(event, Event::Ime(ImeEvent::Preedit { text, .. }) if !text.is_empty()),
    )
}

/// Consumes the first catalog shortcut pressed this frame and returns its command.
///
/// This is [`take_shortcut_in`] for a shell that only knows whether a text
/// field had focus in the previous frame (egui drops text focus in the same
/// frame that Escape is pressed, so the current frame's focus is not
/// reliable). IME composition is read from this frame's events; the board is
/// assumed to have focus and no modal to be open.
///
/// Because such a shell cannot tell a text field that gains focus in this
/// very frame from the canvas, canvas keys that text fields also use
/// (Ctrl+A, Delete) are left alone here; [`take_shortcut_in`] handles them
/// once the region focus is known.
pub fn take_shortcut(input: &mut InputState, text_had_focus: bool) -> Option<CommandId> {
    let context = InputContext {
        text_had_focus,
        ime_composing: ime_composing(input),
        ..InputContext::default()
    };
    take_first(
        input,
        CATALOG
            .iter()
            .filter(|spec| shortcut_allowed(spec, &context))
            .filter(|spec| !(spec.scope == ShortcutScope::Canvas && spec.text_field_owns_shortcut)),
    )
}
