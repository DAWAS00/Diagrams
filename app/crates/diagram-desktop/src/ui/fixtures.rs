//! Design fixtures: one [`Presentation`] value per
//! screen state, used by the view tests, the usability audit and the
//! component showcase (task A-0).
//!
//! Nothing here opens, saves or changes a project. Every fixture is labeled
//! "Design fixture" wherever it could be mistaken for real data.
//!
//! # Contract copy
//!
//! The views draw a read-only `Presentation` and report what the person did
//! as `UiIntent`s. Agent B owns those types (`src/presentation.rs` and
//! `src/intent.rs`, task B-0). Until B-0 is merged, [`contract`] holds a
//! private copy of the planned types so the views can be built and tested.
//! After the merge the views switch to B's types and this copy is deleted.
//!
//! Labels, shortcut text and availability come from the real command
//! catalog ([`crate::ui::commands`]) whenever the catalog already has the
//! command. Planned commands that the catalog does not have yet carry no
//! shortcut text and, when disabled, the single [`FIXTURE_REASON`]; the UI
//! never writes its own reason or shortcut text.

use eframe::egui::{Pos2, Rect, ThemePreference, Vec2};

use super::commands::{self as catalog, CommandContext};
use super::theme;

pub use contract::*;

/// Reason given to disabled planned commands that the catalog does not have
/// yet. Real reasons always come from the command catalog.
pub const FIXTURE_REASON: &str = "Design fixture: the reason comes from the command catalog.";

/// Private copy of Agent B's planned contract types (see the module docs).
pub mod contract {
    use eframe::egui::{Color32, Modifiers, PointerButton, Pos2, Rect, ThemePreference, Vec2};

    pub use crate::ui::commands::Availability;

    /// Every command the planned catalog knows about (D1–D3 applied: Note is
    /// removed and Pan is called Hand).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[allow(missing_docs)]
    pub enum CommandId {
        Undo,
        Redo,
        SearchCommands,
        Save,
        Export,
        NewProject,
        OpenProject,
        SaveAs,
        CloseProject,
        SelectTool,
        HandTool,
        AddClass,
        Connect,
        KindAssociation,
        KindGeneralization,
        Arrange,
        AlignLeft,
        AlignCenter,
        AlignRight,
        AlignTop,
        AlignMiddle,
        AlignBottom,
        DistributeHorizontal,
        DistributeVertical,
        NewDiagram,
        RenameDiagram,
        CloseTab,
        NextTab,
        PreviousTab,
        DeleteDiagram,
        FindObject,
        Reveal,
        ShowInDiagram,
        DuplicateModel,
        RemoveView,
        DeleteModelEverywhere,
        AddAttribute,
        AddOperation,
        MemberUp,
        MemberDown,
        RemoveMember,
        EditMember,
        ToggleAttributes,
        ToggleOperations,
        ZoomIn,
        ZoomOut,
        ActualSize,
        FitAll,
        FitSelection,
        ToggleGrid,
        ToggleSnap,
        ToggleIssues,
        ToggleNavigator,
        ToggleInspector,
        ClearSelection,
        Preferences,
        HelpShortcuts,
        NextRegion,
        PreviousRegion,
        ContextMenu,
        ShowComponentShowcase,
    }

    impl CommandId {
        /// Every command, in catalog order.
        pub const ALL: [Self; 61] = [
            Self::Undo,
            Self::Redo,
            Self::SearchCommands,
            Self::Save,
            Self::Export,
            Self::NewProject,
            Self::OpenProject,
            Self::SaveAs,
            Self::CloseProject,
            Self::SelectTool,
            Self::HandTool,
            Self::AddClass,
            Self::Connect,
            Self::KindAssociation,
            Self::KindGeneralization,
            Self::Arrange,
            Self::AlignLeft,
            Self::AlignCenter,
            Self::AlignRight,
            Self::AlignTop,
            Self::AlignMiddle,
            Self::AlignBottom,
            Self::DistributeHorizontal,
            Self::DistributeVertical,
            Self::NewDiagram,
            Self::RenameDiagram,
            Self::CloseTab,
            Self::NextTab,
            Self::PreviousTab,
            Self::DeleteDiagram,
            Self::FindObject,
            Self::Reveal,
            Self::ShowInDiagram,
            Self::DuplicateModel,
            Self::RemoveView,
            Self::DeleteModelEverywhere,
            Self::AddAttribute,
            Self::AddOperation,
            Self::MemberUp,
            Self::MemberDown,
            Self::RemoveMember,
            Self::EditMember,
            Self::ToggleAttributes,
            Self::ToggleOperations,
            Self::ZoomIn,
            Self::ZoomOut,
            Self::ActualSize,
            Self::FitAll,
            Self::FitSelection,
            Self::ToggleGrid,
            Self::ToggleSnap,
            Self::ToggleIssues,
            Self::ToggleNavigator,
            Self::ToggleInspector,
            Self::ClearSelection,
            Self::Preferences,
            Self::HelpShortcuts,
            Self::NextRegion,
            Self::PreviousRegion,
            Self::ContextMenu,
            Self::ShowComponentShowcase,
        ];
    }

    /// One command as a control shows it this frame.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CommandView {
        /// The command a control emits.
        pub id: CommandId,
        /// Visible label, which is also the accessible name.
        pub label: String,
        /// Shortcut text such as "Ctrl+S", supplied by the catalog.
        pub shortcut_text: Option<String>,
        /// Enabled, or disabled with the catalog's reason.
        pub availability: Availability,
        /// Pressed or checked: the active tool, a shown panel, Grid on.
        pub active: bool,
    }

    impl CommandView {
        /// True when the command can run.
        pub fn enabled(&self) -> bool {
            self.availability.is_enabled()
        }
    }

    /// Opaque identity of a project object (typed IDs arrive with M1).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct ObjectKey(pub u64);

    /// Something a person can select, reveal or act on.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Target {
        /// A node view on the board.
        Node(ObjectKey),
        /// A relationship view on the board.
        Edge(ObjectKey),
        /// A model in the navigator's Model tab.
        Model(ObjectKey),
        /// A diagram in the navigator or a tab.
        Diagram(ObjectKey),
        /// A member row in the inspector.
        Member(ObjectKey),
        /// A validation issue.
        Issue(ObjectKey),
    }

    /// Fields whose text the UI edits as a draft.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[allow(missing_docs)]
    pub enum FieldId {
        DiagramName,
        ClassName,
        MemberName,
        MemberType,
        X,
        Y,
        Width,
        Height,
        SourceRole,
        SourceMultiplicity,
        TargetRole,
        TargetMultiplicity,
        NewDiagramName,
        /// A field of a multiple selection, by its position in the list.
        Common(u8),
    }

    /// Where a search applies.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum SearchScope {
        /// The command search ("Search commands…").
        Commands,
        /// The object search ("Find object…").
        Objects,
    }

    /// A focusable workspace region, cycled with F6.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[allow(missing_docs)]
    pub enum Region {
        Header,
        Navigator,
        Board,
        Inspector,
        ValidationDrawer,
        Status,
    }

    /// What a pointer did on the board.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum PointerKind {
        /// A button went down.
        Down(PointerButton),
        /// The pointer moved while a button was down.
        Move,
        /// The button came up.
        Up(PointerButton),
        /// Ctrl+wheel: zoom by `delta` steps around the pointer.
        Zoom {
            /// Positive zooms in.
            delta: f32,
        },
        /// Wheel without Ctrl: scroll the view.
        Scroll {
            /// Scroll distance in screen points.
            delta: Vec2,
        },
    }

    /// One pointer event on the board, in board-local screen points (the
    /// board's top-left corner is the origin).
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CanvasPointer {
        /// What happened.
        pub kind: PointerKind,
        /// Board-local screen position.
        pub pos: Pos2,
        /// Modifier keys held.
        pub modifiers: Modifiers,
    }

    /// A choice in a dialog.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[allow(missing_docs)]
    pub enum DialogChoice {
        Save,
        Discard,
        Cancel,
        DeleteEverywhere,
        ReturnToField,
        DiscardEdit,
        Retry,
        SaveAs,
        ChooseAnotherFile,
        Restore,
        Ignore,
        Create,
        /// One option of a choice list, by position.
        Option(u8),
    }

    /// The navigator's two tabs.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum NavigatorTab {
        /// Diagrams in the project.
        Diagrams,
        /// Models (classes) in the project.
        Model,
    }

    /// What the person did this frame. Views push these; they never change
    /// state themselves.
    #[derive(Debug, Clone, PartialEq)]
    pub enum UiIntent {
        /// Run a catalog command.
        Command(CommandId),
        /// Run a catalog command on one object, such as a row action.
        CommandOn {
            /// The command.
            command: CommandId,
            /// The object it applies to.
            target: Target,
        },
        /// Select an object; `toggle` adds or removes it (Shift).
        Select {
            /// The object.
            target: Target,
            /// Shift held: toggle membership instead of replacing.
            toggle: bool,
        },
        /// A field's draft text changed.
        DraftChanged {
            /// The field.
            field: FieldId,
            /// The complete new text.
            text: String,
        },
        /// Enter, or focus left the field: commit the draft if it is valid.
        CommitDraft(FieldId),
        /// Escape in the field: restore the original value.
        CancelDraft(FieldId),
        /// Pointer input on the board.
        Pointer(CanvasPointer),
        /// The text of a search field changed.
        Search {
            /// Which search.
            scope: SearchScope,
            /// The complete query.
            query: String,
        },
        /// Focus moved into a region (click or F6).
        FocusRegion(Region),
        /// A dialog button was chosen.
        Dialog(DialogChoice),
        /// The open dialog or search was closed (Escape or Close).
        CloseDialog,
        /// Close one diagram tab (never deletes the diagram).
        CloseTab(ObjectKey),
        /// Show one navigator tab.
        ShowNavigatorTab(NavigatorTab),
        /// Choose the application theme.
        SetTheme(ThemePreference),
        /// The board's size changed, in screen points.
        BoardResized(Vec2),
    }

    /// Facts only the UI layer knows each frame; B uses them to decide shortcuts.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct InputContext {
        /// A text field had keyboard focus at the end of the last frame.
        pub text_had_focus: bool,
        /// An input method is composing text.
        pub ime_composing: bool,
        /// The region that holds keyboard focus.
        pub focused_region: Option<Region>,
        /// A modal dialog or search is open.
        pub modal_open: bool,
    }

    /// How a status text should be read; always shown with words as well.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Tone {
        /// Plain information.
        Neutral,
        /// Work in progress, such as "Saving…".
        Progress,
        /// Finished successfully, such as "Saved".
        Success,
        /// Needs attention.
        Warning,
        /// Failed.
        Danger,
    }

    /// A short status in words, with a tone that adds color.
    #[derive(Debug, Clone, PartialEq)]
    pub struct StatusText {
        /// The words shown.
        pub text: String,
        /// How it is colored.
        pub tone: Tone,
    }

    /// A titled group of menu items.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MenuSection {
        /// Optional heading, such as a command group.
        pub title: Option<String>,
        /// Items in order.
        pub items: Vec<CommandView>,
        /// Destructive items, drawn apart from the others.
        pub destructive: bool,
    }

    /// A menu button and its contents.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MenuView {
        /// Button label.
        pub label: String,
        /// Sections in order.
        pub sections: Vec<MenuSection>,
    }

    /// The header row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeaderView {
        /// The Project menu, grouped by command group.
        pub project_menu: MenuView,
        /// Name of the open project.
        pub project_name: String,
        /// Save state in words: Unsaved, Saving…, Saved, Save failed.
        pub save_state: StatusText,
        /// Undo, with the pending action label ("Undo Rename class").
        pub undo: CommandView,
        /// Redo, with the pending action label.
        pub redo: CommandView,
        /// Search commands.
        pub search: CommandView,
        /// Save.
        pub save: CommandView,
        /// Export.
        pub export: CommandView,
    }

    /// One diagram tab.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TabView {
        /// The diagram shown by the tab.
        pub diagram: ObjectKey,
        /// Tab title.
        pub title: String,
        /// The active tab.
        pub active: bool,
    }

    /// The diagram tab row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TabsView {
        /// Open tabs in order.
        pub tabs: Vec<TabView>,
        /// "+ Diagram".
        pub add: CommandView,
    }

    /// One entry of the tool row.
    #[derive(Debug, Clone, PartialEq)]
    pub enum ToolItem {
        /// A tool button; `active` draws it pressed.
        Tool(CommandView),
        /// A chooser such as the relationship kind ("Association").
        Chooser(MenuView),
        /// A menu button such as Arrange.
        Menu(MenuView),
    }

    /// The tool row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ToolsView {
        /// Tools in order.
        pub items: Vec<ToolItem>,
        /// Project panel toggle at the row's start.
        pub navigator_toggle: CommandView,
        /// Inspector toggle at the row's end.
        pub inspector_toggle: CommandView,
    }

    /// A labeled search field.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SearchFieldView {
        /// Which search the field drives.
        pub scope: SearchScope,
        /// The query typed so far.
        pub query: String,
        /// Shortcut text that focuses the field, from the catalog.
        pub shortcut_text: Option<String>,
    }

    /// A list that can be loading, empty, without matches, failed or full.
    #[derive(Debug, Clone, PartialEq)]
    pub enum ListState<T> {
        /// Still being prepared.
        Loading,
        /// Nothing exists yet.
        Empty,
        /// Items exist, but none matches the query.
        NoMatch,
        /// The list could not be prepared; the text says why.
        Error(String),
        /// The rows.
        Rows(Vec<T>),
    }

    /// One navigator row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct NavRow {
        /// What the row selects.
        pub target: Target,
        /// Object name.
        pub name: String,
        /// Type, such as "Class" or "Class diagram".
        pub kind: String,
        /// Location, such as the diagrams showing it.
        pub location: String,
        /// The row is selected.
        pub selected: bool,
        /// The row's action menu.
        pub actions: Vec<MenuSection>,
    }

    /// The project panel.
    #[derive(Debug, Clone, PartialEq)]
    pub struct NavigatorView {
        /// The tab shown.
        pub tab: NavigatorTab,
        /// "Find object…" search.
        pub search: SearchFieldView,
        /// Diagrams tab rows.
        pub diagrams: ListState<NavRow>,
        /// Model tab rows.
        pub models: ListState<NavRow>,
        /// "+ Diagram".
        pub add_diagram: CommandView,
    }

    /// Diagram colors chosen by the document, never by the chrome theme.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DocumentStyle {
        /// Class box fill.
        pub fill: Color32,
        /// Lines and text.
        pub ink: Color32,
    }

    /// Connect-tool feedback on a node.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NodeHighlight {
        /// No feedback.
        None,
        /// The node the relationship starts from.
        ConnectSource,
        /// A node the relationship may end at.
        ValidTarget,
        /// A node the relationship may not end at.
        InvalidTarget,
    }

    /// One class box, in board-local screen points.
    #[derive(Debug, Clone, PartialEq)]
    pub struct NodeView {
        /// Identity for selection.
        pub target: Target,
        /// Class name.
        pub name: String,
        /// Attribute lines; `None` hides the compartment.
        pub attributes: Option<Vec<String>>,
        /// Operation lines; `None` hides the compartment.
        pub operations: Option<Vec<String>>,
        /// Screen rectangle relative to the board origin.
        pub rect: Rect,
        /// Selected.
        pub selected: bool,
        /// Connect feedback.
        pub highlight: NodeHighlight,
    }

    /// Relationship kind on the board.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EdgeKind {
        /// A plain line.
        Association,
        /// A hollow triangle at the parent end.
        Generalization,
    }

    /// One relationship, in board-local screen points.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EdgeView {
        /// Identity for selection.
        pub target: Target,
        /// Accessible description, such as "Customer to Order".
        pub name: String,
        /// Kind and marker.
        pub kind: EdgeKind,
        /// Route points from source to target.
        pub points: Vec<Pos2>,
        /// Label near the source end (role, multiplicity).
        pub source_label: Option<String>,
        /// Label near the target end.
        pub target_label: Option<String>,
        /// Selected.
        pub selected: bool,
    }

    /// The class being placed, before it exists.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlacementView {
        /// Preview rectangle.
        pub rect: Rect,
        /// Text such as "New class at 120, 80. Press Enter to name it."
        pub text: String,
    }

    /// A relationship being drawn.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConnectView {
        /// Start of the preview line.
        pub from: Pos2,
        /// End of the preview line.
        pub to: Pos2,
        /// Whether the current target is valid; `None` before one is reached.
        pub valid: Option<bool>,
        /// Feedback in words, such as "Invalid target: …".
        pub text: String,
    }

    /// What the empty diagram offers.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EmptyBoardView {
        /// "Add Class".
        pub add_class: CommandView,
        /// One-line hint.
        pub hint: String,
    }

    /// Everything the board draws.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CanvasView {
        /// Accessible name of the board.
        pub name: String,
        /// Document colors.
        pub style: DocumentStyle,
        /// Scale of text and boxes (1.0 is 100%).
        pub zoom: f32,
        /// Grid spacing in screen points; `None` hides the grid.
        pub grid: Option<f32>,
        /// Class boxes in drawing order.
        pub nodes: Vec<NodeView>,
        /// Relationships in drawing order.
        pub edges: Vec<EdgeView>,
        /// Marquee rectangle while dragging.
        pub marquee: Option<Rect>,
        /// Class placement preview.
        pub placement: Option<PlacementView>,
        /// Relationship preview.
        pub connect: Option<ConnectView>,
        /// Shown when the diagram has no objects.
        pub empty: Option<EmptyBoardView>,
    }

    /// Whether a field's text is committed, edited or invalid.
    #[derive(Debug, Clone, PartialEq)]
    pub enum FieldState {
        /// Shows the committed value.
        Committed,
        /// Shows an edit that is not committed yet.
        Unsaved,
        /// Shows an edit that cannot be committed; the text says why.
        Error(String),
    }

    /// One editable field.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FieldView {
        /// Identity for draft intents.
        pub field: FieldId,
        /// Visible label and accessible name.
        pub label: String,
        /// Draft or committed text.
        pub value: String,
        /// Committed, unsaved or error.
        pub state: FieldState,
        /// Disabled fields say why.
        pub availability: Availability,
    }

    /// Which part of the project an inspector section changes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Scope {
        /// "Shared model — affects all views".
        SharedModel,
        /// "This diagram view only".
        ThisView,
        /// Editor preferences, outside the document.
        Preference,
    }

    /// One attribute or operation row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MemberRowView {
        /// Identity.
        pub member: ObjectKey,
        /// "attribute" or "operation", used in accessible names.
        pub kind: String,
        /// Member name, used in accessible names.
        pub name: String,
        /// Full member text, such as "+ id: String".
        pub text: String,
        /// Move up.
        pub up: CommandView,
        /// Move down.
        pub down: CommandView,
        /// Remove.
        pub remove: CommandView,
        /// Open the row editor.
        pub edit: CommandView,
    }

    /// The expanded member editor.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MemberEditorView {
        /// The member being edited.
        pub member: ObjectKey,
        /// Name and type fields.
        pub fields: Vec<FieldView>,
    }

    /// An inspector section with its scope.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SectionView {
        /// Title, such as "Model".
        pub title: String,
        /// What the section changes.
        pub scope: Scope,
        /// Fields in order.
        pub fields: Vec<FieldView>,
        /// Toggles and actions in order.
        pub actions: Vec<CommandView>,
    }

    /// Inspector for one class.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ClassInspectorView {
        /// Model section (name).
        pub model: SectionView,
        /// Attribute rows.
        pub attributes: Vec<MemberRowView>,
        /// Operation rows.
        pub operations: Vec<MemberRowView>,
        /// Add attribute, Add operation.
        pub member_actions: Vec<CommandView>,
        /// The open member editor.
        pub editor: Option<MemberEditorView>,
        /// Appearance section (geometry, compartments).
        pub appearance: SectionView,
    }

    /// The inspector for the current selection.
    #[derive(Debug, Clone, PartialEq)]
    pub enum InspectorView {
        /// Nothing selected: diagram settings.
        Diagram {
            /// Diagram name and preferences.
            settings: SectionView,
            /// Short creation guidance.
            guidance: String,
        },
        /// One class.
        Class(Box<ClassInspectorView>),
        /// One relationship.
        Relationship {
            /// Kind summary, such as "Association from Customer to Order".
            summary: String,
            /// Source end.
            source: SectionView,
            /// Target end.
            target: SectionView,
            /// Route and attachment.
            appearance: SectionView,
        },
        /// Several node views.
        Multiple {
            /// "3 classes selected".
            count: String,
            /// Common fields; differing values read "Mixed".
            common: SectionView,
            /// Align and Distribute.
            arrange: Vec<CommandView>,
        },
        /// Nodes and relationships together.
        Mixed {
            /// "2 classes and 1 relationship selected".
            count: String,
            /// Actions that apply to everything selected.
            actions: Vec<CommandView>,
            /// Why other fields are absent.
            note: String,
        },
    }

    /// The status bar.
    #[derive(Debug, Clone, PartialEq)]
    pub struct StatusView {
        /// Save state in words.
        pub save_state: StatusText,
        /// File path, when the project has one.
        pub path: Option<String>,
        /// Tool hint or selection count.
        pub hint: String,
        /// Issues button ("2 issues"), opens the validation drawer.
        pub issues: CommandView,
        /// Grid toggle.
        pub grid: CommandView,
        /// Snap toggle.
        pub snap: CommandView,
        /// Zoom out.
        pub zoom_out: CommandView,
        /// Zoom percent, such as "100%".
        pub zoom_text: String,
        /// Zoom in.
        pub zoom_in: CommandView,
        /// 100%.
        pub actual_size: CommandView,
        /// Fit.
        pub fit: CommandView,
        /// The theme in use.
        pub theme: ThemePreference,
    }

    /// Kind of dialog, for identity and tests.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[allow(missing_docs)]
    pub enum DialogKind {
        UnsavedChanges,
        DeleteEverywhere,
        ResolveEdit,
        SaveFailed,
        OpenFailed,
        Recovery,
        NewDiagram,
    }

    /// A dialog button.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DialogButton {
        /// What it chooses.
        pub choice: DialogChoice,
        /// Label.
        pub label: String,
        /// Disabled buttons say why.
        pub availability: Availability,
        /// Gets keyboard focus when the dialog opens.
        pub default_focus: bool,
        /// Drawn as destructive.
        pub destructive: bool,
    }

    /// One counted consequence, such as "Views removed: 3".
    #[derive(Debug, Clone, PartialEq)]
    pub struct ImpactRow {
        /// What is affected.
        pub label: String,
        /// How many.
        pub count: usize,
    }

    /// One option of a dialog's choice list (such as a diagram type).
    #[derive(Debug, Clone, PartialEq)]
    pub struct DialogOption {
        /// Label.
        pub label: String,
        /// Chosen.
        pub selected: bool,
        /// Disabled options say why.
        pub availability: Availability,
    }

    /// A modal dialog.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DialogView {
        /// Which dialog.
        pub kind: DialogKind,
        /// Title.
        pub title: String,
        /// Explanation paragraphs.
        pub message: Vec<String>,
        /// An error in words, shown with "Error:".
        pub error: Option<String>,
        /// Counted consequences.
        pub impact: Vec<ImpactRow>,
        /// Choice list title and options.
        pub options: Option<(String, Vec<DialogOption>)>,
        /// Fields.
        pub fields: Vec<FieldView>,
        /// Buttons in order.
        pub buttons: Vec<DialogButton>,
    }

    /// Severity in words.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Severity {
        /// Blocks saving or export.
        Error,
        /// Does not block.
        Warning,
    }

    /// One validation issue.
    #[derive(Debug, Clone, PartialEq)]
    pub struct IssueRow {
        /// Identity.
        pub issue: Target,
        /// Severity.
        pub severity: Severity,
        /// What is wrong.
        pub message: String,
        /// Where, such as "Class Customer in Class overview".
        pub location: String,
        /// The object Go to object reveals.
        pub go_to: Target,
    }

    /// The validation drawer under the board.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ValidationView {
        /// "Current", "Checking…" or "Stale".
        pub freshness: StatusText,
        /// Issues.
        pub issues: ListState<IssueRow>,
        /// Close the drawer.
        pub close: CommandView,
    }

    /// One command search result.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CommandResult {
        /// The command.
        pub command: CommandView,
        /// Its group, such as "File".
        pub group: String,
    }

    /// The command search ("Search commands…").
    #[derive(Debug, Clone, PartialEq)]
    pub struct CommandSearchView {
        /// Shown this frame.
        pub open: bool,
        /// The query.
        pub query: String,
        /// Results.
        pub results: ListState<CommandResult>,
    }

    /// One object search result.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ObjectResult {
        /// What the result selects.
        pub target: Target,
        /// Name.
        pub name: String,
        /// Type.
        pub kind: String,
        /// Location.
        pub location: String,
    }

    /// The object search ("Find object…").
    #[derive(Debug, Clone, PartialEq)]
    pub struct ObjectSearchView {
        /// The field.
        pub field: SearchFieldView,
        /// Results; `None` while the query is empty.
        pub results: Option<ListState<ObjectResult>>,
    }

    /// Docked panels or drawers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LayoutView {
        /// Docked or narrow.
        pub mode: crate::ui::workspace::LayoutMode,
        /// The docked project panel is shown.
        pub navigator_shown: bool,
        /// The docked inspector is shown.
        pub inspector_shown: bool,
        /// The open drawer on narrow windows.
        pub drawer: Option<crate::ui::workspace::Drawer>,
        /// The region drawn with a focus outline.
        pub focused_region: Option<Region>,
    }

    /// Everything the workspace shows this frame.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Presentation {
        /// Panels and drawers.
        pub layout: LayoutView,
        /// Header row.
        pub header: HeaderView,
        /// Diagram tabs.
        pub tabs: TabsView,
        /// Tool row.
        pub tools: ToolsView,
        /// Project panel.
        pub navigator: NavigatorView,
        /// Board.
        pub canvas: CanvasView,
        /// Inspector.
        pub inspector: InspectorView,
        /// Status bar.
        pub status: StatusView,
        /// Validation drawer, when open.
        pub validation: Option<ValidationView>,
        /// Open dialog.
        pub dialog: Option<DialogView>,
        /// Command search.
        pub command_search: CommandSearchView,
    }
}

/// The real catalog entry for a planned command, when the catalog has it.
fn catalog_id(id: CommandId) -> Option<catalog::CommandId> {
    use catalog::CommandId as C;
    Some(match id {
        CommandId::Undo => C::Undo,
        CommandId::Redo => C::Redo,
        CommandId::SearchCommands => C::SearchCommands,
        CommandId::Save => C::Save,
        CommandId::Export => C::Export,
        CommandId::SelectTool => C::SelectTool,
        CommandId::HandTool => C::PanTool,
        CommandId::AddClass => C::AddClass,
        CommandId::Connect => C::Connect,
        CommandId::Arrange => C::Arrange,
        CommandId::NewDiagram => C::NewDiagram,
        CommandId::ToggleNavigator => C::ToggleNavigator,
        CommandId::ToggleInspector => C::ToggleInspector,
        CommandId::ClearSelection => C::ClearSelection,
        CommandId::ShowComponentShowcase => C::ShowComponentShowcase,
        _ => return None,
    })
}

/// Label of a planned command. Commands the catalog has use the catalog's
/// label, except where a confirmed decision renamed them (D3: Hand).
fn planned_label(id: CommandId) -> &'static str {
    use CommandId as I;
    match id {
        I::HandTool => "Hand",
        I::Connect => "Relationship",
        I::SearchCommands => "Search commands",
        I::NewDiagram => "+ Diagram",
        I::NewProject => "New project",
        I::OpenProject => "Open project",
        I::SaveAs => "Save As",
        I::CloseProject => "Close project",
        I::KindAssociation => "Association",
        I::KindGeneralization => "Generalization",
        I::AlignLeft => "Align left",
        I::AlignCenter => "Align center",
        I::AlignRight => "Align right",
        I::AlignTop => "Align top",
        I::AlignMiddle => "Align middle",
        I::AlignBottom => "Align bottom",
        I::DistributeHorizontal => "Distribute horizontally",
        I::DistributeVertical => "Distribute vertically",
        I::RenameDiagram => "Rename diagram",
        I::CloseTab => "Close tab",
        I::NextTab => "Next tab",
        I::PreviousTab => "Previous tab",
        I::DeleteDiagram => "Delete diagram",
        I::FindObject => "Find object",
        I::Reveal => "Reveal",
        I::ShowInDiagram => "Show in diagram",
        I::DuplicateModel => "Duplicate model",
        I::RemoveView => "Remove view",
        I::DeleteModelEverywhere => "Delete model everywhere",
        I::AddAttribute => "Add attribute",
        I::AddOperation => "Add operation",
        I::MemberUp => "Up",
        I::MemberDown => "Down",
        I::RemoveMember => "Remove",
        I::EditMember => "Edit",
        I::ToggleAttributes => "Attributes",
        I::ToggleOperations => "Operations",
        I::ZoomIn => "Zoom in",
        I::ZoomOut => "Zoom out",
        I::ActualSize => "100%",
        I::FitAll => "Fit",
        I::FitSelection => "Fit selection",
        I::ToggleGrid => "Grid",
        I::ToggleSnap => "Snap",
        I::ToggleIssues => "Issues",
        I::Preferences => "Preferences",
        I::HelpShortcuts => "Help and shortcuts",
        I::NextRegion => "Next region",
        I::PreviousRegion => "Previous region",
        I::ContextMenu => "Context menu",
        other => match catalog_id(other) {
            Some(real) => real.spec().label,
            None => "Command",
        },
    }
}

/// A command as the catalog presents it in the sample state: catalog
/// shortcut and availability when the catalog has the command, otherwise
/// enabled with no shortcut.
pub fn command(id: CommandId) -> CommandView {
    let real = catalog_id(id);
    let shortcut_text = real
        .and_then(|real| real.spec().shortcut)
        .map(|shortcut| shortcut.format(&eframe::egui::ModifierNames::NAMES, false));
    let availability = real.map_or(Availability::Enabled, |real| {
        catalog::availability(real, CommandContext::default())
    });
    CommandView {
        id,
        label: planned_label(id).to_owned(),
        shortcut_text,
        availability,
        active: false,
    }
}

/// [`command`], pressed or checked.
pub fn active(id: CommandId) -> CommandView {
    CommandView {
        active: true,
        ..command(id)
    }
}

/// [`command`], disabled with the catalog's reason, or with
/// [`FIXTURE_REASON`] for a planned command.
pub fn disabled(id: CommandId) -> CommandView {
    let mut view = command(id);
    if view.availability.is_enabled() {
        view.availability = Availability::Disabled(FIXTURE_REASON);
    }
    view
}

/// [`command`], enabled even when the catalog disables it today (for
/// fixtures of states the model will make possible).
pub fn enabled(id: CommandId) -> CommandView {
    CommandView {
        availability: Availability::Enabled,
        ..command(id)
    }
}

/// [`command`] with a different label, such as "Undo Rename class".
pub fn labeled(id: CommandId, label: &str) -> CommandView {
    CommandView {
        label: label.to_owned(),
        ..command(id)
    }
}

fn text(text: &str, tone: Tone) -> StatusText {
    StatusText {
        text: text.to_owned(),
        tone,
    }
}

/// Header as the sample project shows it.
pub fn header() -> HeaderView {
    let sections = catalog::CommandGroup::ALL
        .into_iter()
        .map(|group| MenuSection {
            title: Some(group.label().to_owned()),
            items: CommandId::ALL
                .into_iter()
                .filter(|id| catalog_id(*id).is_some_and(|real| real.spec().group == group))
                .map(command)
                .collect(),
            destructive: false,
        })
        .filter(|section| !section.items.is_empty())
        .collect();
    HeaderView {
        project_menu: MenuView {
            label: String::from("Project"),
            sections,
        },
        project_name: String::from("Sample project"),
        save_state: text("Unsaved", Tone::Warning),
        undo: command(CommandId::Undo),
        redo: command(CommandId::Redo),
        search: command(CommandId::SearchCommands),
        save: command(CommandId::Save),
        export: command(CommandId::Export),
    }
}

/// Header with history labels, as after renaming a class and undoing once.
pub fn header_with_history() -> HeaderView {
    HeaderView {
        undo: CommandView {
            availability: Availability::Enabled,
            ..labeled(CommandId::Undo, "Undo Rename class")
        },
        redo: CommandView {
            availability: Availability::Enabled,
            ..labeled(CommandId::Redo, "Redo Add attribute")
        },
        save_state: text("Saved", Tone::Success),
        ..header()
    }
}

/// Header with each save state in words.
pub fn header_with_save_state(state: &str, tone: Tone) -> HeaderView {
    HeaderView {
        save_state: text(state, tone),
        ..header()
    }
}

fn tab(key: u64, title: &str, active: bool) -> TabView {
    TabView {
        diagram: ObjectKey(key),
        title: title.to_owned(),
        active,
    }
}

/// Two tabs, the first active.
pub fn tabs() -> TabsView {
    TabsView {
        tabs: vec![
            tab(1, "Class overview", true),
            tab(2, "Domain details", false),
        ],
        add: command(CommandId::NewDiagram),
    }
}

/// More tabs than fit at 800 wide.
pub fn many_tabs() -> TabsView {
    let titles = [
        "Class overview",
        "Domain details",
        "Billing and invoices",
        "Customer accounts",
        "Shipping",
        "Reporting model",
        "Archive",
    ];
    TabsView {
        tabs: titles
            .iter()
            .enumerate()
            .map(|(index, title)| tab(index as u64 + 1, title, index == 2))
            .collect(),
        add: command(CommandId::NewDiagram),
    }
}

fn arrange_menu() -> MenuView {
    use CommandId as I;
    MenuView {
        label: planned_label(I::Arrange).to_owned(),
        sections: vec![
            MenuSection {
                title: Some(String::from("Align")),
                items: [
                    I::AlignLeft,
                    I::AlignCenter,
                    I::AlignRight,
                    I::AlignTop,
                    I::AlignMiddle,
                    I::AlignBottom,
                ]
                .into_iter()
                .map(disabled)
                .collect(),
                destructive: false,
            },
            MenuSection {
                title: Some(String::from("Distribute")),
                items: [I::DistributeHorizontal, I::DistributeVertical]
                    .into_iter()
                    .map(disabled)
                    .collect(),
                destructive: false,
            },
        ],
    }
}

fn kind_chooser(generalization: bool) -> MenuView {
    let (chosen, other) = if generalization {
        (CommandId::KindGeneralization, CommandId::KindAssociation)
    } else {
        (CommandId::KindAssociation, CommandId::KindGeneralization)
    };
    let mut items = vec![active(chosen), command(other)];
    items.sort_by_key(|item| item.id != CommandId::KindAssociation);
    MenuView {
        label: planned_label(chosen).to_owned(),
        sections: vec![MenuSection {
            title: Some(String::from("Relationship kind")),
            items,
            destructive: false,
        }],
    }
}

/// Tool row with `tool` pressed.
pub fn tools_with(tool: CommandId, generalization: bool) -> ToolsView {
    use CommandId as I;
    let tool_button = |id| {
        if id == tool {
            enabled(id).with_active()
        } else {
            command(id)
        }
    };
    ToolsView {
        items: vec![
            ToolItem::Tool(tool_button(I::SelectTool)),
            ToolItem::Tool(tool_button(I::HandTool)),
            ToolItem::Tool(tool_button(I::AddClass)),
            ToolItem::Tool(tool_button(I::Connect)),
            ToolItem::Chooser(kind_chooser(generalization)),
            ToolItem::Menu(arrange_menu()),
        ],
        navigator_toggle: active(I::ToggleNavigator),
        inspector_toggle: active(I::ToggleInspector),
    }
}

/// Tool row with Select pressed.
pub fn tools() -> ToolsView {
    tools_with(CommandId::SelectTool, false)
}

impl CommandView {
    /// This command, pressed.
    pub fn with_active(self) -> Self {
        Self {
            active: true,
            ..self
        }
    }
}

fn row_actions(model: bool) -> Vec<MenuSection> {
    use CommandId as I;
    let (safe, destructive): (Vec<_>, Vec<_>) = if model {
        (
            vec![
                command(I::Reveal),
                disabled(I::ShowInDiagram),
                disabled(I::DuplicateModel),
            ],
            vec![disabled(I::DeleteModelEverywhere)],
        )
    } else {
        (
            vec![command(I::Reveal), disabled(I::RenameDiagram)],
            vec![disabled(I::DeleteDiagram)],
        )
    };
    vec![
        MenuSection {
            title: None,
            items: safe,
            destructive: false,
        },
        MenuSection {
            title: None,
            items: destructive,
            destructive: true,
        },
    ]
}

/// A model row.
pub fn model_row(key: u64, name: &str, location: &str, selected: bool) -> NavRow {
    NavRow {
        target: Target::Model(ObjectKey(key)),
        name: name.to_owned(),
        kind: String::from("Class"),
        location: location.to_owned(),
        selected,
        actions: row_actions(true),
    }
}

fn diagram_row(key: u64, name: &str, selected: bool) -> NavRow {
    NavRow {
        target: Target::Diagram(ObjectKey(key)),
        name: name.to_owned(),
        kind: String::from("Class diagram"),
        location: String::from("Sample project"),
        selected,
        actions: row_actions(false),
    }
}

fn search_field(scope: SearchScope, query: &str) -> SearchFieldView {
    SearchFieldView {
        scope,
        query: query.to_owned(),
        shortcut_text: None,
    }
}

/// Navigator on the Model tab with the sample classes; `selected` marks a row.
pub fn navigator(tab: NavigatorTab, selected: Option<u64>) -> NavigatorView {
    NavigatorView {
        tab,
        search: search_field(SearchScope::Objects, ""),
        diagrams: ListState::Rows(vec![
            diagram_row(1, "Class overview", true),
            diagram_row(2, "Domain details", false),
        ]),
        models: ListState::Rows(vec![
            model_row(
                10,
                "Customer",
                "Class overview, Domain details",
                selected == Some(10),
            ),
            model_row(11, "Order", "Class overview", selected == Some(11)),
            // Two models may share a name; type and location tell them apart.
            model_row(12, "Order", "Domain details", selected == Some(12)),
        ]),
        add_diagram: command(CommandId::NewDiagram),
    }
}

/// Navigator whose Model list is in `state`, with `query` typed.
pub fn navigator_models(state: ListState<NavRow>, query: &str) -> NavigatorView {
    NavigatorView {
        models: state,
        search: search_field(SearchScope::Objects, query),
        ..navigator(NavigatorTab::Model, None)
    }
}

/// Default document colors from [`theme`], never the chrome palette.
pub fn document_style() -> DocumentStyle {
    DocumentStyle {
        fill: theme::DOCUMENT_DEFAULT_FILL,
        ink: theme::DOCUMENT_DEFAULT_INK,
    }
}

fn node(key: u64, name: &str, attributes: &[&str], operations: &[&str], rect: Rect) -> NodeView {
    let lines = |lines: &[&str]| Some(lines.iter().map(|line| (*line).to_owned()).collect());
    NodeView {
        target: Target::Node(ObjectKey(key)),
        name: name.to_owned(),
        attributes: lines(attributes),
        operations: lines(operations),
        rect,
        selected: false,
        highlight: NodeHighlight::None,
    }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(width, height))
}

/// The sample board: Customer and Order joined by an association, and a
/// generalization from PriorityOrder to Order.
pub fn canvas() -> CanvasView {
    CanvasView {
        name: String::from("Diagram board: Class overview"),
        style: document_style(),
        zoom: 1.0,
        grid: Some(16.0),
        nodes: vec![
            node(
                20,
                "Customer",
                &["+ id: CustomerId", "+ name: String"],
                &["+ rename(name: String)"],
                rect(48.0, 48.0, 200.0, 112.0),
            ),
            node(
                21,
                "Order",
                &["+ id: OrderId", "+ total: Money"],
                &[],
                rect(340.0, 64.0, 180.0, 88.0),
            ),
            node(
                22,
                "PriorityOrder",
                &["+ level: Integer"],
                &[],
                rect(340.0, 232.0, 180.0, 68.0),
            ),
        ],
        edges: vec![
            EdgeView {
                target: Target::Edge(ObjectKey(30)),
                name: String::from("Association from Customer to Order"),
                kind: EdgeKind::Association,
                points: vec![Pos2::new(248.0, 104.0), Pos2::new(340.0, 104.0)],
                source_label: Some(String::from("1")),
                target_label: Some(String::from("0..*")),
                selected: false,
            },
            EdgeView {
                target: Target::Edge(ObjectKey(31)),
                name: String::from("Generalization from PriorityOrder to Order"),
                kind: EdgeKind::Generalization,
                points: vec![Pos2::new(430.0, 232.0), Pos2::new(430.0, 152.0)],
                source_label: None,
                target_label: None,
                selected: false,
            },
        ],
        marquee: None,
        placement: None,
        connect: None,
        empty: None,
    }
}

/// Sample board with `keys` selected (nodes or edges).
pub fn canvas_selected(keys: &[u64]) -> CanvasView {
    let mut view = canvas();
    for node in &mut view.nodes {
        if let Target::Node(ObjectKey(key)) = node.target {
            node.selected = keys.contains(&key);
        }
    }
    for edge in &mut view.edges {
        if let Target::Edge(ObjectKey(key)) = edge.target {
            edge.selected = keys.contains(&key);
        }
    }
    view
}

/// An empty diagram with the Add Class prompt.
pub fn canvas_empty() -> CanvasView {
    CanvasView {
        nodes: Vec::new(),
        edges: Vec::new(),
        empty: Some(EmptyBoardView {
            add_class: labeled(CommandId::AddClass, "Add Class"),
            hint: String::from("Or choose Class in the tool row, then click the board."),
        }),
        ..canvas()
    }
}

/// Marquee selection in progress.
pub fn canvas_marquee() -> CanvasView {
    CanvasView {
        marquee: Some(rect(30.0, 30.0, 260.0, 160.0)),
        ..canvas_selected(&[20])
    }
}

/// Class placement preview.
pub fn canvas_placement() -> CanvasView {
    CanvasView {
        placement: Some(PlacementView {
            rect: rect(96.0, 260.0, 180.0, 68.0),
            text: String::from("New class at 96, 260. Press Enter to name it, Escape to cancel."),
        }),
        ..canvas()
    }
}

/// Relationship preview from Customer; `valid` decides the target feedback.
pub fn canvas_connect(valid: bool) -> CanvasView {
    let mut view = canvas();
    view.nodes[0].highlight = NodeHighlight::ConnectSource;
    let (target, to, text) = if valid {
        (
            1,
            Pos2::new(380.0, 108.0),
            "Valid target: Order. Release to connect.",
        )
    } else {
        (
            0,
            Pos2::new(150.0, 120.0),
            "Invalid target: self-association is not supported yet.",
        )
    };
    if valid {
        view.nodes[target].highlight = NodeHighlight::ValidTarget;
    } else {
        view.nodes[target].highlight = NodeHighlight::InvalidTarget;
    }
    view.connect = Some(ConnectView {
        from: Pos2::new(248.0, 120.0),
        to,
        valid: Some(valid),
        text: text.to_owned(),
    });
    view
}

/// A field showing a committed value.
pub fn field(field: FieldId, label: &str, value: &str) -> FieldView {
    FieldView {
        field,
        label: label.to_owned(),
        value: value.to_owned(),
        state: FieldState::Committed,
        availability: Availability::Enabled,
    }
}

fn section(
    title: &str,
    scope: Scope,
    fields: Vec<FieldView>,
    actions: Vec<CommandView>,
) -> SectionView {
    SectionView {
        title: title.to_owned(),
        scope,
        fields,
        actions,
    }
}

/// Inspector with nothing selected: diagram settings.
pub fn inspector_none() -> InspectorView {
    InspectorView::Diagram {
        settings: section(
            "Diagram",
            Scope::SharedModel,
            vec![field(
                FieldId::DiagramName,
                "Diagram name",
                "Class overview",
            )],
            vec![
                active(CommandId::ToggleGrid),
                command(CommandId::ToggleSnap),
            ],
        ),
        guidance: String::from(
            "Select a class or relationship to edit it, or choose Class to add one.",
        ),
    }
}

fn member(key: u64, kind: &str, name: &str, text: &str, first: bool, last: bool) -> MemberRowView {
    let gate = |id, allowed: bool| if allowed { enabled(id) } else { disabled(id) };
    MemberRowView {
        member: ObjectKey(key),
        kind: kind.to_owned(),
        name: name.to_owned(),
        text: text.to_owned(),
        up: gate(CommandId::MemberUp, !first),
        down: gate(CommandId::MemberDown, !last),
        remove: enabled(CommandId::RemoveMember),
        edit: enabled(CommandId::EditMember),
    }
}

/// Inspector for Customer; `name` is the draft and `state` its state.
pub fn inspector_class(name: &str, state: FieldState) -> InspectorView {
    let mut name_field = field(FieldId::ClassName, "Name", name);
    name_field.state = state;
    InspectorView::Class(Box::new(ClassInspectorView {
        model: section("Model", Scope::SharedModel, vec![name_field], Vec::new()),
        attributes: vec![
            member(40, "attribute", "id", "+ id: CustomerId", true, false),
            member(41, "attribute", "name", "+ name: String", false, true),
        ],
        operations: vec![member(
            42,
            "operation",
            "rename",
            "+ rename(name: String)",
            true,
            true,
        )],
        member_actions: vec![
            disabled(CommandId::AddAttribute),
            disabled(CommandId::AddOperation),
        ],
        editor: None,
        appearance: section(
            "Appearance",
            Scope::ThisView,
            vec![
                field(FieldId::X, "X", "48"),
                field(FieldId::Y, "Y", "48"),
                field(FieldId::Width, "Width", "200"),
                field(FieldId::Height, "Height", "112"),
            ],
            vec![
                active(CommandId::ToggleAttributes),
                active(CommandId::ToggleOperations),
            ],
        ),
    }))
}

/// Inspector for Customer with the member editor open on attribute `id`.
pub fn inspector_member_editor() -> InspectorView {
    let mut view = inspector_class("Customer", FieldState::Committed);
    if let InspectorView::Class(class) = &mut view {
        class.editor = Some(MemberEditorView {
            member: ObjectKey(40),
            fields: vec![
                field(FieldId::MemberName, "Attribute name", "id"),
                FieldView {
                    state: FieldState::Unsaved,
                    ..field(FieldId::MemberType, "Attribute type", "CustomerKey")
                },
            ],
        });
    }
    view
}

/// Inspector for the Customer–Order association.
pub fn inspector_relationship() -> InspectorView {
    InspectorView::Relationship {
        summary: String::from("Association from Customer to Order"),
        source: section(
            "Source end: Customer",
            Scope::SharedModel,
            vec![
                field(FieldId::SourceRole, "Source role", ""),
                field(FieldId::SourceMultiplicity, "Source multiplicity", "1"),
            ],
            Vec::new(),
        ),
        target: section(
            "Target end: Order",
            Scope::SharedModel,
            vec![
                field(FieldId::TargetRole, "Target role", "orders"),
                FieldView {
                    state: FieldState::Error(String::from(
                        "Use a number, *, or lower..upper such as 0..*.",
                    )),
                    ..field(FieldId::TargetMultiplicity, "Target multiplicity", "2..1")
                },
            ],
            Vec::new(),
        ),
        appearance: section("Route", Scope::ThisView, Vec::new(), Vec::new()),
    }
}

/// Inspector for three selected classes.
pub fn inspector_multiple() -> InspectorView {
    use CommandId as I;
    InspectorView::Multiple {
        count: String::from("3 classes selected"),
        common: section(
            "Common appearance",
            Scope::ThisView,
            vec![
                field(FieldId::Common(0), "Width", "Mixed"),
                field(FieldId::Common(1), "Height", "88"),
            ],
            Vec::new(),
        ),
        arrange: [
            I::AlignLeft,
            I::AlignCenter,
            I::AlignRight,
            I::AlignTop,
            I::AlignMiddle,
            I::AlignBottom,
            I::DistributeHorizontal,
            I::DistributeVertical,
        ]
        .into_iter()
        .map(disabled)
        .collect(),
    }
}

/// Inspector for classes and a relationship together.
pub fn inspector_mixed() -> InspectorView {
    InspectorView::Mixed {
        count: String::from("2 classes and 1 relationship selected"),
        actions: vec![disabled(CommandId::RemoveView)],
        note: String::from(
            "Member and size fields apply to classes only. Select only classes to edit them.",
        ),
    }
}

/// Status bar of the sample project.
pub fn status() -> StatusView {
    use CommandId as I;
    StatusView {
        save_state: text("Unsaved", Tone::Warning),
        path: None,
        hint: String::from("Select: click an object, or drag to select several."),
        issues: labeled(I::ToggleIssues, "2 issues"),
        grid: active(I::ToggleGrid),
        snap: command(I::ToggleSnap),
        zoom_out: command(I::ZoomOut),
        zoom_text: String::from("100%"),
        zoom_in: command(I::ZoomIn),
        actual_size: command(I::ActualSize),
        fit: command(I::FitAll),
        theme: ThemePreference::Light,
    }
}

fn button(choice: DialogChoice, label: &str) -> DialogButton {
    DialogButton {
        choice,
        label: label.to_owned(),
        availability: Availability::Enabled,
        default_focus: false,
        destructive: false,
    }
}

fn dialog(
    kind: DialogKind,
    title: &str,
    message: &[&str],
    buttons: Vec<DialogButton>,
) -> DialogView {
    DialogView {
        kind,
        title: title.to_owned(),
        message: message.iter().map(|line| (*line).to_owned()).collect(),
        error: None,
        impact: Vec::new(),
        options: None,
        fields: Vec::new(),
        buttons,
    }
}

/// Every dialog fixture, in the order of the spec (§7, §6).
pub fn dialogs() -> Vec<DialogView> {
    use DialogChoice as D;
    let focus = |mut button: DialogButton| {
        button.default_focus = true;
        button
    };
    let danger = |mut button: DialogButton| {
        button.destructive = true;
        button
    };
    vec![
        dialog(
            DialogKind::UnsavedChanges,
            "Save changes to Sample project?",
            &["Your changes are lost if you don't save them."],
            vec![
                button(D::Save, "Save"),
                danger(button(D::Discard, "Discard")),
                focus(button(D::Cancel, "Cancel")),
            ],
        ),
        DialogView {
            impact: vec![
                ImpactRow {
                    label: String::from("Models deleted"),
                    count: 1,
                },
                ImpactRow {
                    label: String::from("Members deleted"),
                    count: 3,
                },
                ImpactRow {
                    label: String::from("Relationships deleted"),
                    count: 2,
                },
                ImpactRow {
                    label: String::from("Views removed from diagrams"),
                    count: 2,
                },
            ],
            ..dialog(
                DialogKind::DeleteEverywhere,
                "Delete Customer everywhere?",
                &["This deletes the model from the project and removes it from every diagram."],
                vec![
                    danger(button(D::DeleteEverywhere, "Delete everywhere")),
                    focus(button(D::Cancel, "Cancel")),
                ],
            )
        },
        DialogView {
            error: Some(String::from(
                "A class name must contain a non-whitespace character.",
            )),
            ..dialog(
                DialogKind::ResolveEdit,
                "Resolve the edit first",
                &["The Name field has an edit that cannot be saved."],
                vec![
                    focus(button(D::ReturnToField, "Return to field")),
                    danger(button(D::DiscardEdit, "Discard edit")),
                ],
            )
        },
        DialogView {
            error: Some(String::from("The disk is full. Nothing was written.")),
            ..dialog(
                DialogKind::SaveFailed,
                "Save failed",
                &[
                    "Design fixture: C:\\Projects\\Sample project.dgram",
                    "Your changes are still open.",
                ],
                vec![
                    focus(button(D::Retry, "Retry")),
                    button(D::SaveAs, "Save As"),
                    button(D::Cancel, "Cancel"),
                ],
            )
        },
        DialogView {
            error: Some(String::from(
                "The file was made by a newer version of Diagram Studio.",
            )),
            ..dialog(
                DialogKind::OpenFailed,
                "Could not open the project",
                &[
                    "Design fixture: C:\\Projects\\Old.dgram",
                    "Your current project is unchanged.",
                ],
                vec![
                    focus(button(D::ChooseAnotherFile, "Choose another file")),
                    button(D::Cancel, "Cancel"),
                ],
            )
        },
        dialog(
            DialogKind::Recovery,
            "Recover unsaved work?",
            &[
                "Design fixture: a recovery snapshot of Sample project from 10:42.",
                "Restoring opens it as a new unsaved project; the original file is not changed.",
            ],
            vec![
                focus(button(D::Restore, "Restore")),
                button(D::Ignore, "Ignore"),
            ],
        ),
        DialogView {
            options: Some((
                String::from("Diagram type"),
                vec![DialogOption {
                    label: String::from("Class diagram"),
                    selected: true,
                    availability: Availability::Enabled,
                }],
            )),
            fields: vec![FieldView {
                state: FieldState::Error(String::from(
                    "A diagram name must contain a non-whitespace character.",
                )),
                ..field(FieldId::NewDiagramName, "Diagram name", "")
            }],
            ..dialog(
                DialogKind::NewDiagram,
                "New diagram",
                &[],
                vec![
                    DialogButton {
                        availability: Availability::Disabled(FIXTURE_REASON),
                        ..button(D::Create, "Create")
                    },
                    focus(button(D::Cancel, "Cancel")),
                ],
            )
        },
    ]
}

/// The dialog fixture of `kind`.
pub fn dialog_of(kind: DialogKind) -> DialogView {
    dialogs()
        .into_iter()
        .find(|dialog| dialog.kind == kind)
        .unwrap_or_else(|| dialog(kind, "Dialog", &[], Vec::new()))
}

/// Validation drawer with `issues`.
pub fn validation(issues: ListState<IssueRow>, freshness: &str, tone: Tone) -> ValidationView {
    ValidationView {
        freshness: text(freshness, tone),
        issues,
        close: labeled(CommandId::ToggleIssues, "Close issues"),
    }
}

/// Two sample issues.
pub fn sample_issues() -> ListState<IssueRow> {
    ListState::Rows(vec![
        IssueRow {
            issue: Target::Issue(ObjectKey(50)),
            severity: Severity::Error,
            message: String::from("Relationship end has no class."),
            location: String::from("Association in Domain details"),
            go_to: Target::Edge(ObjectKey(30)),
        },
        IssueRow {
            issue: Target::Issue(ObjectKey(51)),
            severity: Severity::Warning,
            message: String::from("Class Order has no attributes."),
            location: String::from("Class Order in Class overview"),
            go_to: Target::Node(ObjectKey(21)),
        },
    ])
}

/// Command search with `query`, listing the matching fixture commands.
pub fn command_search(query: &str) -> CommandSearchView {
    let needle = query.trim().to_lowercase();
    let results: Vec<CommandResult> = CommandId::ALL
        .into_iter()
        .filter(|id| *id != CommandId::SearchCommands)
        .map(|id| {
            let view = command(id);
            let group = catalog_id(id)
                .map_or("Edit", |real| real.spec().group.label())
                .to_owned();
            CommandResult {
                command: view,
                group,
            }
        })
        .filter(|result| {
            needle.is_empty()
                || result.command.label.to_lowercase().contains(&needle)
                || result.group.to_lowercase().contains(&needle)
        })
        .collect();
    CommandSearchView {
        open: true,
        query: query.to_owned(),
        results: if results.is_empty() {
            ListState::NoMatch
        } else {
            ListState::Rows(results)
        },
    }
}

/// Object search with `query` and `results`.
pub fn object_search(query: &str, results: Option<ListState<ObjectResult>>) -> ObjectSearchView {
    ObjectSearchView {
        field: search_field(SearchScope::Objects, query),
        results,
    }
}

/// Object search results for "Order": two models with the same name.
pub fn order_results() -> ListState<ObjectResult> {
    ListState::Rows(vec![
        ObjectResult {
            target: Target::Model(ObjectKey(11)),
            name: String::from("Order"),
            kind: String::from("Class"),
            location: String::from("Class overview"),
        },
        ObjectResult {
            target: Target::Model(ObjectKey(12)),
            name: String::from("Order"),
            kind: String::from("Class"),
            location: String::from("Domain details"),
        },
    ])
}

/// The sample workspace: docked panels, nothing selected.
pub fn sample() -> Presentation {
    Presentation {
        layout: LayoutView {
            mode: crate::ui::workspace::LayoutMode::Docked,
            navigator_shown: true,
            inspector_shown: true,
            drawer: None,
            focused_region: None,
        },
        header: header(),
        tabs: tabs(),
        tools: tools(),
        navigator: navigator(NavigatorTab::Model, None),
        canvas: canvas(),
        inspector: inspector_none(),
        status: status(),
        validation: None,
        dialog: None,
        command_search: CommandSearchView {
            open: false,
            query: String::new(),
            results: ListState::Empty,
        },
    }
}

/// The sample workspace with Customer selected.
pub fn sample_with_class_selected() -> Presentation {
    Presentation {
        navigator: navigator(NavigatorTab::Model, Some(10)),
        canvas: canvas_selected(&[20]),
        inspector: inspector_class("Customer", FieldState::Committed),
        ..sample()
    }
}

/// Every named workspace state, for the audit.
pub fn workspace_states() -> Vec<(&'static str, Presentation)> {
    let mut states = vec![
        ("sample", sample()),
        ("class selected", sample_with_class_selected()),
    ];
    let mut with = |name, change: &dyn Fn(&mut Presentation)| {
        let mut presentation = sample();
        change(&mut presentation);
        states.push((name, presentation));
    };
    with("name error", &|p| {
        p.canvas = canvas_selected(&[20]);
        p.inspector = inspector_class(
            "",
            FieldState::Error(String::from(
                "A class name must contain a non-whitespace character.",
            )),
        );
    });
    with("relationship selected", &|p| {
        p.canvas = canvas_selected(&[30]);
        p.inspector = inspector_relationship();
    });
    with("multiple selected", &|p| {
        p.canvas = canvas_selected(&[20, 21, 22]);
        p.inspector = inspector_multiple();
    });
    with("mixed selected", &|p| {
        p.canvas = canvas_selected(&[20, 21, 30]);
        p.inspector = inspector_mixed();
    });
    with("member editor", &|p| {
        p.inspector = inspector_member_editor()
    });
    with("empty diagram", &|p| p.canvas = canvas_empty());
    with("marquee", &|p| p.canvas = canvas_marquee());
    with("placement", &|p| {
        p.tools = tools_with(CommandId::AddClass, false);
        p.canvas = canvas_placement();
    });
    with("connect valid", &|p| {
        p.tools = tools_with(CommandId::Connect, false);
        p.canvas = canvas_connect(true);
    });
    with("connect invalid", &|p| {
        p.tools = tools_with(CommandId::Connect, true);
        p.canvas = canvas_connect(false);
    });
    with("hand tool", &|p| {
        p.tools = tools_with(CommandId::HandTool, false)
    });
    with("many tabs", &|p| p.tabs = many_tabs());
    with("history labels", &|p| p.header = header_with_history());
    with("validation drawer", &|p| {
        p.validation = Some(validation(sample_issues(), "Current", Tone::Neutral));
    });
    with("navigator loading", &|p| {
        p.navigator = navigator_models(ListState::Loading, "")
    });
    with("navigator empty", &|p| {
        p.navigator = navigator_models(ListState::Empty, "")
    });
    with("navigator no match", &|p| {
        p.navigator = navigator_models(ListState::NoMatch, "Invoice");
    });
    with("diagrams tab", &|p| {
        p.navigator = navigator(NavigatorTab::Diagrams, None)
    });
    states
}
