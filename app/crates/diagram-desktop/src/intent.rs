//! What the person did (`UiIntent`) and the per-frame facts only the UI layer
//! knows (`InputContext`).
//!
//! This is one of the three contract types between the views (Agent A) and
//! the functionality (Agent B); see
//! `plan-rust-diagram-editor/agents/AGENT-B-FUNCTIONALITY.md` section 1.
//! Views **push** intents and never mutate state. [`crate::dispatch::apply`]
//! is the only place that turns an intent into a state change. Changes to
//! these types are additive only: new fields and new variants.

use eframe::egui::{self, ThemePreference};

use crate::ui::commands::CommandId;

/// Something a view can select or act on.
///
/// The sample classes have no model identity yet, so a class is addressed by
/// its index in the sample list. Typed view IDs replace this when the
/// project model is bound to the editor (UI-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// A class box, by its index in the sample class list.
    Class(usize),
}

/// A text field whose value is edited as a draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DraftField {
    /// The selected class's name in the inspector.
    ClassName,
}

/// Which search list a query belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchScope {
    /// Command search (Ctrl+K / Ctrl+Shift+P).
    Commands,
    /// Object search in the project panel (Ctrl+F).
    Objects,
}

/// A workspace region that can hold keyboard focus (F6 order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Region {
    /// Header: menu, project name and file commands.
    Header,
    /// Project panel (navigator).
    Navigator,
    /// Diagram board: tabs, tools and canvas.
    #[default]
    Board,
    /// Inspector.
    Inspector,
    /// The open drawer (validation issues, or a panel drawer on narrow windows).
    Drawer,
    /// Status bar.
    Status,
}

impl Region {
    /// Every region, in F6 order.
    pub const ALL: [Self; 6] = [
        Self::Header,
        Self::Navigator,
        Self::Board,
        Self::Inspector,
        Self::Drawer,
        Self::Status,
    ];
}

/// A transient surface that can be closed without a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    /// The open project-panel or inspector drawer.
    Drawer,
    /// The component showcase window.
    Showcase,
    /// The command and object search dialog.
    CommandSearch,
    /// The object search in the project panel.
    ObjectSearch,
}

/// Kind of pointer event over the canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerKind {
    /// A button went down.
    Down,
    /// The pointer moved while a button was down.
    Move,
    /// The button was released.
    Up,
    /// The gesture was interrupted (pointer capture lost).
    Cancel,
    /// Ctrl+wheel zoom; `delta` is the zoom factor (1.0 = unchanged).
    Zoom {
        /// Multiplicative zoom factor proposed by the wheel.
        delta: f32,
    },
}

/// One pointer event over the canvas, in screen points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPointer {
    /// What happened.
    pub kind: PointerKind,
    /// Pointer position in screen points.
    pub position: egui::Pos2,
    /// Modifier keys held during the event.
    pub modifiers: egui::Modifiers,
    /// The object under the pointer according to the view's hit regions.
    pub hit: Option<Target>,
}

/// What the person did this frame. Views push these; [`crate::dispatch::apply`]
/// applies them.
#[derive(Debug, Clone, PartialEq)]
pub enum UiIntent {
    /// Run a catalog command (button, menu, shortcut or search result).
    Command(CommandId),
    /// Select an object; `toggle` (Shift) adds or removes it instead of replacing.
    Select {
        /// The object.
        target: Target,
        /// Toggle membership instead of replacing the selection.
        toggle: bool,
    },
    /// The text of a draft field changed.
    DraftChanged {
        /// The field.
        field: DraftField,
        /// Its complete new text.
        text: String,
    },
    /// Enter (or leaving the field) asks to commit the draft.
    CommitDraft(DraftField),
    /// Escape asks to discard the draft and restore the original value.
    CancelDraft(DraftField),
    /// A pointer event over the canvas.
    Pointer(CanvasPointer),
    /// The query of a search surface changed.
    Search {
        /// Which search.
        scope: SearchScope,
        /// The complete query text.
        query: String,
    },
    /// Keyboard focus moved into a region (F6 or a click).
    FocusRegion(Region),
    /// Close a transient surface (drawer, showcase or search).
    Close(Surface),
    /// Change the theme preference.
    SetTheme(ThemePreference),
    /// Open the built-in sample project from the welcome screen.
    OpenSampleProject,
    /// Create an empty project with this (already validated) name.
    CreateProject {
        /// Project name.
        name: String,
    },
    /// Make the tab at this index the active one (a click on the tab).
    ActivateTab(usize),
    /// Space went down (`true`) or up (`false`) over the canvas: drags pan
    /// while it is held. Never sent while a text field has focus.
    TemporaryPan(bool),
    /// The canvas has this size on screen, in points.
    CanvasResized {
        /// Width in points.
        width: f32,
        /// Height in points.
        height: f32,
    },
    /// The person's answer in the Resolve edit dialog.
    ResolveEdit(ResolveChoice),
}

/// Answers to the Resolve edit dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolveChoice {
    /// Go back to the field with the invalid text; nothing else happens.
    ReturnToField,
    /// Restore the original value, then do what was held back.
    DiscardEdit,
}

/// Facts only the UI layer knows this frame. Shortcut handling and focus
/// return read them; the views fill them in.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InputContext {
    /// A text field had keyboard focus at the end of the previous frame.
    /// egui drops text focus in the frame Escape is pressed, so the current
    /// frame's focus is not reliable for deciding who owns a key.
    pub text_had_focus: bool,
    /// An input method editor is composing text.
    pub ime_composing: bool,
    /// The region that has keyboard focus.
    pub focused_region: Region,
    /// A modal dialog is open.
    pub modal_open: bool,
    /// The widget that opened a dialog, drawer or window from this intent;
    /// focus returns to it when that surface closes.
    pub opener: Option<egui::Id>,
}
