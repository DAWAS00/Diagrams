//! Responsive workspace composition (UI-03): row heights, panel sizes, and
//! when side panels dock or become drawers.

/// Height of the header row.
pub const HEADER_HEIGHT: f32 = 48.0;
/// Height of the diagram tab row.
pub const TAB_ROW_HEIGHT: f32 = 38.0;
/// Height of the tool row.
pub const TOOL_ROW_HEIGHT: f32 = 40.0;
/// Narrowest canvas kept beside docked panels.
pub const MIN_CANVAS_WIDTH: f32 = 480.0;

/// Default width and resize range of a side panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelSize {
    /// Width when first shown.
    pub default: f32,
    /// Narrowest width the user can drag to.
    pub min: f32,
    /// Widest width the user can drag to.
    pub max: f32,
}

/// The project panel (navigator) on the left.
pub const NAVIGATOR: PanelSize = PanelSize {
    default: 240.0,
    min: 180.0,
    max: 360.0,
};

/// The inspector on the right.
pub const INSPECTOR: PanelSize = PanelSize {
    default: 300.0,
    min: 240.0,
    max: 420.0,
};

/// How the side panels are shown at the current window width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Both panels can sit beside a canvas of at least [`MIN_CANVAS_WIDTH`].
    Docked,
    /// Too narrow: panels open one at a time as drawers over the canvas.
    Narrow,
}

/// Layout for a window `width` logical points wide.
pub fn layout_mode(width: f32) -> LayoutMode {
    if width >= NAVIGATOR.default + INSPECTOR.default + MIN_CANVAS_WIDTH {
        LayoutMode::Docked
    } else {
        LayoutMode::Narrow
    }
}

/// A side panel shown as a drawer on narrow windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drawer {
    /// The project panel, from the left.
    Navigator,
    /// The inspector, from the right.
    Inspector,
}

impl Drawer {
    /// Drawer width, never more than `available` minus a strip of canvas.
    pub fn width(self, available: f32) -> f32 {
        let preferred = match self {
            Self::Navigator => NAVIGATOR.default,
            Self::Inspector => INSPECTOR.default,
        };
        preferred.min((available - 48.0).max(NAVIGATOR.min))
    }
}

/// Which docked panels the user wants. Kept while the window changes size,
/// so a hidden panel stays hidden after a narrow phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelPreferences {
    /// Show the project panel when docked.
    pub navigator_visible: bool,
    /// Show the inspector when docked.
    pub inspector_visible: bool,
}

impl Default for PanelPreferences {
    fn default() -> Self {
        Self {
            navigator_visible: true,
            inspector_visible: true,
        }
    }
}
