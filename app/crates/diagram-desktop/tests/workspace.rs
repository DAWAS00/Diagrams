//! UI-03: responsive workspace composition — rows, docked panels, drawers,
//! focus return and retained panel preferences.

mod common;
use common::harness_at;
use diagram_desktop::shell::ShellState;
use diagram_desktop::ui::workspace::{
    self, Drawer, INSPECTOR, LayoutMode, MIN_CANVAS_WIDTH, NAVIGATOR,
};
use eframe::egui::accesskit::{Rect, Toggled};
use eframe::egui::{Key, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn bounds(harness: &Harness<'_, ShellState>, label: &str) -> Rect {
    harness
        .get_by_label(label)
        .accesskit_node()
        .bounding_box()
        .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0))
}

fn pressed(harness: &Harness<'_, ShellState>, label: &str) -> bool {
    harness.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True)
}

fn overlaps(first: Rect, second: Rect) -> bool {
    first.x0 < second.x1 && second.x0 < first.x1 && first.y0 < second.y1 && second.y0 < first.y1
}

#[test]
fn panels_should_dock_only_when_a_full_canvas_still_fits() {
    let threshold = NAVIGATOR.default + INSPECTOR.default + MIN_CANVAS_WIDTH;

    assert_eq!(threshold, 1020.0);
    assert_eq!(workspace::layout_mode(threshold), LayoutMode::Docked);
    assert_eq!(workspace::layout_mode(threshold - 1.0), LayoutMode::Narrow);
    assert_eq!(workspace::layout_mode(1280.0), LayoutMode::Docked);
    assert_eq!(workspace::layout_mode(800.0), LayoutMode::Narrow);
}

#[test]
fn drawer_width_should_leave_a_strip_of_canvas() {
    assert_eq!(Drawer::Inspector.width(800.0), 300.0);
    assert_eq!(Drawer::Navigator.width(800.0), 240.0);
    assert_eq!(Drawer::Inspector.width(260.0), 212.0);
}

#[test]
fn rows_should_stack_header_tabs_tools_then_canvas() {
    let harness = harness_at(1280.0, 800.0);
    let middle = |rect: Rect| (rect.y0 + rect.y1) / 2.0;

    assert!(middle(bounds(&harness, "Menu")) < 48.0);
    let tabs = middle(bounds(&harness, "New diagram"));
    assert!((48.0..86.0).contains(&tabs), "tab row at {tabs}");
    let tools = middle(bounds(&harness, "Select"));
    assert!((86.0..126.0).contains(&tools), "tool row at {tools}");
    assert!(bounds(&harness, "Diagram canvas").y0 >= 126.0);
}

#[test]
fn wide_window_should_dock_both_panels_beside_a_full_canvas() {
    let harness = harness_at(1280.0, 800.0);

    assert_eq!(harness.state().layout, LayoutMode::Docked);
    harness.get_by_label("Project");
    harness.get_by_label("Selection");
    assert!(pressed(&harness, "Project panel"));
    assert!(pressed(&harness, "Inspector"));
    assert!(bounds(&harness, "Diagram canvas").width() >= f64::from(MIN_CANVAS_WIDTH) - 16.0);
}

#[test]
fn narrow_window_should_hide_panels_and_give_the_canvas_the_width() {
    let harness = harness_at(800.0, 500.0);

    assert_eq!(harness.state().layout, LayoutMode::Narrow);
    assert!(harness.query_by_label("Project").is_none());
    assert!(harness.query_by_label("Selection").is_none());
    assert!(!pressed(&harness, "Project panel"));
    assert!(bounds(&harness, "Diagram canvas").width() >= 760.0);
}

#[test]
fn inspector_drawer_should_cover_only_the_canvas() {
    let mut harness = harness_at(800.0, 500.0);

    harness.get_by_label("Inspector").click();
    harness.run();

    assert_eq!(harness.state().drawer, Some(Drawer::Inspector));
    assert!(pressed(&harness, "Inspector"));
    let canvas = bounds(&harness, "Diagram canvas");
    let drawer = bounds(&harness, "Close inspector");
    assert!(
        drawer.x0 >= canvas.x0 && drawer.x1 <= canvas.x1 + 1.0,
        "{drawer:?}"
    );
    assert!(
        drawer.y0 >= canvas.y0,
        "{drawer:?} above the canvas {canvas:?}"
    );
    for control in [
        "Menu",
        "Save",
        "Select",
        "Inspector",
        "Light",
        "No selection",
    ] {
        assert!(
            !overlaps(bounds(&harness, "Selection"), bounds(&harness, control)),
            "drawer covers {control}"
        );
    }
}

#[test]
fn opening_a_drawer_should_close_the_other_one() {
    let mut harness = harness_at(800.0, 500.0);

    harness.get_by_label("Project panel").click();
    harness.run();
    harness.get_by_label("Inspector").click();
    harness.run();

    assert_eq!(harness.state().drawer, Some(Drawer::Inspector));
    assert!(harness.query_by_label("Project").is_none());
}

#[test]
fn close_button_and_toggle_should_close_the_drawer() {
    let mut harness = harness_at(800.0, 500.0);
    harness.get_by_label("Project panel").click();
    harness.run();

    harness.get_by_label("Close project panel").click();
    harness.run();
    assert_eq!(harness.state().drawer, None);

    harness.get_by_label("Project panel").click();
    harness.run();
    harness.get_by_label("Project panel").click();
    harness.run();
    assert_eq!(harness.state().drawer, None);
}

#[test]
fn escape_should_close_the_drawer_and_return_focus_to_its_button() {
    let mut harness = harness_at(800.0, 500.0);
    harness.state_mut().select(0);
    harness.run();
    harness.get_by_label("Inspector").focus();
    harness.run();

    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().drawer, Some(Drawer::Inspector));

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().drawer, None);
    assert_eq!(
        harness.state().selected,
        Some(0),
        "Escape closed the drawer only"
    );
    assert!(harness.get_by_label("Inspector").is_focused());
}

#[test]
fn escape_while_typing_in_a_drawer_should_first_leave_the_field() {
    let mut harness = harness_at(800.0, 500.0);
    harness.state_mut().select(0);
    harness.run();
    harness.get_by_label("Inspector").click();
    harness.run();
    harness.get_by_label("Name").focus();
    harness.run();

    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().drawer, Some(Drawer::Inspector));

    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().drawer, None);
    assert_eq!(harness.state().selected, Some(0));
}

#[test]
fn renaming_in_the_drawer_should_update_the_canvas() {
    let mut harness = harness_at(800.0, 500.0);
    harness.state_mut().select(1);
    harness.run();
    harness.get_by_label("Inspector").click();
    harness.run();

    let name = harness.get_by_label("Name");
    name.focus();
    name.type_text("Line");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    harness.get_by_label("Class OrderLine");
}

#[test]
fn hidden_panel_should_stay_hidden_after_a_narrow_phase() {
    let mut harness = harness_at(1280.0, 800.0);

    harness.get_by_label("Project panel").click();
    harness.run();
    assert!(harness.query_by_label("Project").is_none());
    assert!(harness.state().panels.inspector_visible);

    harness.set_size(Vec2::new(800.0, 500.0));
    harness.run();
    assert_eq!(harness.state().layout, LayoutMode::Narrow);
    harness.set_size(Vec2::new(1280.0, 800.0));
    harness.run();

    assert_eq!(harness.state().layout, LayoutMode::Docked);
    assert!(harness.query_by_label("Project").is_none());
    harness.get_by_label("Selection");
}

#[test]
fn growing_the_window_should_close_an_open_drawer() {
    let mut harness = harness_at(800.0, 500.0);
    harness.get_by_label("Inspector").click();
    harness.run();

    harness.set_size(Vec2::new(1280.0, 800.0));
    harness.run();

    assert_eq!(harness.state().drawer, None);
    assert!(harness.query_by_label("Close inspector").is_none());
}

#[test]
fn menu_should_list_every_group_and_run_commands() {
    let mut harness = harness_at(1280.0, 800.0);
    harness.state_mut().select(0);
    harness.run();

    harness.get_by_label("Menu").click();
    harness.run();
    for group in ["File", "Edit", "View", "Tools", "Navigate"] {
        harness.get_by_label(group);
    }
    // Submenus open on hover.
    harness.get_by_label("Edit").hover();
    harness.run();
    harness.get_by_label_contains("Clear selection").click();
    harness.run();

    assert_eq!(harness.state().selected, None);
}

#[test]
fn escape_on_a_focused_button_should_still_clear_the_selection() {
    let mut harness = harness_at(1280.0, 800.0);
    harness.state_mut().select(1);
    harness.run();
    harness.get_by_label("Select").focus();
    harness.run();

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().selected, None);
}
