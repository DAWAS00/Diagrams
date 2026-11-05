//! Editor state outside the document: tool, gesture, tabs with their own
//! viewport and selection, preferences, region focus, drafts and searches.
//!
//! Nothing here is part of the saved project, so none of it can mark the
//! project dirty or enter document history.

use diagram_canvas::{ScreenSize, Viewport};
use diagram_core::Point;

use crate::drafts::Draft;
use crate::intent::{Region, UiIntent};

/// The active canvas tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    /// Click to select, drag empty space for a marquee, drag a view to move it.
    #[default]
    Select,
    /// Drag to pan the view; a drag never moves a node.
    Hand,
}

/// The one gesture in progress. An explicit enum, so contradictory states
/// (dragging while connecting while editing) cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Gesture {
    /// Nothing in progress.
    #[default]
    Idle,
    /// Panning; `last` is the previous pointer position on screen.
    Pan {
        /// Previous pointer position, relative to the canvas origin.
        last: Point,
    },
    /// Selecting with a rectangle between two screen points.
    Marquee {
        /// Where the drag started.
        start: Point,
        /// Where the pointer is now.
        current: Point,
        /// Shift was held: add to the selection.
        additive: bool,
    },
    /// Dragging selected views; only a preview until the move command exists.
    MovePreview {
        /// Where the drag started.
        start: Point,
        /// Where the pointer is now.
        current: Point,
    },
    /// Resizing a view; only a preview until the resize command exists.
    ResizePreview {
        /// Where the drag started.
        start: Point,
        /// Where the pointer is now.
        current: Point,
    },
    /// Placing a new class; `at` is the preview position in world units.
    Placement {
        /// Preview position.
        at: Point,
    },
    /// Choosing the source of a relationship.
    ConnectSource,
    /// Choosing the target of a relationship from `source`.
    ConnectTarget {
        /// Index of the source class.
        source: usize,
    },
    /// Editing a name directly on the canvas.
    InlineEdit,
}

impl Gesture {
    /// A geometric preview that a tool switch or Escape discards.
    pub const fn is_preview(self) -> bool {
        !matches!(self, Self::Idle | Self::InlineEdit)
    }
}

/// One open diagram tab.
#[derive(Debug, Clone, PartialEq)]
pub struct TabState {
    /// Index of the diagram in the project's diagram list.
    pub diagram: usize,
    /// This tab's own pan and zoom.
    pub viewport: Viewport,
    /// Selection kept while another tab is active.
    pub selection: Vec<usize>,
}

impl TabState {
    /// A tab for `diagram` at 100% with nothing selected.
    pub fn new(diagram: usize) -> Self {
        Self {
            diagram,
            viewport: Viewport::default(),
            selection: Vec::new(),
        }
    }
}

/// Per-user preferences stored outside the project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preferences {
    /// Show the grid.
    pub grid: bool,
    /// Snap placement and movement to the grid.
    pub snap: bool,
}

/// What the running shell draws and sends. Commands whose effect only a
/// surface listed here can show are disabled, with a reason, until the shell
/// says it draws that surface. Today's shell supports none of them; the new
/// screens (Agent A) set [`ShellSupport::NEW_SCREENS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShellSupport {
    /// The inspector sends `CommitDraft` on Enter and on leaving a field, and
    /// `CancelDraft` on Escape. Without it a valid name commits as it is typed.
    pub draft_events: bool,
    /// The Resolve edit dialog is drawn from `Presentation::dialog`.
    pub resolve_dialog: bool,
    /// The canvas draws the grid and the status bar shows grid and snap.
    pub grid: bool,
    /// The project panel draws the object search.
    pub object_search: bool,
    /// The shell moves keyboard focus to `Presentation::focus_region`.
    pub region_focus: bool,
}

impl ShellSupport {
    /// Everything the new screens draw.
    pub const NEW_SCREENS: Self = Self {
        draft_events: true,
        resolve_dialog: true,
        grid: true,
        object_search: true,
        region_focus: true,
    };
}

/// Canvas size assumed until the shell reports the real one.
pub const DEFAULT_CANVAS: ScreenSize = ScreenSize {
    width: 640.0,
    height: 360.0,
};

/// Editor state that is not part of the document.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorState {
    /// Chosen tool.
    pub tool: Tool,
    /// Gesture in progress.
    pub gesture: Gesture,
    /// Space is held: drags pan whatever the tool.
    pub temporary_pan: bool,
    /// Open tabs.
    pub tabs: Vec<TabState>,
    /// Index of the active tab in [`Self::tabs`].
    pub active_tab: usize,
    /// Region with keyboard focus.
    pub focus_region: Region,
    /// Preferences.
    pub preferences: Preferences,
    /// Size of the canvas on screen.
    pub canvas_size: ScreenSize,
    /// The uncommitted field edit, if any.
    pub draft: Option<Draft>,
    /// An action held back until an invalid draft is resolved.
    pub pending: Option<Box<UiIntent>>,
    /// The object search query while the object search is open.
    pub object_search: Option<String>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            gesture: Gesture::Idle,
            temporary_pan: false,
            tabs: vec![TabState::new(0)],
            active_tab: 0,
            focus_region: Region::Board,
            preferences: Preferences::default(),
            canvas_size: DEFAULT_CANVAS,
            draft: None,
            pending: None,
            object_search: None,
        }
    }
}

impl EditorState {
    /// The tool drags use now: Hand while Space is held.
    pub fn effective_tool(&self) -> Tool {
        if self.temporary_pan {
            Tool::Hand
        } else {
            self.tool
        }
    }

    /// The active tab's viewport.
    pub fn viewport(&self) -> Viewport {
        self.tabs
            .get(self.active_tab)
            .map(|tab| tab.viewport)
            .unwrap_or_default()
    }

    /// Replaces the active tab's viewport.
    pub fn set_viewport(&mut self, viewport: Viewport) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.viewport = viewport;
        }
    }

    /// Chooses a tool; a geometric preview in progress is discarded.
    pub fn set_tool(&mut self, tool: Tool) {
        if self.gesture.is_preview() {
            self.gesture = Gesture::Idle;
        }
        self.tool = tool;
    }

    /// The center of the canvas, relative to its origin.
    pub fn canvas_center(&self) -> Point {
        Point::new(self.canvas_size.width / 2.0, self.canvas_size.height / 2.0)
            .unwrap_or(Point::ORIGIN)
    }
}
