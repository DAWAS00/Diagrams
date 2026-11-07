//! Typed user intentions (UI-06).
//!
//! Widgets never change editor or project state directly. They queue a
//! [`UiIntent`]; after the frame is drawn, the shell's coordinator applies the
//! queue in order. Text fields are the exception: they edit a draft in place,
//! committed drafts become `diagram-core` commands in the coordinator.
//! Targets are typed IDs, never list positions.

use diagram_core::ids::{DiagramId, ElementId, RelationId};
use eframe::egui::{Id, ThemePreference};

use super::class_tool::{CanvasPoint, CanvasSize, PlacementId};
use super::commands::CommandId;
use super::dialogs::DialogAnswer;
use super::inspector::RelationField;

/// Something the user asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    SelectClass(ElementId),
    /// Add a class to the selection, or remove it if already selected.
    ToggleClassInSelection(ElementId),
    /// Select a relationship.
    SelectRelation(RelationId),
    /// Commit a class name draft (validated by the coordinator).
    RenameClass(ElementId, String),
    /// Commit a relationship field draft (validated by the coordinator).
    SetRelationField(RelationId, RelationField, String),
    /// Select a class from the project panel, opening a diagram that shows it.
    RevealClass(ElementId),
    /// Show a diagram in its tab, opening a tab if needed.
    OpenDiagram(DiagramId),
    /// Make an open tab active.
    ActivateTab(DiagramId),
    /// Close an open tab (model elements are untouched).
    CloseTab(DiagramId),
    /// Change the theme preference.
    SetTheme(ThemePreference),
    /// Close the open drawer.
    CloseDrawer,
    /// Close the component showcase.
    CloseShowcase,
    /// Create a project with this name and open its empty diagram.
    CreateProject(String),
    /// Open the built-in sample project.
    OpenSample,
    /// Answer the open dialog.
    AnswerDialog(DialogAnswer),
    /// Start placing a new class at `at` on `diagram` (Class tool). `visible`
    /// is the canvas size, kept to fit the class later.
    BeginNewClass {
        /// The diagram shown on the canvas.
        diagram: DiagramId,
        /// Top-left of the new class box on the canvas.
        at: CanvasPoint,
        /// The visible canvas size.
        visible: CanvasSize,
    },
    /// Finish naming the placement with `name` (Enter or focus loss).
    CommitNewClass {
        /// The placement the name is for.
        placement: PlacementId,
        /// The typed name.
        name: String,
    },
    /// Cancel the placement; nothing is created.
    CancelNewClass(PlacementId),
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
