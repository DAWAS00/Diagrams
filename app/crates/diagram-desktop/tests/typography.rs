//! UI-02: text roles come from one theme module, and longer labels stay
//! inside the window at the supported sizes.

use diagram_desktop::shell::{self, ShellState};
use diagram_desktop::ui::components;
use diagram_desktop::ui::theme::{self, TextRole};
use eframe::egui::accesskit::Rect;
use eframe::egui::{Color32, FontFamily, LayerId, Painter, TextStyle, Theme, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

const LONG_NAME: &str = "PreferredCustomerAccountWithLoyaltyProgrammeAndRegionalBillingSettings";
const TOOLBAR: [&str; 7] = [
    "Undo",
    "Redo",
    "Add class",
    "Connect",
    "Save",
    "Export",
    "Search",
];
const SUPPORTED_SIZES: [(f32, f32); 3] = [(800.0, 500.0), (1280.0, 800.0), (1440.0, 900.0)];

fn harness_at(width: f32, height: f32, state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(width, height))
        .build_ui_state(shell::show, state);
    harness.run();
    harness
}

fn bounds(harness: &Harness<'_, ShellState>, label: &str) -> Rect {
    let node = harness.get_by_label(label);
    node.accesskit_node()
        .bounding_box()
        .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0))
}

fn assert_inside(rect: Rect, width: f32, height: f32, what: &str) {
    let (width, height) = (f64::from(width), f64::from(height));
    assert!(
        rect.x0 >= 0.0 && rect.y0 >= 0.0 && rect.x1 <= width && rect.y1 <= height,
        "{what} at {rect:?} is outside {width}x{height}"
    );
}

fn overlaps(first: Rect, second: Rect) -> bool {
    first.x0 < second.x1 && second.x0 < first.x1 && first.y0 < second.y1 && second.y0 < first.y1
}

#[test]
fn roles_should_have_the_planned_sizes() {
    let sizes: Vec<f32> = TextRole::ALL.iter().map(|role| role.size()).collect();

    assert_eq!(sizes, [24.0, 16.0, 15.0, 14.0, 14.0]);
    assert_eq!(TextRole::Code.font_id().family, FontFamily::Monospace);
    assert_eq!(TextRole::Body.font_id().family, FontFamily::Proportional);
}

#[test]
fn both_theme_styles_should_carry_every_role() {
    for theme_choice in [Theme::Light, Theme::Dark] {
        let style = theme::style(theme_choice);

        for role in TextRole::ALL {
            assert_eq!(
                style.text_styles.get(&role.text_style()),
                Some(&role.font_id()),
                "{role:?} in {theme_choice:?}"
            );
        }
    }
}

#[test]
fn shell_should_render_with_the_role_sizes() {
    let harness = harness_at(1280.0, 800.0, ShellState::default());
    let style = harness.ctx.global_style();

    assert_eq!(style.text_styles[&TextStyle::Body].size, 15.0);
    assert_eq!(style.text_styles[&TextStyle::Heading].size, 16.0);
    assert_eq!(style.text_styles[&TextStyle::Button].size, 14.0);
    assert_eq!(style.text_styles[&TextStyle::Monospace].size, 14.0);
}

#[test]
fn chrome_should_fit_without_overlap_at_every_supported_size() {
    for (width, height) in SUPPORTED_SIZES {
        let harness = harness_at(width, height, ShellState::default());
        let labels: Vec<&str> = TOOLBAR
            .iter()
            .copied()
            .chain(["Light", "Dark", "System", "No selection"])
            .collect();
        let rects: Vec<Rect> = labels.iter().map(|label| bounds(&harness, label)).collect();

        for (label, rect) in labels.iter().zip(&rects) {
            assert_inside(*rect, width, height, label);
        }
        for (first, first_rect) in labels.iter().zip(&rects) {
            for (second, second_rect) in labels.iter().zip(&rects) {
                assert!(
                    first == second || !overlaps(*first_rect, *second_rect),
                    "{first} overlaps {second} at {width}x{height}"
                );
            }
        }
    }
}

#[test]
fn long_class_name_should_not_push_controls_out_of_the_window() {
    for (width, height) in SUPPORTED_SIZES {
        let mut state = ShellState::default();
        state.classes[0].name = String::from(LONG_NAME);
        state.select(0);
        let harness = harness_at(width, height, state);

        for label in ["Light", "Dark", "System", "Name", "Undo", "Export"] {
            assert_inside(bounds(&harness, label), width, height, label);
        }
        let status = harness.get_by_label_contains("Selected: ");
        let status_rect = status
            .accesskit_node()
            .bounding_box()
            .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0));
        assert_inside(status_rect, width, height, "selection status");
        assert!(
            !overlaps(status_rect, bounds(&harness, "Light")),
            "status overlaps theme choice at {width}x{height}"
        );
    }
}

#[test]
fn long_class_name_should_stay_complete_in_the_accessibility_tree() {
    let mut state = ShellState::default();
    state.classes[0].name = String::from(LONG_NAME);
    state.select(0);
    let harness = harness_at(800.0, 500.0, state);

    harness.get_by_label(&format!("Class {LONG_NAME}"));
    assert_eq!(
        harness.get_by_label("Name").value().as_deref(),
        Some(LONG_NAME)
    );
}

#[test]
fn truncated_status_should_keep_the_full_name_for_screen_readers() {
    let mut state = ShellState::default();
    state.classes[0].name = String::from(LONG_NAME);
    state.select(0);
    let harness = harness_at(800.0, 500.0, state);

    let status = harness.get_by_label_contains("Selected: ");
    let expected = format!("Selected: {LONG_NAME}");
    assert_eq!(
        status.accesskit_node().value().as_deref(),
        Some(expected.as_str())
    );
}

#[test]
fn list_rows_should_stay_on_one_line_in_the_narrowest_sidebar() {
    let mut state = ShellState::default();
    state.classes[0].name = String::from(LONG_NAME);
    let harness = harness_at(800.0, 500.0, state);

    let canvas_left = bounds(&harness, "Diagram canvas").x0;
    for label in ["Class diagram 1", LONG_NAME, "Order"] {
        let rect = bounds(&harness, label);
        // One line is the control height (34); a wrapped row is about 50.
        assert!(
            rect.height() <= f64::from(theme::CONTROL_HEIGHT) + 1.0,
            "{label} wraps: {rect:?}"
        );
        assert!(
            rect.x1 <= canvas_left,
            "{label} leaves the sidebar: {rect:?}"
        );
    }
}

#[test]
fn painted_class_text_should_fit_its_box_width() {
    let harness = harness_at(800.0, 500.0, ShellState::default());
    let painter = Painter::new(
        harness.ctx,
        LayerId::background(),
        eframe::egui::Rect::EVERYTHING,
    );
    let max_width = 160.0;

    let long = components::one_line_galley(
        &painter,
        LONG_NAME,
        TextRole::Body.font_id(),
        Color32::BLACK,
        max_width,
    );
    let short = components::one_line_galley(
        &painter,
        "Order",
        TextRole::Body.font_id(),
        Color32::BLACK,
        max_width,
    );

    assert!(long.size().x <= max_width, "{}", long.size().x);
    assert_eq!(long.rows.len(), 1);
    assert!(long.elided);
    assert!(!short.elided);
}
