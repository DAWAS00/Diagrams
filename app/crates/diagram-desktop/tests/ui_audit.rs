//! Automated usability audit (UI rule, see plan-rust-diagram-editor/UI-RULES.md).
//!
//! Every screen state is checked for every interactive control: it has an
//! accessible name, lies inside the window, is at least the planned hit
//! height, does not partly overlap another control, and enabled controls are
//! reachable with the Tab key. New screens must be added here.

mod common_ui;

use common_ui::{LARGE, SMALL, audit_layout, audit_tab_order, controls, report, visible};
use diagram_desktop::shell::{self, ShellState};
use eframe::egui::{
    Button, Key, Modifiers, Pos2, Rect as EguiRect, Role as WidgetRole, Sense, Vec2, WidgetInfo,
};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn harness_at(size: (f32, f32), state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(size.0, size.1))
        .build_ui_state(shell::show, state);
    harness.run();
    harness
}

/// Canvas boxes are reached through the project panel's Model list instead.
const POINTER_ONLY: [&str; 1] = ["Class "];

#[test]
fn welcome_screen_should_pass_the_audit() {
    for size in [LARGE, SMALL] {
        let mut harness = harness_at(size, ShellState::welcome());
        let mut findings = audit_layout("welcome", &harness, size, false);
        findings.extend(audit_tab_order("welcome", &mut harness, size, &[]));
        report(findings);
    }
}

#[test]
fn new_project_form_should_pass_the_audit() {
    for size in [LARGE, SMALL] {
        let mut harness = harness_at(size, ShellState::welcome());
        harness.get_by_label("New project").click();
        harness.run();
        harness.get_by_label("Project name").type_text(" ");
        harness.run();
        let mut findings = audit_layout("new project (with error)", &harness, size, false);
        findings.extend(audit_tab_order("new project", &mut harness, size, &[]));
        report(findings);
    }
}

#[test]
fn docked_workspace_should_pass_the_audit() {
    let mut state = ShellState::default();
    state.select(0);
    let mut harness = harness_at(LARGE, state);
    let mut findings = audit_layout("docked workspace", &harness, LARGE, false);
    findings.extend(audit_tab_order(
        "docked workspace",
        &mut harness,
        LARGE,
        &POINTER_ONLY,
    ));
    report(findings);
}

#[test]
fn narrow_workspace_and_drawers_should_pass_the_audit() {
    let mut state = ShellState::default();
    state.select(1);
    let mut harness = harness_at(SMALL, state);
    let mut findings = audit_layout("narrow workspace", &harness, SMALL, false);
    for toggle in ["Inspector", "Project panel"] {
        harness.get_by_label(toggle).click();
        harness.run();
        let scene = format!("narrow workspace, {toggle} drawer");
        // The drawer covers part of the canvas by design; check it with the
        // canvas boxes left out.
        findings.extend(
            audit_layout(&scene, &harness, SMALL, false)
                .into_iter()
                .filter(|finding| !finding.contains("\"Class ")),
        );
        findings.extend(audit_tab_order(&scene, &mut harness, SMALL, &POINTER_ONLY));
    }
    report(findings);
}

#[test]
fn command_search_should_pass_the_audit() {
    let mut harness = harness_at(LARGE, ShellState::default());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    report(audit_layout("command search", &harness, LARGE, true));
}

#[test]
fn component_showcase_should_pass_the_audit() {
    let mut harness = harness_at(LARGE, ShellState::default());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness
        .get_by_label("Find a command or object")
        .type_text("showcase");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    report(audit_layout("component showcase", &harness, LARGE, true));
}

#[test]
fn audit_should_report_broken_controls() {
    // egui_kittest's own accessibility check (on in every other test) would
    // stop at the unnamed control; turn it off to test this audit's checks.
    let mut harness = Harness::builder()
        .with_size(Vec2::new(400.0, 300.0))
        .with_accessibility_check(false)
        .build_ui(|ui| {
            // Too small, and a second button partly on top of it.
            ui.put(
                EguiRect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(80.0, 16.0)),
                Button::new("Tiny"),
            );
            ui.put(
                EguiRect::from_min_size(Pos2::new(50.0, 12.0), Vec2::new(80.0, 34.0)),
                Button::new("Overlapping"),
            );
            // A custom control that publishes accessibility info but no name.
            // (Without widget_info it would have no node at all: UI rule 6.)
            let unnamed = ui.interact(
                EguiRect::from_min_size(Pos2::new(10.0, 100.0), Vec2::new(60.0, 40.0)),
                ui.scope_id().with("unnamed"),
                Sense::click(),
            );
            unnamed.widget_info(|| WidgetInfo::labeled(WidgetRole::Button, true, ""));
            // Partly outside the window.
            ui.put(
                EguiRect::from_min_size(Pos2::new(360.0, 200.0), Vec2::new(80.0, 34.0)),
                Button::new("Clipped"),
            );
        });
    harness.run();

    let findings = audit_layout("self-test", &harness, (400.0, 300.0), false).join("\n");

    assert!(findings.contains("\"Tiny\" is 20 high"), "{findings}");
    assert!(
        findings.contains("\"Tiny\" overlaps \"Overlapping\""),
        "{findings}"
    );
    assert!(
        findings.contains("\"Clipped\" extends outside the window"),
        "{findings}"
    );
    assert!(findings.contains("has no accessible name"), "{findings}");
}

#[test]
fn audit_should_see_every_screen_s_controls() {
    let count = |harness: &Harness<'_, ShellState>| {
        controls(harness)
            .into_iter()
            .filter(|c| visible(c.rect, LARGE))
            .count()
    };
    let welcome = harness_at(LARGE, ShellState::welcome());
    let workspace = harness_at(LARGE, ShellState::default());

    // Welcome: New, Open, Continue, Review, Dismiss, search, 2 recent, Remove, 3 theme.
    assert!(count(&welcome) >= 12, "{}", count(&welcome));
    // Workspace: menu, 5 header, 2 tab row, 8 tool row, rows, canvas, classes, theme.
    assert!(count(&workspace) >= 22, "{}", count(&workspace));
}
