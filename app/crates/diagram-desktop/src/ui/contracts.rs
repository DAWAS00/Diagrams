//! The contract of every catalog command (plan section 5): when it can run,
//! what it changes, who owns the change, its undo policy and its test.
//!
//! Moved out of `ui/commands.rs` unchanged; `commands` re-exports it.

use super::commands::CommandId;

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
            "escape_should_return_the_class_tool_to_select_and_keep_the_selection",
        ),
        CommandId::PanTool => c(
            "Canvas scrolling and zoom (UI-07)",
            "Makes Pan the active tool",
            Editor,
            NotRecorded,
            "visible_buttons_should_match_catalog_availability",
        ),
        CommandId::AddClass => c(
            "A diagram is open",
            "Makes Class the active tool; a click on the canvas then places and names a new class as one undo step",
            Editor,
            NoProjectChange,
            "clicking_the_canvas_should_place_and_name_a_class_in_one_step",
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
