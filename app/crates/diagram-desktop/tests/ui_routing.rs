//! Routing hops H1 and H2 (TASKS-AND-TESTS.md §1), for Agent A's views:
//!
//! - H1: every command a view places renders a control with the same label
//!   (its accessible name) and the same enabled state, and a disabled one
//!   shows the catalog's reason;
//! - H2: activating that control (click or Enter) emits `Command(id)`
//!   exactly once, and a disabled control emits nothing.
//!
//! The tests loop over the presentation and over `CommandId::ALL`, so a new
//! catalog command is covered without editing a list here. The non-command
//! intents of §4 (select, draft, pointer, theme) are covered at the end.

mod common_ui;

use common_ui::{
    LARGE, Recorder, click, harness, in_side_panel, in_top_row, light, press_enter_on,
};
use diagram_desktop::intent::{DraftField, PointerKind, Target, UiIntent};
use diagram_desktop::presentation::{CommandView, Presentation};
use diagram_desktop::ui::commands::{Availability, CommandId};
use diagram_desktop::ui::{
    canvas_view, components, fixtures, header, inspector, navigator, status, tabs, toolbar,
    workspace,
};
use eframe::egui::{self, Key, Modifiers, ThemePreference, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// The three command rows, stacked as in the shell.
fn rows(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_top_row(ui, workspace::HEADER_HEIGHT, |ui| {
        header::show(ui, &view.header, out);
    });
    egui::Panel::top("tabs-under-test")
        .exact_size(workspace::TAB_ROW_HEIGHT)
        .show(ui, |ui| tabs::show(ui, &view.tabs, out));
    egui::Panel::top("tools-under-test")
        .exact_size(workspace::TOOL_ROW_HEIGHT)
        .show(ui, |ui| {
            toolbar::show(ui, &view.tools, out);
        });
}

/// Every command with a control of its own in the three rows.
fn placed(view: &Presentation) -> Vec<CommandView> {
    let tools = &view.tools;
    view.header
        .commands
        .iter()
        .chain(&view.tabs.commands)
        .chain(&tools.tools)
        .chain([&tools.navigator_toggle, &tools.inspector_toggle])
        .cloned()
        .collect()
}

fn all_enabled(mut view: Presentation) -> Presentation {
    let enable = |command: &mut CommandView| command.availability = Availability::Enabled;
    view.header.commands.iter_mut().for_each(enable);
    view.tabs.commands.iter_mut().for_each(enable);
    view.tools.tools.iter_mut().for_each(enable);
    for group in &mut view.header.menu {
        group.commands.iter_mut().for_each(enable);
    }
    view
}

#[test]
fn h1_every_placed_command_should_render_with_its_label_and_availability() {
    let view = fixtures::sample();
    for (size, theme) in [
        (LARGE, ThemePreference::Light),
        (common_ui::WIDE, ThemePreference::Dark),
    ] {
        let mut harness = harness(size, theme, view.clone(), rows);
        for command in placed(&view) {
            let node = harness.get_by_label(&command.label);
            assert_eq!(
                node.accesskit_node().is_disabled(),
                !command.is_enabled(),
                "{}",
                command.label
            );
            if let Some(reason) = command.availability.reason() {
                harness.get_by_label(&command.label).hover();
                harness.run();
                assert!(
                    harness.query_by_label_contains(reason).is_some(),
                    "{}: reason not shown on hover",
                    command.label
                );
            }
        }
    }
}

#[test]
fn h2_every_enabled_placed_command_should_emit_its_command_once() {
    let view = all_enabled(fixtures::sample());
    let mut harness = light(view.clone(), rows);
    for command in placed(&view) {
        let expected = [UiIntent::Command(command.id)];
        assert_eq!(
            click(&mut harness, &command.label),
            expected,
            "{} click",
            command.label
        );
        assert_eq!(
            press_enter_on(&mut harness, &command.label),
            expected,
            "{} Enter",
            command.label
        );
    }
}

#[test]
fn h2_disabled_placed_commands_should_emit_nothing() {
    let view = fixtures::sample();
    let mut harness = light(view.clone(), rows);
    for command in placed(&view).into_iter().filter(|c| !c.is_enabled()) {
        assert!(
            click(&mut harness, &command.label).is_empty(),
            "{}",
            command.label
        );
    }
}

fn choose_from_menu(
    harness: &mut Harness<'_, Recorder<Presentation>>,
    group: &str,
    label: &str,
) -> Vec<UiIntent> {
    // A command with a row button shares its label with its menu item; the
    // menu item is the node that was not there before the menu opened.
    let before: Vec<_> = harness
        .query_all_by_label(label)
        .map(|node| node.accesskit_node().bounding_box())
        .collect();
    harness.state_mut().take();
    harness.get_by_label(header::PROJECT_MENU).click();
    harness.run();
    // Submenus open on hover.
    harness.get_by_label(group).hover();
    harness.run();
    // A menu item's accessible name also carries its shortcut text.
    if let Some(item) = harness
        .get_all_by(|node| {
            node.data()
                .label()
                .is_some_and(|name| name == label || name.starts_with(&format!("{label} ")))
        })
        .find(|node| !before.contains(&node.accesskit_node().bounding_box()))
    {
        item.click();
    }
    common_ui::intents_after(harness)
}

#[test]
fn h1_h2_every_catalog_command_should_be_in_the_project_menu_and_emit_once() {
    let view = all_enabled(fixtures::sample());
    for id in CommandId::ALL {
        let found = view.header.menu.iter().find_map(|group| {
            group
                .commands
                .iter()
                .find(|command| command.id == id)
                .map(|command| (group.label, command.label.clone()))
        });
        assert!(found.is_some(), "{id:?} is missing from the Project menu");
        let Some((group, label)) = found else {
            continue;
        };
        let mut harness = light(view.clone(), rows);
        assert_eq!(
            choose_from_menu(&mut harness, group, &label),
            [UiIntent::Command(id)],
            "{id:?}"
        );
    }
}

#[test]
fn disabled_menu_items_should_say_unavailable_and_give_the_reason_on_hover() {
    let view = fixtures::sample();
    let mut harness = light(view.clone(), rows);
    let group = view
        .header
        .menu
        .iter()
        .find(|group| group.commands.iter().any(|command| !command.is_enabled()))
        .cloned();
    assert!(group.is_some(), "the sample has disabled commands");
    let Some(group) = group else {
        return;
    };
    harness.get_by_label(header::PROJECT_MENU).click();
    harness.run();
    harness.get_by_label(group.label).hover();
    harness.run();
    let disabled: Vec<_> = group
        .commands
        .iter()
        .filter(|command| !command.is_enabled())
        .collect();
    for command in &disabled {
        // The row's accessible name carries the visible "Unavailable".
        let name = format!("{} {}", command.label, components::UNAVAILABLE);
        assert!(
            harness.query_by_label(&name).is_some(),
            "{}: no visible Unavailable",
            command.label
        );
    }
    let first = disabled[0];
    let reason = first.availability.reason().unwrap_or_default();
    harness
        .get_by_label(&format!("{} {}", first.label, components::UNAVAILABLE))
        .hover();
    harness.run();
    assert!(
        harness.query_by_label_contains(reason).is_some(),
        "{}: reason not shown on hover",
        first.label
    );
}

fn panel_and_inspector(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_side_panel(ui, 240.0, |ui| navigator::show(ui, &view.navigator, out));
    egui::CentralPanel::default().show(ui, |ui| inspector::show(ui, &view.inspector, out));
}

#[test]
fn navigator_row_should_emit_select_and_shift_should_toggle() {
    let mut harness = light(fixtures::sample(), panel_and_inspector);

    assert_eq!(
        click(&mut harness, "Order"),
        [UiIntent::Select {
            target: Target::Class(1),
            toggle: false
        }]
    );
    harness.state_mut().take();
    harness
        .get_by_label("Customer")
        .click_modifiers(Modifiers::SHIFT);
    assert_eq!(
        common_ui::intents_after(&mut harness),
        [UiIntent::Select {
            target: Target::Class(0),
            toggle: true
        }]
    );
}

#[test]
fn name_field_should_emit_draft_changed_commit_and_cancel() {
    let mut harness = light(fixtures::class_selected(), panel_and_inspector);

    harness.get_by_label("Name").focus();
    harness.run();
    harness.state_mut().take();
    harness.get_by_label("Name").type_text("X");
    let typed = common_ui::intents_after(&mut harness);
    assert_eq!(
        typed,
        [UiIntent::DraftChanged {
            field: DraftField::ClassName,
            text: String::from("CustomerX")
        }]
    );

    harness.key_press(Key::Enter);
    assert_eq!(
        common_ui::intents_after(&mut harness),
        [UiIntent::CommitDraft(DraftField::ClassName)]
    );

    harness.get_by_label("Name").focus();
    harness.run();
    harness.state_mut().take();
    harness.key_press(Key::Escape);
    assert_eq!(
        common_ui::intents_after(&mut harness),
        [UiIntent::CancelDraft(DraftField::ClassName)]
    );
}

fn board(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show(ui, |ui| canvas_view::show(ui, &view.canvas, out));
}

#[test]
fn board_node_click_should_select_and_empty_click_should_report_pointer() {
    let mut harness = light(fixtures::sample(), board);

    assert_eq!(
        click(&mut harness, "Class Order"),
        [UiIntent::Select {
            target: Target::Class(1),
            toggle: false
        }]
    );

    harness.state_mut().take();
    harness.get_by_label("Diagram canvas").click();
    let intents = common_ui::intents_after(&mut harness);
    let kinds: Vec<_> = intents
        .iter()
        .filter_map(|intent| match intent {
            UiIntent::Pointer(pointer) => Some((pointer.kind, pointer.hit)),
            _ => None,
        })
        .collect();
    assert_eq!(
        kinds.len(),
        intents.len(),
        "only pointer intents: {intents:?}"
    );
    assert_eq!(kinds, [(PointerKind::Down, None), (PointerKind::Up, None)]);
}

#[test]
fn board_drag_should_report_down_at_the_press_position() {
    // Regression (A-req-B2): Down used to be reported where the pointer was
    // when egui recognized the drag, past the press, so a Hand pan lost
    // its first step.
    let mut harness = light(fixtures::sample(), board);
    harness.state_mut().take();
    let start = egui::Pos2::new(700.0, 500.0);
    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    for step in 1..=3 {
        harness.hover_at(start + Vec2::new(20.0 * step as f32, 10.0));
        harness.run();
    }
    harness.drop_at(start + Vec2::new(60.0, 30.0));
    let down = common_ui::intents_after(&mut harness)
        .into_iter()
        .find_map(|intent| match intent {
            UiIntent::Pointer(pointer) if pointer.kind == PointerKind::Down => {
                Some(pointer.position)
            }
            _ => None,
        });
    assert_eq!(down, Some(start));
}

#[test]
fn board_drag_should_report_down_moves_and_up_once() {
    let mut harness = light(fixtures::sample(), board);
    harness.state_mut().take();
    let start = egui::Pos2::new(700.0, 500.0);
    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    for step in 1..=3 {
        harness.hover_at(start + Vec2::new(20.0 * step as f32, 10.0));
        harness.run();
    }
    harness.drop_at(start + Vec2::new(60.0, 30.0));
    let kinds: Vec<_> = common_ui::intents_after(&mut harness)
        .into_iter()
        .filter_map(|intent| match intent {
            UiIntent::Pointer(pointer) => Some(pointer.kind),
            _ => None,
        })
        .collect();
    assert_eq!(kinds.first(), Some(&PointerKind::Down), "{kinds:?}");
    assert_eq!(kinds.last(), Some(&PointerKind::Up), "{kinds:?}");
    assert!(kinds.contains(&PointerKind::Move), "{kinds:?}");
    assert_eq!(kinds.iter().filter(|k| **k == PointerKind::Down).count(), 1);
    assert_eq!(kinds.iter().filter(|k| **k == PointerKind::Up).count(), 1);
}

fn status_bar(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    egui::Panel::bottom("status-under-test")
        .show(ui, |ui| status::show(ui, &view.status, true, out));
}

#[test]
fn theme_choice_should_emit_set_theme() {
    let mut harness = light(fixtures::sample(), status_bar);
    for (preference, label) in status::THEME_CHOICES {
        assert_eq!(
            click(&mut harness, label),
            [UiIntent::SetTheme(preference)],
            "{label}"
        );
    }
}

#[test]
fn clicking_another_tab_should_emit_activate_tab_and_the_active_one_nothing() {
    let view = fixtures::many_tabs();
    let mut harness = harness(common_ui::WIDE, ThemePreference::Light, view.clone(), rows);
    let active = view
        .tabs
        .tabs
        .iter()
        .position(|tab| tab.active)
        .unwrap_or(0);
    let other = if active == 0 { 1 } else { 0 };

    assert_eq!(
        click(&mut harness, &view.tabs.tabs[other].label),
        [UiIntent::ActivateTab(other)]
    );
    assert!(click(&mut harness, &view.tabs.tabs[active].label).is_empty());
}

#[test]
fn board_should_report_its_size_once_per_change() {
    let mut harness = light(fixtures::sample(), board);
    let resized: Vec<_> = harness
        .state_mut()
        .take()
        .into_iter()
        .filter(|intent| matches!(intent, UiIntent::CanvasResized { .. }))
        .collect();
    assert_eq!(resized.len(), 1, "{resized:?}");
    harness.run();
    assert!(
        !harness
            .state_mut()
            .take()
            .iter()
            .any(|intent| matches!(intent, UiIntent::CanvasResized { .. })),
        "size sent again without a change"
    );
    harness.set_size(Vec2::new(1000.0, 700.0));
    harness.run();
    assert!(
        harness
            .state_mut()
            .take()
            .iter()
            .any(|intent| matches!(intent, UiIntent::CanvasResized { .. })),
        "new size not reported"
    );
}

#[test]
fn space_over_the_board_should_toggle_temporary_pan() {
    let mut harness = light(fixtures::sample(), board);
    harness.hover_at(egui::Pos2::new(700.0, 500.0));
    harness.run();
    harness.state_mut().take();

    harness.key_down(Key::Space);
    let down = common_ui::intents_after(&mut harness);
    harness.key_up(Key::Space);
    let up = common_ui::intents_after(&mut harness);

    assert!(down.contains(&UiIntent::TemporaryPan(true)), "{down:?}");
    assert!(up.contains(&UiIntent::TemporaryPan(false)), "{up:?}");
}
