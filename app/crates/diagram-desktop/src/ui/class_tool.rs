//! The Class tool (S5): placing a new class on the canvas and naming it.
//!
//! The active tool and the one pending placement are plain data owned by the
//! shell's coordinator. The rules here are pure and tested directly; the
//! naming panel only draws and queues intents. Decision D2 (an existing name
//! is reported in words and the user chooses) is [`decide`].

use std::collections::BTreeSet;

use diagram_core::ids::{DiagramId, ElementId};
use diagram_core::model::Document;
use eframe::egui::{self, CornerRadius, Id, Key, Rect, Stroke, StrokeKind, UiBuilder, Vec2};

use crate::shell::ShellState;
use crate::ui::canvas;
use crate::ui::components;
use crate::ui::inspector::{self as rules, DraftKey};
use crate::ui::intents::UiIntent;
use crate::ui::theme::{self, Palette, TextRole};

/// The fixed ID of the new-class name field, so the coordinator can prepare
/// its text selection and focus can return to it.
pub const NEW_CLASS_FIELD: &str = "new-class-name";

/// Label of the new-class name field.
pub const NEW_CLASS_LABEL: &str = "New class name";

/// Hint under the field while the name is valid.
pub const NEW_CLASS_HINT: &str = "Enter creates the class. Escape cancels.";

/// Width of the naming panel's contents.
const PANEL_WIDTH: f32 = 240.0;

/// Grid margin and gap used by [`next_free_spot`].
const MARGIN: i32 = 48;
const GAP: i32 = 40;
/// How many grid cells [`next_free_spot`] looks at at most.
const MAX_CELLS: i32 = 200;

/// The egui ID of the new-class name field.
pub fn new_class_field() -> Id {
    Id::unique(NEW_CLASS_FIELD)
}

/// The egui ID of the naming panel's click blocker.
fn field_panel_id() -> Id {
    Id::unique("new-class-panel")
}

/// The active canvas tool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    /// Click selects.
    #[default]
    Select,
    /// Click places a new class.
    Class,
}

/// A whole-point position on the canvas, relative to its top-left corner.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CanvasPoint {
    /// Points from the left edge.
    pub x: i32,
    /// Points from the top edge.
    pub y: i32,
}

impl CanvasPoint {
    /// The point (`x`, `y`).
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A whole-point size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CanvasSize {
    /// Width in points.
    pub width: i32,
    /// Height in points.
    pub height: i32,
}

impl CanvasSize {
    /// The size `width` × `height`.
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// `size` rounded up to whole points.
    pub fn of(size: Vec2) -> Self {
        Self::new(size.x.ceil() as i32, size.y.ceil() as i32)
    }
}

/// A rectangle on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasRect {
    /// Top-left corner.
    pub at: CanvasPoint,
    /// Size.
    pub size: CanvasSize,
}

impl CanvasRect {
    /// Whether the two rectangles share any area.
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.at.x < other.at.x + other.size.width
            && other.at.x < self.at.x + self.size.width
            && self.at.y < other.at.y + other.size.height
            && other.at.y < self.at.y + self.size.height
    }
}

/// The size of a new, empty class box, from the canvas's own constants.
pub const CLASS_SIZE: CanvasSize = canvas::EMPTY_CARD_SIZE;

/// Identity of one placement, so a late or repeated request can be told apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementId(u64);

/// Where a placement is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The name field is being edited.
    Naming,
    /// The duplicate-name dialog is asking; `existing` is the class that
    /// "Show the existing class here" would show, if any.
    Asking {
        /// The class to show here, if one is not already in this diagram.
        existing: Option<ElementId>,
    },
}

/// A new class being placed and named. Owned by the coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    /// Identity.
    pub id: PlacementId,
    /// The diagram it is placed on.
    pub diagram: DiagramId,
    /// Top-left of the new class box.
    pub at: CanvasPoint,
    /// The visible canvas when the placement began.
    pub visible: CanvasSize,
    /// Naming or asking.
    pub phase: Phase,
}

/// The tool and the pending placement, kept together so their rules live in
/// one place. Changed only by the coordinator.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClassTool {
    tool: Tool,
    placement: Option<Placement>,
    issued: u64,
}

impl ClassTool {
    /// The active tool.
    pub const fn tool(&self) -> Tool {
        self.tool
    }

    /// The pending placement, if any.
    pub const fn placement(&self) -> Option<&Placement> {
        self.placement.as_ref()
    }

    /// The placement while its name is being edited.
    pub fn naming(&self) -> Option<Placement> {
        self.placement
            .filter(|placement| placement.phase == Phase::Naming)
    }

    /// Makes `tool` the active tool.
    pub(crate) fn arm(&mut self, tool: Tool) {
        self.tool = tool;
    }

    /// Starts a placement, if the Class tool is active and none exists. The
    /// tool goes back to Select.
    pub(crate) fn begin(
        &mut self,
        diagram: DiagramId,
        at: CanvasPoint,
        visible: CanvasSize,
    ) -> Option<PlacementId> {
        if self.tool != Tool::Class || self.placement.is_some() {
            return None;
        }
        self.issued += 1;
        let id = PlacementId(self.issued);
        self.tool = Tool::Select;
        self.placement = Some(Placement {
            id,
            diagram,
            at,
            visible,
            phase: Phase::Naming,
        });
        Some(id)
    }

    /// Moves the placement `id` to `phase`, if it is the live one.
    pub(crate) fn set_phase(&mut self, id: PlacementId, phase: Phase) {
        if let Some(placement) = self.placement.as_mut().filter(|p| p.id == id) {
            placement.phase = phase;
        }
    }

    /// Back to Select with no placement, as for a newly opened project. The
    /// ID counter keeps counting, so a late request from the previous
    /// project never matches a new placement.
    pub(crate) fn reset(&mut self) {
        self.tool = Tool::Select;
        self.placement = None;
    }

    /// Ends the placement, returning it.
    pub(crate) fn take(&mut self) -> Option<Placement> {
        self.placement.take()
    }
}

/// How trying to finish naming ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    /// Nothing is being named any more (created, refused, or nothing to do).
    Completed,
    /// The name is invalid; naming goes on.
    Blocked,
    /// The duplicate-name dialog is asking.
    Asking,
}

/// The core rectangle for a box of `size` at `at`.
pub fn bounds(at: CanvasPoint, size: CanvasSize) -> diagram_core::Rect {
    diagram_core::Rect::new(
        diagram_core::Point::new(f64::from(at.x), f64::from(at.y))
            .unwrap_or(diagram_core::Point::ORIGIN),
        diagram_core::Size::new(f64::from(size.width), f64::from(size.height))
            .unwrap_or(diagram_core::Size::UNIT),
    )
}

/// What committing a typed name should do (D2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// No class has this name: create one with the trimmed name.
    Create(String),
    /// A class has this name: ask. `existing` is the lowest-ID class with
    /// the name that is not already in the diagram, or `None` when every one
    /// is already there.
    Ask {
        /// The trimmed name.
        name: String,
        /// The class "Show the existing class here" would show.
        existing: Option<ElementId>,
    },
}

/// Decides what committing `typed` on `diagram` does. Names are compared
/// trimmed and case-sensitively.
pub fn decide(document: &Document, diagram: DiagramId, typed: &str) -> Decision {
    let name = typed.trim().to_owned();
    let shown: BTreeSet<ElementId> = document
        .diagrams
        .get(&diagram)
        .map(|diagram| diagram.nodes.iter().map(|node| node.element).collect())
        .unwrap_or_default();
    let mut same: Vec<ElementId> = document
        .elements
        .values()
        .filter(|element| element.name.as_str().trim() == name)
        .map(|element| element.id)
        .collect();
    if same.is_empty() {
        return Decision::Create(name);
    }
    same.sort_unstable();
    let existing = same.into_iter().find(|id| !shown.contains(id));
    Decision::Ask { name, existing }
}

/// "Class N" with the smallest N ≥ 1 that no class is named.
pub fn default_class_name(document: &Document) -> String {
    let taken: BTreeSet<&str> = document
        .elements
        .values()
        .map(|element| element.name.as_str().trim())
        .collect();
    (1..)
        .map(|number: u64| format!("Class {number}"))
        .find(|name| !taken.contains(name.as_str()))
        .unwrap_or_default()
}

/// Moves `at` so a box of `footprint` is inside a canvas of `visible`. Never
/// negative; 0 on an axis where the canvas is smaller than the box.
pub fn fit_inside(at: CanvasPoint, footprint: CanvasSize, visible: CanvasSize) -> CanvasPoint {
    let fit = |value: i32, size: i32, room: i32| value.min(room - size).max(0);
    CanvasPoint::new(
        fit(at.x, footprint.width, visible.width),
        fit(at.y, footprint.height, visible.height),
    )
}

/// The first free cell of a grid inside `visible` for a box of `footprint`,
/// row by row, that overlaps none of `occupied`. When no visible cell is
/// free, the first cell, kept inside the canvas (overlap is allowed until
/// pan and zoom arrive).
pub fn next_free_spot(
    occupied: &[CanvasRect],
    footprint: CanvasSize,
    visible: CanvasSize,
) -> CanvasPoint {
    let pitch_x = footprint.width + GAP;
    let pitch_y = footprint.height + GAP;
    // Euclidean division rounds down, so a canvas narrower than one cell
    // has no cells (plain division would round -0.5 up to 0 and count one).
    let columns = ((visible.width - MARGIN - footprint.width).div_euclid(pitch_x) + 1).max(0);
    let rows = ((visible.height - MARGIN - footprint.height).div_euclid(pitch_y) + 1).max(0);
    let cells = (columns * rows).min(MAX_CELLS);
    (0..cells)
        .map(|cell| {
            CanvasPoint::new(
                MARGIN + (cell % columns) * pitch_x,
                MARGIN + (cell / columns) * pitch_y,
            )
        })
        .find(|&at| {
            let candidate = CanvasRect {
                at,
                size: footprint,
            };
            !occupied.iter().any(|taken| taken.overlaps(&candidate))
        })
        .unwrap_or_else(|| fit_inside(CanvasPoint::new(MARGIN, MARGIN), footprint, visible))
}

/// Draws the naming panel of the pending placement inside the canvas and
/// queues its commit or cancel. `canvas` is the canvas rectangle on screen.
pub(crate) fn naming_panel(
    ui: &mut egui::Ui,
    state: &mut ShellState,
    canvas: Rect,
    palette: &Palette,
) {
    let Some(placement) = state.placement().copied() else {
        return;
    };
    let owns = placement.phase == Phase::Naming && !state.has_dialog() && state.palette.is_none();
    let mut text = state
        .drafts
        .get(&DraftKey::NewClassName)
        .cloned()
        .unwrap_or_default();
    let error = rules::class_name_error(&text);

    // Measured before drawing: the panel keeps the same size with or without
    // the error, so the first frame with an error neither jumps nor clips.
    let measure = |text: String, role: TextRole| {
        ui.painter()
            .layout(text, role.font_id(), palette.text_secondary, PANEL_WIDTH)
            .size()
            .y
    };
    let label = measure(NEW_CLASS_LABEL.to_owned(), TextRole::Label);
    let hint = measure(NEW_CLASS_HINT.to_owned(), TextRole::Label);
    let error_height = measure(
        format!("Error: {}", crate::shell::BLANK_NAME_WARNING),
        TextRole::Body,
    );
    let gap = ui.spacing().item_spacing.y;
    let inner = Vec2::new(
        PANEL_WIDTH,
        label + gap + theme::CONTROL_HEIGHT + gap + hint.max(error_height),
    );
    let outer = inner + Vec2::splat(2.0 * theme::SPACING);
    let at = fit_inside(
        placement.at,
        CanvasSize::of(outer),
        CanvasSize::of(canvas.size()),
    );
    let panel = Rect::from_min_size(canvas.min + Vec2::new(at.x as f32, at.y as f32), outer);
    ui.painter().rect(
        panel,
        CornerRadius::same(4),
        palette.panel,
        Stroke::new(1.5, palette.accent),
        StrokeKind::Inside,
    );
    // The whole panel takes clicks, so its padding never reaches the canvas
    // or a class box under it. Its widgets are added after, so they win.
    let _blocker = ui.interact(panel, field_panel_id(), egui::Sense::click());
    let field = new_class_field();
    let response = ui
        .scope_builder(
            UiBuilder::new().max_rect(panel.shrink(theme::SPACING)),
            |ui| {
                let response = components::text_field_at(
                    ui,
                    NEW_CLASS_LABEL,
                    &mut text,
                    field,
                    error.as_deref(),
                );
                if error.is_none() {
                    components::field_label(ui, NEW_CLASS_HINT);
                }
                response
            },
        )
        .inner;
    if response.changed() {
        state.drafts.insert(DraftKey::NewClassName, text.clone());
    }
    if !owns || !response.lost_focus() {
        return;
    }
    if ui.input(|input| input.key_pressed(Key::Escape)) {
        state.queue(UiIntent::CancelNewClass(placement.id));
    } else if rules::class_name_error(&text).is_some() {
        // Checked on the text as edited this frame, not the frame's start.
        // The field keeps focus while its error shows.
        response.request_focus();
    } else {
        state.queue(UiIntent::CommitNewClass {
            placement: placement.id,
            name: text,
        });
    }
}
