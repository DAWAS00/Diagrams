//! S5: the Class tool. Clicking the canvas places a class and names it in one
//! step; the pure placement and naming rules are tested directly.

use diagram_core::command::Command;
use diagram_core::ids::{DiagramId, ElementId};
use diagram_desktop::session::ProjectSession;
use diagram_desktop::shell::ShellState;
use diagram_desktop::ui::class_tool::{
    self, CLASS_SIZE, CanvasPoint, CanvasRect, CanvasSize, Decision, NEW_CLASS_LABEL, Tool,
};
use diagram_desktop::ui::commands::CommandId;
use diagram_desktop::ui::intents::UiIntent;
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

mod common;
use common::{choose_class_tool, class_id, click_canvas_at, diagram_id, harness_with};

/// A point on the canvas that no sample class covers.
const EMPTY: (f32, f32) = (100.0, 300.0);

fn pressed(harness: &Harness<'_, ShellState>, label: &str) -> bool {
    harness.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True)
}

fn classes_named(harness: &Harness<'_, ShellState>, name: &str) -> Vec<ElementId> {
    harness
        .state()
        .document()
        .elements
        .values()
        .filter(|element| element.name.as_str() == name)
        .map(|element| element.id)
        .collect()
}

fn element_count(harness: &Harness<'_, ShellState>) -> usize {
    harness.state().document().elements.len()
}

fn field_value(harness: &Harness<'_, ShellState>) -> Option<String> {
    harness.get_by_label(NEW_CLASS_LABEL).value()
}

fn field_focused(harness: &Harness<'_, ShellState>) -> bool {
    harness.get_by_label(NEW_CLASS_LABEL).is_focused()
}

fn has_field(harness: &Harness<'_, ShellState>) -> bool {
    harness.query_by_label(NEW_CLASS_LABEL).is_some()
}

fn undo(harness: &mut Harness<'static, ShellState>) {
    harness.state_mut().queue(UiIntent::Run {
        command: CommandId::Undo,
        origin: None,
    });
    harness.run();
}

/// The sample with the Class tool chosen and the canvas clicked at `EMPTY`.
fn naming() -> Harness<'static, ShellState> {
    naming_in(ShellState::default())
}

fn naming_in(state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = harness_with(common::LARGE, state);
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, EMPTY.0, EMPTY.1);
    harness
}

fn placement_id(harness: &Harness<'_, ShellState>) -> Option<class_tool::PlacementId> {
    harness.state().placement().map(|placement| placement.id)
}

fn view_origin(
    harness: &Harness<'_, ShellState>,
    diagram: DiagramId,
    element: ElementId,
) -> Option<(f64, f64)> {
    let node = harness
        .state()
        .document()
        .diagrams
        .get(&diagram)?
        .node_of(element)?;
    Some((node.bounds.left(), node.bounds.top()))
}

// --- Pure rules -----------------------------------------------------------

#[test]
fn default_class_name_should_use_the_smallest_free_number() {
    let mut session = ProjectSession::sample();
    assert_eq!(
        class_tool::default_class_name(session.document()),
        "Class 1"
    );
    let diagram = session.document().diagram_order[0];
    for name in ["Class 1", " Class 3 "] {
        common::run(
            &mut session,
            Command::CreateClass {
                diagram,
                name: name.to_owned(),
                bounds: common::bounds(600.0, 600.0),
            },
        );
    }
    assert_eq!(
        class_tool::default_class_name(session.document()),
        "Class 2"
    );
}

#[test]
fn next_free_spot_should_take_the_first_free_grid_cell() {
    let visible = CanvasSize::new(800, 600);
    let first = CanvasPoint::new(48, 48);
    assert_eq!(class_tool::next_free_spot(&[], CLASS_SIZE, visible), first);

    // The first cell taken: the next one along the row.
    let taken = CanvasRect {
        at: first,
        size: CanvasSize::new(180, 120),
    };
    let second = CanvasPoint::new(48 + CLASS_SIZE.width + 40, 48);
    assert_eq!(
        class_tool::next_free_spot(&[taken], CLASS_SIZE, visible),
        second
    );

    // A tall class with members covers the first cell of the next row too.
    let tall = CanvasRect {
        at: first,
        size: CanvasSize::new(CLASS_SIZE.width, 400),
    };
    let row = CanvasRect {
        at: second,
        size: CanvasSize::new(800, CLASS_SIZE.height),
    };
    let below = class_tool::next_free_spot(&[tall, row], CLASS_SIZE, visible);
    assert_eq!(below.y, 48 + CLASS_SIZE.height + 40);
    assert_ne!(below.x, 48);

    // Everything visible taken: the clamped first cell.
    let all = CanvasRect {
        at: CanvasPoint::new(0, 0),
        size: visible,
    };
    assert_eq!(
        class_tool::next_free_spot(&[all], CLASS_SIZE, visible),
        first
    );
    // A canvas too small for even one cell keeps the box inside.
    let tiny = CanvasSize::new(100, 40);
    assert_eq!(
        class_tool::next_free_spot(&[], CLASS_SIZE, tiny),
        CanvasPoint::new(0, 0)
    );
}

#[test]
fn fit_inside_should_keep_the_box_on_the_canvas() {
    let visible = CanvasSize::new(500, 400);
    let size = CanvasSize::new(180, 60);
    assert_eq!(
        class_tool::fit_inside(CanvasPoint::new(10, 20), size, visible),
        CanvasPoint::new(10, 20)
    );
    assert_eq!(
        class_tool::fit_inside(CanvasPoint::new(490, 390), size, visible),
        CanvasPoint::new(320, 340)
    );
    assert_eq!(
        class_tool::fit_inside(CanvasPoint::new(-5, -5), size, visible),
        CanvasPoint::new(0, 0)
    );
    // Larger than the canvas: pinned to the top-left, never negative.
    assert_eq!(
        class_tool::fit_inside(CanvasPoint::new(50, 50), size, CanvasSize::new(100, 30)),
        CanvasPoint::new(0, 0)
    );
}

#[test]
fn decide_should_ask_only_when_the_trimmed_name_exists() {
    let state = ShellState::default();
    let document = state.document();
    let first = diagram_id(&state, "Class diagram 1");
    let details = diagram_id(&state, "Order details");
    let customer = class_id(&state, "Customer");
    let order = class_id(&state, "Order");

    assert_eq!(
        class_tool::decide(document, first, "  Invoice "),
        Decision::Create(String::from("Invoice"))
    );
    // Case-sensitive: a different case is a different name.
    assert_eq!(
        class_tool::decide(document, details, "customer"),
        Decision::Create(String::from("customer"))
    );
    // A match shown elsewhere can be shown here.
    assert_eq!(
        class_tool::decide(document, details, " Customer "),
        Decision::Ask {
            name: String::from("Customer"),
            existing: Some(customer),
        }
    );
    // A match already here cannot.
    assert_eq!(
        class_tool::decide(document, details, "Order"),
        Decision::Ask {
            name: String::from("Order"),
            existing: None,
        }
    );
    assert_eq!(
        class_tool::decide(document, first, "Order"),
        Decision::Ask {
            name: String::from("Order"),
            existing: None,
        }
    );
    let _ = order;
}

#[test]
fn decide_should_offer_the_lowest_id_not_already_shown() {
    let mut session = ProjectSession::sample();
    let first = session.document().diagram_order[0];
    let second = session.document().diagram_order[1];
    for diagram in [first, second] {
        common::run(
            &mut session,
            Command::CreateClass {
                diagram,
                name: String::from("Line"),
                bounds: common::bounds(600.0, 600.0),
            },
        );
    }
    let mut lines: Vec<ElementId> = session
        .document()
        .elements
        .values()
        .filter(|element| element.name.as_str() == "Line")
        .map(|element| element.id)
        .collect();
    lines.sort_unstable();
    assert_eq!(lines.len(), 2);
    // From the first diagram, the second Line is the one not shown there.
    assert_eq!(
        class_tool::decide(session.document(), first, "Line"),
        Decision::Ask {
            name: String::from("Line"),
            existing: lines.get(1).copied(),
        }
    );
    assert_eq!(
        class_tool::decide(session.document(), second, "Line"),
        Decision::Ask {
            name: String::from("Line"),
            existing: lines.first().copied(),
        }
    );
}

#[test]
fn bounds_should_place_the_box_at_its_point_and_size() {
    let rect = class_tool::bounds(CanvasPoint::new(12, 34), CanvasSize::new(180, 60));
    assert_eq!(
        (rect.left(), rect.top()),
        (12.0, 34.0),
        "top-left at the point"
    );
    assert_eq!((rect.size().width(), rect.size().height()), (180.0, 60.0));
}

#[test]
fn new_class_field_should_have_one_fixed_id() {
    assert_eq!(class_tool::new_class_field(), class_tool::new_class_field());
    assert_ne!(class_tool::new_class_field(), eframe::egui::Id::NULL);
}

#[test]
fn canvas_rects_should_overlap_only_when_they_share_area() {
    let a = CanvasRect {
        at: CanvasPoint::new(0, 0),
        size: CanvasSize::new(10, 10),
    };
    let touching = CanvasRect {
        at: CanvasPoint::new(10, 0),
        size: CanvasSize::new(10, 10),
    };
    let inside = CanvasRect {
        at: CanvasPoint::new(5, 5),
        size: CanvasSize::new(10, 10),
    };
    assert!(!a.overlaps(&touching));
    assert!(a.overlaps(&inside));
    assert_eq!(
        CanvasSize::of(eframe::egui::Vec2::new(10.2, 3.0)),
        CanvasSize::new(11, 3)
    );
}

// --- The tool ------------------------------------------------------------

#[test]
fn class_should_be_available_with_a_diagram_and_show_as_pressed() {
    let mut harness = harness_with(common::LARGE, ShellState::default());
    assert!(pressed(&harness, "Select"));
    assert!(!pressed(&harness, "Class"));

    choose_class_tool(&mut harness);

    assert_eq!(harness.state().active_tool(), Tool::Class);
    assert!(pressed(&harness, "Class"));
    assert!(!pressed(&harness, "Select"));
    harness.get_by_label_contains("click the canvas");
}

#[test]
fn class_should_be_disabled_with_its_reason_when_no_diagram_is_open() {
    let mut state = ShellState::default();
    for diagram in state.document().diagram_order.clone() {
        state.close_tab(diagram);
    }
    let mut harness = harness_with(common::LARGE, state);

    assert!(harness.get_by_label("Class").accesskit_node().is_disabled());
    harness.get_by_label("Class").hover();
    harness.run();
    harness.get_by_label_contains("Open a diagram first");
}

#[test]
fn clicking_the_canvas_should_place_and_name_a_class_in_one_step() {
    let mut harness = naming();
    let diagram = diagram_id(harness.state(), "Class diagram 1");
    let before = element_count(&harness);

    assert!(field_focused(&harness), "the name field has focus");
    assert_eq!(field_value(&harness).as_deref(), Some("Class 1"));
    assert_eq!(harness.state().active_tool(), Tool::Select);

    // The whole default name is selected, so typing replaces it.
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();
    assert_eq!(field_value(&harness).as_deref(), Some("Invoice"));
    harness.key_press(Key::Enter);
    harness.run();

    let invoice = classes_named(&harness, "Invoice");
    assert_eq!(invoice.len(), 1);
    assert_eq!(element_count(&harness), before + 1);
    let Some(&invoice) = invoice.first() else {
        return;
    };
    assert_eq!(
        view_origin(&harness, diagram, invoice),
        Some((f64::from(EMPTY.0), f64::from(EMPTY.1))),
        "the view is at the click point"
    );
    assert_eq!(harness.state().selected_element(), Some(invoice));
    assert_eq!(harness.state().active_tool(), Tool::Select);
    assert!(harness.state().placement().is_none());
    assert!(!has_field(&harness));

    undo(&mut harness);
    assert!(classes_named(&harness, "Invoice").is_empty());
    assert_eq!(element_count(&harness), before);
}

#[test]
fn enter_with_the_default_name_should_create_class_1() {
    let mut harness = naming();
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(classes_named(&harness, "Class 1").len(), 1);
}

#[test]
fn a_taken_default_name_should_give_the_next_number() {
    let mut harness = naming();
    harness.key_press(Key::Enter);
    harness.run();
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, 100.0, 420.0);

    assert_eq!(field_value(&harness).as_deref(), Some("Class 2"));
}

#[test]
fn escape_while_naming_should_create_nothing() {
    let mut harness = naming();
    let before = harness.state().document().clone();

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().document(), &before);
    assert!(harness.state().placement().is_none());
    assert!(!has_field(&harness));
    assert!(!harness.state().command_context().can_undo);
}

#[test]
fn moving_focus_away_should_commit_the_name() {
    let mut harness = naming();
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();

    // Undo is disabled here, so focus moves to an enabled control.
    harness.get_by_label("Class Customer").focus();
    harness.run();

    assert_eq!(classes_named(&harness, "Invoice").len(), 1);
}

#[test]
fn escape_should_return_the_class_tool_to_select_and_keep_the_selection() {
    let mut harness = harness_with(common::LARGE, ShellState::default());
    common::select_class(&mut harness, "Customer");
    harness.run();
    choose_class_tool(&mut harness);
    let customer = class_id(harness.state(), "Customer");

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().active_tool(), Tool::Select);
    assert_eq!(harness.state().selected_element(), Some(customer));
    assert!(pressed(&harness, "Select"));

    // The next Escape clears the selection as before.
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().selected_element(), None);
}

#[test]
fn a_blank_name_should_keep_the_error_and_the_focus() {
    let mut harness = naming();
    let before = element_count(&harness);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.key_press(Key::Backspace);
    harness.run();
    assert!(harness.query_all_by_label_contains("Error:").count() > 0);

    // Enter, a blur and a canvas click each keep the field and its error.
    harness.key_press(Key::Enter);
    harness.run();
    assert!(has_field(&harness) && field_focused(&harness));
    harness.get_by_label("Class Customer").focus();
    harness.run();
    harness.run();
    assert!(has_field(&harness) && field_focused(&harness));
    click_canvas_at(&mut harness, 600.0, 500.0);
    assert!(has_field(&harness) && field_focused(&harness));
    assert!(harness.query_all_by_label_contains("Error:").count() > 0);
    assert_eq!(element_count(&harness), before);

    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(classes_named(&harness, "Invoice").len(), 1);
}

#[test]
fn text_undo_inside_the_field_should_not_undo_the_document() {
    // One earlier change, so the document could be undone.
    let mut harness = naming();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(classes_named(&harness, "Class 1").len(), 1);
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, 100.0, 420.0);

    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();

    assert_eq!(
        classes_named(&harness, "Class 1").len(),
        1,
        "no document undo"
    );
    assert!(has_field(&harness));
}

#[test]
fn ctrl_z_and_escape_before_the_panel_renders_should_not_touch_the_history() {
    let mut harness = harness_with(common::LARGE, ShellState::default());
    let diagram = diagram_id(harness.state(), "Class diagram 1");
    // A first class, so there is something to undo.
    harness.state_mut().queue(UiIntent::Run {
        command: CommandId::AddClass,
        origin: None,
    });
    harness.run();
    let begin = UiIntent::BeginNewClass {
        diagram,
        at: CanvasPoint::new(100, 300),
        visible: CanvasSize::new(600, 500),
    };
    harness.state_mut().queue(begin.clone());
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(classes_named(&harness, "Class 1").len(), 1);

    harness.state_mut().queue(UiIntent::Run {
        command: CommandId::AddClass,
        origin: None,
    });
    harness.run();
    harness.state_mut().queue(begin);
    harness.step();
    // Both keys arrive in the very next frame, before the field has focus.
    // (`key_press` would spread them over several frames.)
    for (key, modifiers) in [(Key::Z, Modifiers::COMMAND), (Key::Escape, Modifiers::NONE)] {
        harness.input_mut().events.push(eframe::egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
    }
    harness.step();
    harness.run();

    assert_eq!(classes_named(&harness, "Class 1").len(), 1, "no undo ran");
    assert!(harness.state().placement().is_none(), "Escape cancelled");
}

#[test]
fn the_keyboard_should_place_a_class_at_a_free_spot() -> Result<(), String> {
    let mut harness = harness_with(common::LARGE, ShellState::default());
    let diagram = diagram_id(harness.state(), "Class diagram 1");
    choose_class_tool(&mut harness);
    harness.remove_cursor();
    harness.run();

    let mut reached = false;
    for _ in 0..80 {
        harness.key_press(Key::Tab);
        harness.run();
        if harness.get_by_label("Diagram canvas").is_focused() {
            reached = true;
            break;
        }
    }
    assert!(reached, "Tab reaches the canvas");
    harness.key_press(Key::Enter);
    harness.run();
    assert!(has_field(&harness));
    harness.key_press(Key::Enter);
    harness.run();

    let Some(&created) = classes_named(&harness, "Class 1").first() else {
        return Err(String::from("a class was created"));
    };
    let Some((x, y)) = view_origin(&harness, diagram, created) else {
        return Ok(());
    };
    // Not over Customer (48, 48, 180×80) or Order (300, 96, 180×80).
    let new = CanvasRect {
        at: CanvasPoint::new(x as i32, y as i32),
        size: CLASS_SIZE,
    };
    for (left, top) in [(48, 48), (300, 96)] {
        let taken = CanvasRect {
            at: CanvasPoint::new(left, top),
            size: CanvasSize::new(180, 140),
        };
        assert!(!new.overlaps(&taken), "{new:?} overlaps {taken:?}");
    }
    Ok(())
}

// --- Requests while placing ----------------------------------------------

#[test]
fn repeated_and_stale_commits_should_create_one_class() -> Result<(), String> {
    let mut harness = naming();
    let Some(id) = placement_id(&harness) else {
        return Err(String::from("a placement exists"));
    };
    let before = element_count(&harness);
    let commit = |name: &str| UiIntent::CommitNewClass {
        placement: id,
        name: name.to_owned(),
    };
    harness.state_mut().queue(commit("Invoice"));
    harness.state_mut().queue(commit("Receipt"));
    harness.run();
    assert_eq!(element_count(&harness), before + 1);
    assert_eq!(classes_named(&harness, "Invoice").len(), 1);

    // The old ID after the placement ended does nothing.
    harness.state_mut().queue(commit("Late"));
    harness.run();
    assert_eq!(element_count(&harness), before + 1);

    // A commit after Cancel does nothing.
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, 100.0, 420.0);
    let Some(second) = placement_id(&harness) else {
        return Ok(());
    };
    assert_ne!(second, id);
    // The first placement's ID does nothing to the live one.
    harness.state_mut().queue(commit("Stale"));
    harness.run();
    assert!(classes_named(&harness, "Stale").is_empty());
    assert_eq!(placement_id(&harness), Some(second), "still naming");
    harness.state_mut().queue(UiIntent::CancelNewClass(second));
    harness.state_mut().queue(UiIntent::CommitNewClass {
        placement: second,
        name: String::from("Ghost"),
    });
    harness.run();
    assert!(classes_named(&harness, "Ghost").is_empty());
    Ok(())
}

#[test]
fn begin_while_placing_should_change_nothing() {
    let mut harness = naming();
    let diagram = diagram_id(harness.state(), "Class diagram 1");
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();
    let placement = harness.state().placement().copied();
    let document = harness.state().document().clone();

    harness.state_mut().queue(UiIntent::BeginNewClass {
        diagram,
        at: CanvasPoint::new(400, 400),
        visible: CanvasSize::new(600, 500),
    });
    harness.run();

    assert_eq!(harness.state().placement().copied(), placement);
    assert_eq!(field_value(&harness).as_deref(), Some("Invoice"));
    assert_eq!(harness.state().document(), &document);
}

#[test]
fn undo_or_redo_in_a_batch_should_cancel_the_placement_and_skip_the_commit() {
    for history in [CommandId::Undo, CommandId::Redo] {
        for commit_first in [true, false] {
            let mut harness = naming();
            let Some(id) = placement_id(&harness) else {
                return;
            };
            let document = harness.state().document().clone();
            let commit = UiIntent::CommitNewClass {
                placement: id,
                name: String::from("Invoice"),
            };
            let run = UiIntent::Run {
                command: history,
                origin: None,
            };
            let batch = if commit_first {
                [commit, run]
            } else {
                [run, commit]
            };
            for intent in batch {
                harness.state_mut().queue(intent);
            }
            harness.run();

            assert!(harness.state().placement().is_none(), "{history:?} cancels");
            assert_eq!(
                harness.state().document(),
                &document,
                "{history:?} {commit_first}: nothing to undo or redo, no commit"
            );
        }
    }
}

#[test]
fn switching_tabs_while_naming_should_create_the_class_first() -> Result<(), String> {
    let mut state = ShellState::default();
    let first = diagram_id(&state, "Class diagram 1");
    let details = diagram_id(&state, "Order details");
    state.open_diagram(details);
    state.activate_tab(first);
    let mut harness = naming_in(state);
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();

    harness.state_mut().queue(UiIntent::ActivateTab(details));
    harness.step();

    let Some(&invoice) = classes_named(&harness, "Invoice").first() else {
        return Err(String::from("Invoice was created"));
    };
    assert!(view_origin(&harness, first, invoice).is_some());
    assert_eq!(harness.state().active_diagram(), Some(details));
    let tab = harness
        .state()
        .tabs()
        .iter()
        .find(|tab| tab.diagram == first)
        .and_then(|tab| tab.selected);
    assert_eq!(tab, Some(invoice), "selected in the first diagram");
    Ok(())
}

#[test]
fn ctrl_k_while_naming_should_create_the_class_then_open_search() {
    let mut harness = naming();
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();

    assert_eq!(classes_named(&harness, "Invoice").len(), 1);
    assert!(harness.state().palette.is_some(), "search opened");
}

#[test]
fn close_project_while_naming_should_create_the_class_then_ask() {
    let mut harness = naming();
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();

    harness.state_mut().queue(UiIntent::Run {
        command: CommandId::CloseProject,
        origin: None,
    });
    harness.run();

    assert_eq!(classes_named(&harness, "Invoice").len(), 1);
    harness.get_by_label("Close project?");
}

#[test]
fn a_blank_name_should_block_other_requests() {
    let mut state = ShellState::default();
    let first = diagram_id(&state, "Class diagram 1");
    let details = diagram_id(&state, "Order details");
    state.open_diagram(details);
    state.activate_tab(first);
    let requests = [
        UiIntent::ActivateTab(details),
        UiIntent::CloseTab(first),
        UiIntent::Run {
            command: CommandId::CloseProject,
            origin: None,
        },
        UiIntent::Run {
            command: CommandId::SearchCommands,
            origin: None,
        },
    ];
    for request in requests {
        let mut harness = naming_in(state.clone());
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness.key_press(Key::Backspace);
        harness.run();
        let placement = harness.state().placement().copied();
        let document = harness.state().document().clone();

        harness.state_mut().queue(request.clone());
        harness.run();

        assert_eq!(
            harness.state().placement().copied(),
            placement,
            "{request:?}"
        );
        assert_eq!(harness.state().document(), &document, "{request:?}");
        assert_eq!(harness.state().active_diagram(), Some(first), "{request:?}");
        assert_eq!(harness.state().tabs().len(), 2, "{request:?}");
        assert!(!harness.state().has_dialog(), "{request:?}");
        assert!(harness.state().palette.is_none(), "{request:?}");
        assert!(field_focused(&harness), "{request:?}");
        assert!(harness.query_all_by_label_contains("Error:").count() > 0);
    }
}

#[test]
fn clicking_another_class_while_naming_should_create_then_select_it() {
    let mut harness = naming();
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();
    let order = class_id(harness.state(), "Order");

    harness.get_by_label("Class Order").click();
    harness.run();

    assert_eq!(classes_named(&harness, "Invoice").len(), 1);
    assert_eq!(harness.state().selected_element(), Some(order));
}

#[test]
fn a_duplicate_commit_with_navigation_should_ask_and_stay() {
    for commit_first in [true, false] {
        let mut state = ShellState::default();
        let first = diagram_id(&state, "Class diagram 1");
        let details = diagram_id(&state, "Order details");
        state.open_diagram(details);
        state.activate_tab(first);
        let mut harness = naming_in(state);
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness.get_by_label(NEW_CLASS_LABEL).type_text("Order");
        harness.run();
        let Some(id) = placement_id(&harness) else {
            return;
        };
        let commit = UiIntent::CommitNewClass {
            placement: id,
            name: String::from("Order"),
        };
        let switch = UiIntent::ActivateTab(details);
        let batch = if commit_first {
            [commit, switch]
        } else {
            [switch, commit]
        };
        for intent in batch {
            harness.state_mut().queue(intent);
        }
        harness.run();

        assert!(harness.state().has_dialog(), "{commit_first}");
        assert_eq!(
            harness.state().active_diagram(),
            Some(first),
            "{commit_first}"
        );
        harness.get_by_label("A class with this name exists");
    }
}

#[test]
fn escape_while_the_dialog_asks_should_close_only_the_dialog() {
    let mut harness = naming();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Order");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.state().has_dialog());

    harness.key_press(Key::Escape);
    harness.run();

    assert!(!harness.state().has_dialog());
    assert!(harness.state().placement().is_some(), "still naming");
    assert_eq!(field_value(&harness).as_deref(), Some("Order"));
    assert!(field_focused(&harness));
}

// --- Geometry --------------------------------------------------------------

fn inside(outer: eframe::egui::Rect, inner: eframe::egui::Rect) -> bool {
    outer.expand(0.5).contains_rect(inner)
}

#[test]
fn a_click_near_the_edge_should_keep_the_box_and_panel_inside() {
    let mut harness = harness_with(common::LARGE, ShellState::default());
    let diagram = diagram_id(harness.state(), "Class diagram 1");
    choose_class_tool(&mut harness);
    let canvas = harness.get_by_label("Diagram canvas").rect();
    click_canvas_at(&mut harness, canvas.width() - 4.0, canvas.height() - 4.0);

    let field = harness.get_by_label(NEW_CLASS_LABEL).rect();
    assert!(inside(canvas, field), "{field:?} inside {canvas:?}");

    // The error state keeps the same panel, still inside.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.key_press(Key::Backspace);
    harness.run();
    let with_error = harness.get_by_label(NEW_CLASS_LABEL).rect();
    assert_eq!(with_error, field, "the panel does not move");
    let error = harness.get_by_label_contains("Error:").rect();
    assert!(inside(canvas, error), "{error:?} inside {canvas:?}");

    harness.get_by_label(NEW_CLASS_LABEL).type_text("Invoice");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    let Some(&invoice) = classes_named(&harness, "Invoice").first() else {
        return;
    };
    let Some((x, y)) = view_origin(&harness, diagram, invoice) else {
        return;
    };
    assert!(x + f64::from(CLASS_SIZE.width) <= f64::from(canvas.width()) + 0.5);
    assert!(y + f64::from(CLASS_SIZE.height) <= f64::from(canvas.height()) + 0.5);
}

#[test]
fn a_long_name_should_stay_on_one_line_in_the_field() {
    let mut harness = naming();
    let single = harness.get_by_label(NEW_CLASS_LABEL).rect().height();
    harness
        .get_by_label(NEW_CLASS_LABEL)
        .type_text("AVeryLongClassNameThatIsWiderThanTheNamingPanelItself");
    harness.run();

    assert_eq!(
        harness.get_by_label(NEW_CLASS_LABEL).rect().height(),
        single
    );
}
