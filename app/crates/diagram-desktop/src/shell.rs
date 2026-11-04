//! Native shell prototype: toolbar, sidebar, canvas, inspector and status bar.
//!
//! The preview classes are prototype data owned by the desktop crate. They
//! exercise layout, selection and real text editing before M1 introduces the
//! core model, commands and history.

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Frame, Key, Layout, Pos2, Rect, Role,
    Sense, Stroke, StrokeKind, ThemePreference, Vec2, WidgetInfo,
};

use crate::ui::components;
use crate::ui::theme::{self, Palette};

const CLASS_WIDTH: f32 = 180.0;
const HEADER_HEIGHT: f32 = 32.0;
const ATTRIBUTE_HEIGHT: f32 = 20.0;
const NOT_IMPLEMENTED: &str = "Not available in this prototype yet";

/// Document colors for diagram objects. They do not follow the editor theme,
/// so switching themes never changes how a diagram looks when exported.
const DOCUMENT_FILL: Color32 = Color32::from_rgb(0xFA, 0xFC, 0xFF);
const DOCUMENT_INK: Color32 = Color32::from_rgb(0x22, 0x31, 0x47);

/// Message shown when a class name has no visible characters.
pub const BLANK_NAME_WARNING: &str = "A class name must contain a non-whitespace character.";

/// A class box drawn on the prototype canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewClass {
    /// Editable class name, bound directly to the inspector text field.
    pub name: String,
    /// Attribute lines displayed below the name.
    pub attributes: Vec<String>,
    /// Top-left corner, relative to the canvas origin.
    pub position: Vec2,
}

impl PreviewClass {
    /// Name used in the canvas and accessibility tree; blank names get a placeholder.
    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            "Unnamed class"
        } else {
            &self.name
        }
    }

    fn size(&self) -> Vec2 {
        let rows = self.attributes.len().max(1) as f32;
        Vec2::new(CLASS_WIDTH, HEADER_HEIGHT + rows * ATTRIBUTE_HEIGHT + 8.0)
    }
}

/// Editor state for the prototype shell.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellState {
    /// Classes shown on the canvas, in drawing order.
    pub classes: Vec<PreviewClass>,
    /// Index into [`Self::classes`] of the selected class, if any.
    pub selected: Option<usize>,
    /// Light, dark or follow the operating system. Light is the default.
    pub theme: ThemePreference,
    /// Theme preference currently installed in the egui context.
    applied_theme: Option<ThemePreference>,
    /// Whether a text field had keyboard focus at the end of the last frame.
    text_had_focus: bool,
}

impl Default for ShellState {
    fn default() -> Self {
        let class = |name: &str, attributes: &[&str], x, y| PreviewClass {
            name: name.to_owned(),
            attributes: attributes.iter().map(|a| (*a).to_owned()).collect(),
            position: Vec2::new(x, y),
        };
        Self {
            classes: vec![
                class("Customer", &["id: CustomerId", "name: String"], 48.0, 48.0),
                class("Order", &["id: OrderId", "total: Money"], 300.0, 96.0),
            ],
            selected: None,
            theme: ThemePreference::Light,
            applied_theme: None,
            text_had_focus: false,
        }
    }
}

impl ShellState {
    /// The currently selected class, if the selection index is valid.
    pub fn selected_class(&self) -> Option<&PreviewClass> {
        self.selected.and_then(|index| self.classes.get(index))
    }

    /// Selects the class at `index`; out-of-range indices clear the selection.
    pub fn select(&mut self, index: usize) {
        self.selected = (index < self.classes.len()).then_some(index);
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }
}

/// Draws the complete shell into `ui` and applies the user's interactions to `state`.
pub fn show(ui: &mut egui::Ui, state: &mut ShellState) {
    if state.applied_theme != Some(state.theme) {
        theme::apply(ui.ctx(), state.theme);
        state.applied_theme = Some(state.theme);
    }
    let palette = *theme::palette(ui.ctx().theme());

    // egui drops text focus as soon as Escape is pressed, so use the focus
    // from the previous frame: Escape first leaves the field, then deselects.
    if !state.text_had_focus && ui.input(|input| input.key_pressed(Key::Escape)) {
        state.clear_selection();
    }

    let bar = Frame::side_top_panel(ui.style()).fill(palette.workspace);
    egui::Panel::top("toolbar").frame(bar).show(ui, toolbar);
    egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| status_bar(ui, state));
    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(220.0)
        .min_size(160.0)
        .show(ui, |ui| sidebar(ui, state));
    egui::Panel::right("inspector")
        .resizable(true)
        .default_size(280.0)
        .min_size(200.0)
        .show(ui, |ui| inspector(ui, state));
    egui::CentralPanel::default()
        .frame(Frame::central_panel(ui.style()).fill(palette.workspace))
        .show(ui, |ui| canvas(ui, state, &palette));
    state.text_had_focus = ui.ctx().egui_wants_keyboard_input();
}

fn toolbar(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.strong(crate::APPLICATION_NAME);
        ui.separator();
        for label in ["Undo", "Redo", "Add class", "Connect", "Save", "Export"] {
            components::action_button(ui, label, Some(NOT_IMPLEMENTED), None);
        }
    });
}

fn status_bar(ui: &mut egui::Ui, state: &mut ShellState) {
    ui.horizontal(|ui| {
        ui.label("Prototype · nothing is saved");
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left layout: add the options in reverse visual order.
            for (preference, label) in [
                (ThemePreference::System, "System"),
                (ThemePreference::Dark, "Dark"),
                (ThemePreference::Light, "Light"),
            ] {
                if components::choice(ui, state.theme == preference, label).clicked() {
                    state.theme = preference;
                }
            }
            ui.label("Theme:");
            ui.separator();
            let selection = match state.selected_class() {
                Some(class) => format!("Selected: {}", class.display_name()),
                None => String::from("No selection"),
            };
            components::truncated_text(ui, &selection);
        });
    });
}

fn sidebar(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Project");
    components::field_label(ui, "Diagrams");
    ui.indent("diagrams", |ui| {
        let _ = components::list_row(ui, true, "Class diagram 1");
    });
    ui.add_space(theme::SPACING);
    components::field_label(ui, "Model");
    ui.indent("model", |ui| {
        for index in 0..state.classes.len() {
            let selected = state.selected == Some(index);
            let label = state.classes[index].display_name().to_owned();
            if components::list_row(ui, selected, &label).clicked() {
                state.select(index);
            }
        }
    });
}

fn inspector(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Selection");
    let Some(class) = state
        .selected
        .and_then(|index| state.classes.get_mut(index))
    else {
        ui.label("Nothing selected. Select a class on the canvas or in the Model list.");
        return;
    };

    components::text_field(ui, "Name", &mut class.name);
    if class.name.trim().is_empty() {
        components::warning_text(ui, BLANK_NAME_WARNING);
    }

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Attributes");
    for attribute in &class.attributes {
        ui.monospace(attribute);
    }
}

fn canvas(ui: &mut egui::Ui, state: &mut ShellState, palette: &Palette) {
    let (background, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
    background.widget_info(|| WidgetInfo::labeled(Role::Canvas, true, "Diagram canvas"));
    painter.rect_filled(background.rect, CornerRadius::ZERO, palette.canvas);
    let origin = background.rect.min;

    let mut clicked_class = None;
    for (index, class) in state.classes.iter().enumerate() {
        let rect = Rect::from_min_size(origin + class.position, class.size());
        let selected = state.selected == Some(index);
        let response = ui.interact(
            rect,
            ui.scope_id().with(("canvas-class", index)),
            Sense::click(),
        );
        let name = class.display_name();
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, true, selected, format!("Class {name}"))
        });
        if response.clicked() {
            clicked_class = Some(index);
        }
        paint_class(&painter, rect, class, selected, response.hovered(), palette);
    }

    match clicked_class {
        Some(index) => state.select(index),
        None if background.clicked() => state.clear_selection(),
        None => {}
    }
}

fn paint_class(
    painter: &egui::Painter,
    rect: Rect,
    class: &PreviewClass,
    selected: bool,
    hovered: bool,
    palette: &Palette,
) {
    // Selection and hover are editor overlays in the theme accent; the box
    // itself keeps its document colors.
    let border = if selected || hovered {
        palette.accent
    } else {
        DOCUMENT_INK
    };
    let width = if selected { 2.5 } else { 1.0 };

    painter.rect(
        rect,
        CornerRadius::same(4),
        DOCUMENT_FILL,
        Stroke::new(width, border),
        StrokeKind::Inside,
    );
    let header_bottom = rect.top() + HEADER_HEIGHT;
    painter.hline(
        rect.x_range(),
        header_bottom,
        Stroke::new(1.0, DOCUMENT_INK),
    );
    painter.text(
        Pos2::new(rect.center().x, rect.top() + HEADER_HEIGHT / 2.0),
        Align2::CENTER_CENTER,
        class.display_name(),
        FontId::proportional(15.0),
        DOCUMENT_INK,
    );
    for (row, attribute) in class.attributes.iter().enumerate() {
        painter.text(
            Pos2::new(
                rect.left() + 10.0,
                header_bottom + 6.0 + row as f32 * ATTRIBUTE_HEIGHT,
            ),
            Align2::LEFT_TOP,
            attribute,
            FontId::monospace(13.0),
            DOCUMENT_INK,
        );
    }

    if selected {
        // Corner handles mark selection without relying on colour alone.
        for corner in [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ] {
            let handle = Rect::from_center_size(corner, Vec2::splat(8.0));
            painter.rect(
                handle,
                CornerRadius::ZERO,
                Color32::WHITE,
                Stroke::new(1.5, palette.accent),
                StrokeKind::Middle,
            );
        }
    }
}

/// eframe application wrapper around [`ShellState`].
#[derive(Debug, Default)]
pub struct ShellApp {
    state: ShellState,
}

impl eframe::App for ShellApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, &mut self.state);
    }
}
