//! UI-04: welcome and project entry screens. Sample entries are labeled, and
//! nothing pretends to open or save real files.

use diagram_desktop::shell::{self, SAMPLE_PROJECT, ShellState};
use diagram_desktop::ui::welcome::{self, EntryScreen, RecentStatus};
use eframe::egui::accesskit::Rect;
use eframe::egui::{Key, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

const BLANK_NAME_ERROR: &str = "A project name must contain a non-whitespace character.";

fn harness_at(width: f32, height: f32) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(width, height))
        .build_ui_state(shell::show, ShellState::welcome());
    harness.run();
    harness
}

fn harness() -> Harness<'static, ShellState> {
    harness_at(1280.0, 800.0)
}

fn screen(harness: &Harness<'_, ShellState>) -> Option<EntryScreen> {
    harness.state().entry.as_ref().map(|entry| entry.screen)
}

fn disabled(harness: &Harness<'_, ShellState>, label: &str) -> bool {
    harness.get_by_label(label).accesskit_node().is_disabled()
}

fn open_form(harness: &mut Harness<'_, ShellState>) {
    harness.get_by_label("New project").click();
    harness.run();
}

#[test]
fn name_rule_should_come_from_the_core_project_name() {
    assert_eq!(welcome::name_error("Customer model"), None);
    assert_eq!(welcome::name_error("  x  "), None);
    assert_eq!(welcome::name_error("").as_deref(), Some(BLANK_NAME_ERROR));
    assert_eq!(
        welcome::name_error(" \t ").as_deref(),
        Some(BLANK_NAME_ERROR)
    );
}

#[test]
fn recent_search_should_match_name_or_path() {
    let recent = welcome::sample_recent_projects();

    assert_eq!(welcome::matching_recent(&recent, ""), [0, 1, 2]);
    assert_eq!(welcome::matching_recent(&recent, "ORDER"), [1]);
    assert_eq!(welcome::matching_recent(&recent, "archive"), [2]);
    assert!(welcome::matching_recent(&recent, "zzz").is_empty());
    assert_eq!(recent[2].status, RecentStatus::Missing);
}

#[test]
fn app_should_start_on_the_welcome_screen() {
    let harness = harness();

    assert_eq!(screen(&harness), Some(EntryScreen::Welcome));
    // The name appears in the header and as the page title.
    assert_eq!(harness.get_all_by_label("Diagram Studio").count(), 2);
    harness.get_by_label("New project");
    harness.get_by_label("Recent projects");
    assert!(harness.query_by_label("Diagram canvas").is_none());
    assert!(
        harness.query_by_label("No selection").is_none(),
        "entry screens have no selection"
    );
}

#[test]
fn sample_entries_should_be_labeled_as_a_fixture() {
    let harness = harness();

    harness.get_by_label_contains("Sample entries (design fixture)");
    harness.get_by_label_contains("Sample notice (design fixture)");
}

#[test]
fn nothing_should_pretend_to_open_real_files() {
    let harness = harness();

    assert!(disabled(&harness, "Open project…"));
    assert!(disabled(
        &harness,
        r"Customer model — D:\Projects\customer-model.dgram"
    ));
    assert!(disabled(
        &harness,
        r"Order workflow — D:\Projects\order-workflow.dgram"
    ));
    assert!(disabled(&harness, "Review recovery"));
    assert!(!disabled(&harness, "New project"));
    assert!(!disabled(&harness, "Continue with the sample project"));
}

#[test]
fn missing_file_should_explain_and_offer_removal() {
    let mut harness = harness();

    harness.get_by_label_contains("Error: File not found");
    harness
        .get_by_label("Remove Old inventory from the list")
        .click();
    harness.run();

    let names: Vec<_> = harness
        .state()
        .entry
        .as_ref()
        .map(|entry| {
            entry
                .recent
                .iter()
                .map(|project| project.name.clone())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(names, ["Customer model", "Order workflow"]);
    assert!(harness.query_by_label_contains("File not found").is_none());
}

#[test]
fn recent_search_should_filter_and_explain_no_matches() {
    let mut harness = harness();
    harness.get_by_label("Search recent projects").focus();
    harness.run();

    harness
        .get_by_label("Search recent projects")
        .type_text("order");
    harness.run();
    assert!(
        harness
            .query_by_label_contains("Customer model —")
            .is_none()
    );
    harness.get_by_label_contains("Order workflow —");

    harness
        .get_by_label("Search recent projects")
        .type_text("zzz");
    harness.run();
    harness.get_by_label_contains("No recent projects match");
}

#[test]
fn recovery_notice_should_be_dismissible() {
    let mut harness = harness();
    harness.get_by_label_contains("Unsaved changes to Customer model were recovered");

    harness.get_by_label("Dismiss recovery").click();
    harness.run();

    assert!(harness.query_by_label("Review recovery").is_none());
}

#[test]
fn new_project_form_should_focus_the_name_and_offer_only_class_diagrams() {
    let mut harness = harness();

    open_form(&mut harness);

    assert_eq!(screen(&harness), Some(EntryScreen::NewProject));
    assert!(harness.get_by_label("Project name").is_focused());
    harness.get_by_label("Class diagram");
    assert_eq!(welcome::CREATABLE_TYPES, ["Class diagram"]);
    assert!(harness.query_by_label("Sequence diagram").is_none());
    assert!(disabled(&harness, "Create project"), "nothing typed yet");
    assert!(
        harness.query_by_label_contains("Error:").is_none(),
        "no error before typing"
    );
}

#[test]
fn blank_name_should_show_the_error_beside_the_field() {
    let mut harness = harness();
    open_form(&mut harness);

    harness.get_by_label("Project name").type_text("   ");
    harness.run();

    harness.get_by_label(&format!("Error: {BLANK_NAME_ERROR}"));
    assert!(disabled(&harness, "Create project"));
}

#[test]
fn create_should_open_an_empty_diagram_with_the_exact_name() {
    let mut harness = harness();
    open_form(&mut harness);

    harness
        .get_by_label("Project name")
        .type_text("  Customer model ");
    harness.run();
    harness.get_by_label("Create project").click();
    harness.run();

    assert!(harness.state().entry.is_none());
    assert_eq!(harness.state().project_name, "  Customer model ");
    assert!(harness.state().classes.is_empty());
    harness.get_by_label("Diagram canvas");
    // The exact spelling, including spaces, is kept in the header.
    harness.get_by_label_contains("  Customer model  · not saved");
}

#[test]
fn enter_in_the_name_field_should_create_the_project() {
    let mut harness = harness();
    open_form(&mut harness);

    harness.get_by_label("Project name").type_text("Orders");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert!(harness.state().entry.is_none());
    assert_eq!(harness.state().project_name, "Orders");
}

#[test]
fn cancel_should_return_to_welcome_and_focus_new_project() {
    let mut harness = harness();
    open_form(&mut harness);

    harness.get_by_label("Cancel").click();
    harness.run();

    assert_eq!(screen(&harness), Some(EntryScreen::Welcome));
    assert!(harness.get_by_label("New project").is_focused());
}

#[test]
fn escape_should_leave_the_field_then_cancel_the_form() {
    let mut harness = harness();
    open_form(&mut harness);
    harness.get_by_label("Project name").type_text("Draft");
    harness.run();

    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(screen(&harness), Some(EntryScreen::NewProject));

    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(screen(&harness), Some(EntryScreen::Welcome));
}

#[test]
fn continue_should_open_the_sample_project() {
    let mut harness = harness();

    harness
        .get_by_label("Continue with the sample project")
        .click();
    harness.run();

    assert!(harness.state().entry.is_none());
    assert_eq!(harness.state().project_name, SAMPLE_PROJECT);
    assert_eq!(harness.state().classes.len(), 2);
    harness.get_by_label("Class Customer");
}

#[test]
fn entry_screens_should_fit_the_smallest_window() {
    let (width, height) = (800.0, 500.0);
    let mut harness = harness_at(width, height);
    let inside = |rect: Rect| {
        rect.x0 >= 0.0
            && rect.y0 >= 0.0
            && rect.x1 <= f64::from(width)
            && rect.y1 <= f64::from(height)
    };
    let bounds = |harness: &Harness<'_, ShellState>, label: &str| {
        harness
            .get_by_label(label)
            .accesskit_node()
            .bounding_box()
            .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0))
    };

    for label in [
        "New project",
        "Open project…",
        "Continue with the sample project",
    ] {
        assert!(inside(bounds(&harness, label)), "{label}");
    }
    open_form(&mut harness);
    for label in ["Project name", "Create project", "Cancel"] {
        assert!(inside(bounds(&harness, label)), "{label}");
    }
}

#[test]
fn typing_should_continue_after_the_error_appears() {
    let mut harness = harness();
    open_form(&mut harness);

    harness.get_by_label("Project name").type_text(" ");
    harness.run();
    harness.get_by_label(&format!("Error: {BLANK_NAME_ERROR}"));
    // No refocus: the field must still have focus after switching to its error state.
    harness.get_by_label("Project name").type_text("Orders");
    harness.run();

    assert!(harness.get_by_label("Project name").is_focused());
    assert_eq!(
        harness
            .state()
            .entry
            .as_ref()
            .map(|entry| entry.new_name.as_str()),
        Some(" Orders")
    );
    assert!(harness.query_by_label_contains("Error:").is_none());
}
