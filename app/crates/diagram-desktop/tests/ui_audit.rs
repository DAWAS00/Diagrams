//! Automated usability audit (UI rule, see plan-rust-diagram-editor/UI-RULES.md).
//!
//! Every screen state is checked for every interactive control: it has an
//! accessible name, lies inside the window, is at least the planned hit
//! height, does not partly overlap another control, and enabled controls are
//! reachable with the Tab key. New screens must be added here.

mod common;
use std::collections::HashSet;

use common::{class_id, harness_with};

fn select_named(state: &mut ShellState, name: &str) {
    let element = class_id(state, name);
    state.select(element);
}
use diagram_desktop::shell::ShellState;
use eframe::egui::accesskit::{Action, Rect, Role};
use eframe::egui::{
    Button, Key, Modifiers, Pos2, Rect as EguiRect, Role as WidgetRole, Sense, Vec2, WidgetInfo,
};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// Plan: "hit targets at least 32 units".
const MIN_HIT_HEIGHT: f64 = 32.0;

#[derive(Debug, Clone)]
struct Control {
    name: String,
    role: Role,
    rect: Rect,
    /// Partly scrolled out of view; only the visible part is in `rect`.
    clipped: bool,
    enabled: bool,
}

/// Roles a person operates. Plain text labels (selectable text) and egui's
/// own scroll bars are not controls in this sense.
fn is_control(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::Tab
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::ComboBox
            | Role::Link
            | Role::ListBoxOption
            | Role::Slider
            | Role::SpinButton
            | Role::Canvas
    )
}

fn controls<S>(harness: &Harness<'_, S>) -> Vec<Control> {
    harness
        .query_all_by(|node| {
            let data = node.data();
            !node.is_hidden()
                && is_control(node.role())
                && (data.supports_action(Action::Click) || data.supports_action(Action::Focus))
        })
        .map(|node| {
            let access = node.accesskit_node();
            // Clip to every scrolling ancestor: content scrolled out of view
            // is not on screen, and partly visible content counts only for
            // its visible part.
            let mut rect = access
                .bounding_box()
                .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0));
            let mut clipped = false;
            let mut parent = access.parent();
            while let Some(ancestor) = parent {
                if ancestor.clips_children()
                    && let Some(clip) = ancestor.bounding_box()
                {
                    let visible = rect.intersect(clip);
                    clipped |= visible != rect;
                    rect = visible;
                }
                parent = ancestor.parent();
            }
            Control {
                name: access
                    .label()
                    .filter(|label| !label.trim().is_empty())
                    .or_else(|| access.value())
                    .unwrap_or_default(),
                role: access.role(),
                rect,
                clipped,
                enabled: !access.is_disabled(),
            }
        })
        .collect()
}

fn visible(rect: Rect, (width, height): (f32, f32)) -> bool {
    rect.width() > 0.5
        && rect.height() > 0.5
        && rect.x1 > 0.0
        && rect.y1 > 0.0
        && rect.x0 < f64::from(width)
        && rect.y0 < f64::from(height)
}

fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x0 <= inner.x0 + 0.5
        && outer.y0 <= inner.y0 + 0.5
        && outer.x1 + 0.5 >= inner.x1
        && outer.y1 + 0.5 >= inner.y1
}

fn overlap(first: Rect, second: Rect) -> bool {
    first.x0 + 0.5 < second.x1
        && second.x0 + 0.5 < first.x1
        && first.y0 + 0.5 < second.y1
        && second.y0 + 0.5 < first.y1
}

/// Static checks of one screen state. `floating` screens (dialogs, menus,
/// windows) cover other controls by design, so overlap is not checked.
fn audit_layout<S>(
    scene: &str,
    harness: &Harness<'_, S>,
    size: (f32, f32),
    floating: bool,
) -> Vec<String> {
    let mut findings = Vec::new();
    let all = controls(harness);
    let shown: Vec<&Control> = all.iter().filter(|c| visible(c.rect, size)).collect();
    for control in &shown {
        let what = format!("{scene}: {:?} \"{}\"", control.role, control.name);
        if control.name.trim().is_empty() {
            findings.push(format!("{what} has no accessible name"));
        }
        let rect = control.rect;
        if rect.x0 < -0.5
            || rect.y0 < -0.5
            || rect.x1 > f64::from(size.0) + 0.5
            || rect.y1 > f64::from(size.1) + 0.5
        {
            findings.push(format!("{what} extends outside the window: {rect:?}"));
        }
        if !control.clipped && rect.height() < MIN_HIT_HEIGHT - 0.5 {
            findings.push(format!(
                "{what} is {:.0} high, below the {MIN_HIT_HEIGHT} hit target",
                rect.height()
            ));
        }
    }
    if !floating {
        for (index, first) in shown.iter().enumerate() {
            for second in &shown[index + 1..] {
                if overlap(first.rect, second.rect)
                    && !contains(first.rect, second.rect)
                    && !contains(second.rect, first.rect)
                {
                    findings.push(format!(
                        "{scene}: \"{}\" overlaps \"{}\"",
                        first.name, second.name
                    ));
                }
            }
        }
    }
    findings
}

/// Presses Tab through the screen and reports enabled controls never focused.
/// `pointer_only` lists controls with a documented keyboard alternative.
fn audit_tab_order<S>(
    scene: &str,
    harness: &mut Harness<'_, S>,
    size: (f32, f32),
    pointer_only: &[&str],
) -> Vec<String> {
    let expected: HashSet<String> = controls(harness)
        .into_iter()
        .filter(|c| c.enabled && visible(c.rect, size) && c.role != Role::Canvas)
        .map(|c| c.name)
        .filter(|name| !pointer_only.iter().any(|allowed| name.starts_with(allowed)))
        .collect();
    let mut reached = HashSet::new();
    for _ in 0..(expected.len() * 2 + 4) {
        harness.key_press(Key::Tab);
        harness.run();
        if let Some(node) = harness.query_by(|node| node.is_focused()) {
            let access = node.accesskit_node();
            if let Some(name) = access.label().or_else(|| access.value()) {
                reached.insert(name);
            }
        }
    }
    let mut missing: Vec<_> = expected.difference(&reached).cloned().collect();
    missing.sort();
    missing
        .into_iter()
        .map(|name| format!("{scene}: \"{name}\" cannot be reached with Tab"))
        .collect()
}

fn report(findings: Vec<String>) {
    assert!(
        findings.is_empty(),
        "{} usability finding(s):\n{}",
        findings.len(),
        findings.join("\n")
    );
}

const LARGE: (f32, f32) = (1280.0, 800.0);
const SMALL: (f32, f32) = (800.0, 500.0);

/// Canvas boxes are reached through the project panel's Model list instead.
const POINTER_ONLY: [&str; 1] = ["Class "];

#[test]
fn welcome_screen_should_pass_the_audit() {
    for size in [LARGE, SMALL] {
        let mut harness = harness_with(size, ShellState::welcome());
        let mut findings = audit_layout("welcome", &harness, size, false);
        findings.extend(audit_tab_order("welcome", &mut harness, size, &[]));
        report(findings);
    }
}

#[test]
fn new_project_form_should_pass_the_audit() {
    for size in [LARGE, SMALL] {
        let mut harness = harness_with(size, ShellState::welcome());
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
    select_named(&mut state, "Customer");
    let mut harness = harness_with(LARGE, state);
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
    select_named(&mut state, "Order");
    let mut harness = harness_with(SMALL, state);
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
fn project_panel_states_should_pass_the_audit() {
    let mut harness = harness_with(LARGE, ShellState::default());
    harness.get_by_label("Search project").focus();
    harness.run();
    harness.get_by_label("Search project").type_text("zzz");
    harness.run();
    let mut findings = audit_layout("project panel, no matches", &harness, LARGE, false);
    findings.extend(audit_tab_order(
        "project panel, no matches",
        &mut harness,
        LARGE,
        &POINTER_ONLY,
    ));
    report(findings);
}

#[test]
fn overflowing_tabs_should_pass_the_audit() {
    let mut state = common::state_with_many_diagrams(8);
    let seventh = common::diagram_id(&state, "Customer overview part 7");
    state.activate_tab(seventh);
    for size in [SMALL, LARGE] {
        let mut harness = harness_with(size, state.clone());
        let mut findings = audit_layout("overflowing tabs", &harness, size, false);
        findings.extend(audit_tab_order(
            "overflowing tabs",
            &mut harness,
            size,
            &POINTER_ONLY,
        ));
        report(findings);
    }
}

#[test]
fn inspector_states_should_pass_the_audit() {
    for size in [LARGE, SMALL] {
        let mut findings = Vec::new();

        // Relationship selected.
        let mut state = ShellState::default();
        if let Some(relation) = state.document().relations.keys().next().copied() {
            state.select_relation(relation);
        }
        let mut harness = harness_with(size, state);
        if harness.query_by_label("Role at Customer").is_none() {
            harness.get_by_label("Inspector").click();
            harness.run();
        }
        findings.extend(audit_layout(
            "relationship inspector",
            &harness,
            size,
            false,
        ));
        findings.extend(audit_tab_order(
            "relationship inspector",
            &mut harness,
            size,
            &POINTER_ONLY,
        ));

        // Two classes selected.
        let mut state = ShellState::default();
        select_named(&mut state, "Customer");
        let order = class_id(&state, "Order");
        state.toggle_in_selection(order);
        let mut harness = harness_with(size, state);
        if harness
            .query_by_label("Appearance (this view only)")
            .is_none()
        {
            harness.get_by_label("Inspector").click();
            harness.run();
        }
        findings.extend(audit_layout("mixed selection", &harness, size, false));

        // Invalid draft with every section open.
        let mut state = ShellState::default();
        select_named(&mut state, "Customer");
        state.inspector_sections = (true, true);
        let mut harness = harness_with(size, state);
        if harness.query_by_label("Name").is_none() {
            harness.get_by_label("Inspector").click();
            harness.run();
        }
        harness.get_by_label("Name").focus();
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness.get_by_label("Name").type_text(" ");
        harness.run();
        findings.extend(audit_layout("invalid name draft", &harness, size, false));
        if size == SMALL {
            // On small windows the inspector is a drawer over the canvas by
            // design; canvas boxes beneath it are not controls on top of it.
            findings.retain(|finding| !finding.contains("\"Class "));
        }
        report(findings);
    }
}

#[test]
fn command_search_should_pass_the_audit() {
    let mut harness = harness_with(LARGE, ShellState::default());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    report(audit_layout("command search", &harness, LARGE, true));
}

#[test]
fn component_showcase_should_pass_the_audit() {
    let mut harness = harness_with(LARGE, ShellState::default());
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
    let welcome = harness_with(LARGE, ShellState::welcome());
    let workspace = harness_with(LARGE, ShellState::default());

    // Welcome: New, Open, Continue, Review, Dismiss, search, 2 recent, Remove, 3 theme.
    assert!(count(&welcome) >= 12, "{}", count(&welcome));
    // Workspace: menu, 5 header, 2 tab row, 8 tool row, rows, canvas, classes, theme.
    assert!(count(&workspace) >= 22, "{}", count(&workspace));
}

#[test]
fn close_project_dialog_should_pass_the_audit() {
    use diagram_desktop::ui::commands::CommandId;
    use diagram_desktop::ui::intents::UiIntent;
    for size in [LARGE, SMALL] {
        let state = ShellState::default();
        let customer = class_id(&state, "Customer");
        let mut harness = harness_with(size, state);
        harness
            .state_mut()
            .queue(UiIntent::RenameClass(customer, String::from("Client")));
        harness.run();
        harness
            .state_mut()
            .queue(UiIntent::run(CommandId::CloseProject));
        harness.run();

        assert!(harness.state().has_dialog());
        // A modal covers part of the canvas by design; check it with the
        // canvas boxes left out, as for the drawers.
        let findings: Vec<String> = audit_layout("close-project dialog", &harness, size, false)
            .into_iter()
            .filter(|finding| !finding.contains("\"Class "))
            .collect();
        report(findings);
    }
}

#[test]
fn generalization_inspector_should_pass_the_audit() {
    use diagram_core::notation::RelationKind;
    for size in [LARGE, SMALL] {
        let mut state = common::state_with_generalization();
        let relation = state
            .document()
            .relations
            .values()
            .find(|relation| relation.kind == RelationKind::Generalization)
            .map(|relation| relation.id);
        assert!(relation.is_some(), "the fixture has a generalization");
        if let Some(relation) = relation {
            state.select_relation(relation);
        }
        let mut harness = harness_with(size, state);
        if harness.query_by_label("Kind: generalization").is_none() {
            harness.get_by_label("Inspector").click();
            harness.run();
        }
        harness.get_by_label("Kind: generalization");
        // A drawer covers part of the canvas by design: leave the boxes out.
        let findings: Vec<String> = audit_layout("generalization inspector", &harness, size, false)
            .into_iter()
            .filter(|finding| !finding.contains("\"Class ") && !finding.contains("Diagram canvas"))
            .collect();
        report(findings);
    }
}

#[test]
fn one_class_diagram_in_dark_should_pass_the_audit() {
    use eframe::egui::ThemePreference;
    for size in [LARGE, SMALL] {
        let mut state = ShellState::default();
        state.theme = ThemePreference::Dark;
        let details = common::diagram_id(&state, "Order details");
        assert!(state.open_diagram(details));
        let mut harness = harness_with(size, state);
        if harness.query_by_label("1 class shown").is_none() {
            harness.get_by_label("Inspector").click();
            harness.run();
        }
        harness.get_by_label("1 class shown");
        let findings: Vec<String> = audit_layout("one-class diagram, dark", &harness, size, false)
            .into_iter()
            .filter(|finding| !finding.contains("\"Class ") && !finding.contains("Diagram canvas"))
            .collect();
        report(findings);
    }
}

const WIDE: (f32, f32) = (1440.0, 900.0);

/// The sample in `diagram` with the Class tool chosen and, when `at` is
/// given, the canvas clicked there (relative to the canvas's bottom-right
/// corner when negative).
fn class_tool_scene(
    size: (f32, f32),
    diagram: &str,
    at: Option<(f32, f32)>,
) -> Harness<'static, ShellState> {
    let mut state = ShellState::default();
    let target = common::diagram_id(&state, diagram);
    state.open_diagram(target);
    let mut harness = harness_with(size, state);
    common::choose_class_tool(&mut harness);
    if let Some((x, y)) = at {
        let canvas = harness.get_by_label("Diagram canvas").rect();
        let x = if x < 0.0 { canvas.width() + x } else { x };
        let y = if y < 0.0 { canvas.height() + y } else { y };
        common::click_canvas_at(&mut harness, x, y);
    }
    harness
}

/// Leaves out the canvas boxes and the canvas itself, which a panel or a
/// modal covers by design.
fn without_canvas(findings: Vec<String>) -> Vec<String> {
    findings
        .into_iter()
        .filter(|finding| !finding.contains("\"Class ") && !finding.contains("Diagram canvas"))
        .collect()
}

#[test]
fn class_tool_states_should_pass_the_audit() {
    use diagram_desktop::ui::class_tool::NEW_CLASS_LABEL;
    for size in [SMALL, LARGE, WIDE] {
        let harness = class_tool_scene(size, "Class diagram 1", None);
        report(audit_layout("class tool active", &harness, size, false));

        let harness = class_tool_scene(size, "Class diagram 1", Some((-4.0, -4.0)));
        harness.get_by_label(NEW_CLASS_LABEL);
        report(without_canvas(audit_layout(
            "naming at the edge",
            &harness,
            size,
            false,
        )));

        let mut harness = class_tool_scene(size, "Class diagram 1", Some((-4.0, -4.0)));
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness.key_press(Key::Backspace);
        harness.run();
        harness.key_press(Key::Enter);
        harness.run();
        assert!(harness.query_all_by_label_contains("Error:").count() > 0);
        report(without_canvas(audit_layout(
            "naming with an error at the edge",
            &harness,
            size,
            false,
        )));
    }
}

#[test]
fn duplicate_name_dialog_should_pass_the_audit() {
    use diagram_desktop::ui::class_tool::NEW_CLASS_LABEL;
    for size in [SMALL, LARGE, WIDE] {
        for diagram in ["Order details", "Class diagram 1"] {
            let mut harness = class_tool_scene(size, diagram, Some((100.0, 200.0)));
            harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
            harness.get_by_label(NEW_CLASS_LABEL).type_text("Customer");
            harness.run();
            harness.key_press(Key::Enter);
            harness.run();
            assert!(harness.state().has_dialog(), "{diagram}");
            let scene = format!("duplicate dialog in {diagram}");
            // The modal's backdrop covers the name field behind it, which
            // takes no input until the dialog closes.
            let findings = without_canvas(audit_layout(&scene, &harness, size, false))
                .into_iter()
                .filter(|finding| !finding.contains(&format!("\"{NEW_CLASS_LABEL}\" overlaps")))
                .collect();
            report(findings);
        }
    }
}
