//! S5, decision D2: a new class given an existing name asks what to do.
//! "Show the existing class here" shares the class; "Keep a new class" makes
//! a separate one; Cancel goes back to the name field.

use diagram_core::ids::{DiagramId, ElementId};
use diagram_desktop::shell::ShellState;
use diagram_desktop::ui::class_tool::{NEW_CLASS_LABEL, Phase};
use diagram_desktop::ui::commands::CommandId;
use diagram_desktop::ui::dialogs::{self, DialogKind};
use diagram_desktop::ui::intents::UiIntent;
use eframe::egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

mod common;
use common::{choose_class_tool, class_id, click_canvas_at, diagram_id, harness_with};

const TITLE: &str = "A class with this name exists";
const SHOW_HERE: &str = "Show the existing class here";
const KEEP_NEW: &str = "Keep a new class";

/// The sample with `diagram` active, the Class tool used at (`x`, `y`) and
/// `name` typed and committed with Enter.
fn committed_in(diagram: &str, at: (f32, f32), name: &str) -> Harness<'static, ShellState> {
    let mut state = ShellState::default();
    let target = diagram_id(&state, diagram);
    state.open_diagram(target);
    let mut harness = harness_with(common::LARGE, state);
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, at.0, at.1);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label(NEW_CLASS_LABEL).type_text(name);
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    harness
}

fn asking(name: &str) -> Harness<'static, ShellState> {
    committed_in("Order details", (100.0, 300.0), name)
}

fn named(harness: &Harness<'_, ShellState>, name: &str) -> Vec<ElementId> {
    harness
        .state()
        .document()
        .elements
        .values()
        .filter(|element| element.name.as_str() == name)
        .map(|element| element.id)
        .collect()
}

fn view(
    harness: &Harness<'_, ShellState>,
    diagram: DiagramId,
    element: ElementId,
) -> Option<diagram_core::Rect> {
    harness
        .state()
        .document()
        .diagrams
        .get(&diagram)?
        .node_of(element)
        .map(|node| node.bounds)
}

fn undo(harness: &mut Harness<'static, ShellState>) {
    harness.state_mut().queue(UiIntent::Run {
        command: CommandId::Undo,
        origin: None,
    });
    harness.run();
}

#[test]
fn duplicate_dialog_words_should_name_the_class_and_explain_the_choice() -> Result<(), String> {
    let placement = placement_of_sample().ok_or("a placement exists")?;
    let kind = DialogKind::DuplicateClassName {
        placement,
        name: String::from("Customer"),
        existing: Some(ElementId::from_raw(1)),
    };
    let words = dialogs::text(&kind);
    assert_eq!(words.title, TITLE);
    assert!(words.body.contains("\"Customer\""));
    assert_eq!(words.confirm, KEEP_NEW);
    assert_eq!(words.alternative, Some(SHOW_HERE));
    assert_eq!(words.alternative_disabled, None);

    let here = DialogKind::DuplicateClassName {
        placement,
        name: String::from("Order"),
        existing: None,
    };
    assert_eq!(
        dialogs::text(&here).alternative_disabled.as_deref(),
        Some("Every class named \"Order\" is already in this diagram.")
    );
    Ok(())
}

/// A real placement ID, taken from a placement in a harness.
fn placement_of_sample() -> Option<diagram_desktop::ui::class_tool::PlacementId> {
    let mut harness = harness_with(common::LARGE, ShellState::default());
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, 100.0, 300.0);
    harness.state().placement().map(|placement| placement.id)
}

#[test]
fn an_existing_name_should_open_the_dialog_with_focus_on_cancel() {
    let harness = asking("Customer");

    harness.get_by_label(TITLE);
    harness.get_by_label_contains("\"Customer\"");
    assert!(harness.get_by_label("Cancel").is_focused());
    assert!(
        !harness
            .get_by_label(SHOW_HERE)
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label(KEEP_NEW);
    assert!(matches!(
        harness.state().placement().map(|placement| placement.phase),
        Some(Phase::Asking { .. })
    ));
}

#[test]
fn show_here_should_share_the_class_at_its_drawn_size() -> Result<(), String> {
    let mut harness = asking("Customer");
    let details = diagram_id(harness.state(), "Order details");
    let customer = class_id(harness.state(), "Customer");
    let count = harness.state().document().elements.len();

    harness.get_by_label(SHOW_HERE).click();
    harness.run();

    assert!(!harness.state().has_dialog());
    assert_eq!(
        harness.state().document().elements.len(),
        count,
        "no new class"
    );
    let bounds = view(&harness, details, customer).ok_or("Customer is shown in Order details")?;
    assert_eq!((bounds.left(), bounds.top()), (100.0, 300.0));
    // Customer has two members, so its box is taller than an empty one.
    assert!(bounds.size().height() > 60.0, "{bounds:?}");
    assert_eq!(harness.state().selected_element(), Some(customer));

    undo(&mut harness);
    assert!(view(&harness, details, customer).is_none());
    Ok(())
}

#[test]
fn show_here_near_the_bottom_should_move_a_tall_class_up_to_fit() {
    let harness = harness_with(common::LARGE, ShellState::default());
    let canvas = harness.get_by_label("Diagram canvas").rect();
    drop(harness);
    let mut harness = committed_in("Order details", (100.0, canvas.height() - 10.0), "Customer");
    let details = diagram_id(harness.state(), "Order details");
    let customer = class_id(harness.state(), "Customer");

    harness.get_by_label(SHOW_HERE).click();
    harness.run();

    let Some(bounds) = view(&harness, details, customer) else {
        return;
    };
    assert!(
        bounds.top() + bounds.size().height() <= f64::from(canvas.height()) + 0.5,
        "{bounds:?} fits in {canvas:?}"
    );
}

#[test]
fn keep_new_should_make_a_second_class_with_the_name() {
    let mut harness = asking("Customer");

    harness.get_by_label(KEEP_NEW).click();
    harness.run();

    assert!(!harness.state().has_dialog());
    assert_eq!(named(&harness, "Customer").len(), 2);

    undo(&mut harness);
    assert_eq!(named(&harness, "Customer").len(), 1);
}

#[test]
fn cancel_escape_and_outside_click_should_return_to_the_field() {
    for how in ["cancel", "escape", "outside"] {
        let mut harness = asking("Customer");
        let document = harness.state().document().clone();

        match how {
            "cancel" => harness.get_by_label("Cancel").click(),
            "escape" => harness.key_press(Key::Escape),
            _ => {
                let corner = harness.get_by_label("Diagram canvas").rect().min
                    + eframe::egui::Vec2::new(4.0, 4.0);
                harness.hover_at(corner);
                harness.drag_at(corner);
                harness.event(eframe::egui::Event::PointerButton {
                    pos: corner,
                    button: eframe::egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                });
            }
        }
        harness.run();

        assert!(!harness.state().has_dialog(), "{how}");
        assert_eq!(harness.state().document(), &document, "{how}");
        assert_eq!(
            harness.state().placement().map(|placement| placement.phase),
            Some(Phase::Naming),
            "{how}: no click-through to the canvas"
        );
        assert_eq!(
            harness.get_by_label(NEW_CLASS_LABEL).value().as_deref(),
            Some("Customer"),
            "{how}"
        );
        assert!(
            harness.get_by_label(NEW_CLASS_LABEL).is_focused(),
            "{how}: focus back in the field"
        );
        for _ in 0..5 {
            harness.step();
            assert!(!harness.state().has_dialog(), "{how}: stays closed");
        }
    }
}

#[test]
fn show_here_should_be_disabled_with_a_visible_reason_when_already_shown() {
    let harness = committed_in("Class diagram 1", (100.0, 300.0), "Customer");

    assert!(
        harness
            .get_by_label(SHOW_HERE)
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Every class named \"Customer\" is already in this diagram.");
}

#[test]
fn the_keyboard_should_move_between_buttons_and_answer() {
    let mut harness = asking("Customer");

    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label(SHOW_HERE).is_focused());
    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label(KEEP_NEW).is_focused());
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(named(&harness, "Customer").len(), 2);
}

#[test]
fn a_tab_click_whose_commit_asks_should_not_switch() {
    let mut state = ShellState::default();
    let first = diagram_id(&state, "Class diagram 1");
    let details = diagram_id(&state, "Order details");
    state.open_diagram(details);
    let mut harness = harness_with(common::LARGE, state);
    choose_class_tool(&mut harness);
    click_canvas_at(&mut harness, 100.0, 300.0);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label(NEW_CLASS_LABEL).type_text("Customer");
    harness.run();

    harness.state_mut().queue(UiIntent::ActivateTab(first));
    harness.run();

    assert!(harness.state().has_dialog());
    assert_eq!(harness.state().active_diagram(), Some(details));
}

#[test]
fn names_should_match_trimmed_and_case_sensitively() {
    let harness = asking(" Customer ");
    harness.get_by_label(TITLE);

    let harness = asking("customer");
    assert!(!harness.state().has_dialog());
    assert_eq!(named(&harness, "customer").len(), 1);
}
