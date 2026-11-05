//! A-9: object search (brief §A10), separate from command search.

mod common_ui;

use common_ui::{audit_view_everywhere, click, in_side_panel, intents_after, is_focused, light};
use diagram_desktop::intent::{SearchScope, Surface, UiIntent};
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::{fixtures, search_view};
use eframe::egui::{self, Key};
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_side_panel(ui, 240.0, |ui| {
        search_view::object_search(ui, &view.object_search, out)
    });
}

#[test]
fn closed_search_should_draw_nothing() {
    let harness = light(fixtures::sample(), draw);

    assert!(harness.query_by_label(search_view::FIND_OBJECT).is_none());
}

#[test]
fn open_search_should_focus_its_field_and_report_typing_as_an_object_search() {
    let mut harness = light(fixtures::object_search(""), draw);
    harness.run();
    assert!(is_focused(&harness, "Name of a class or diagram"));

    harness.state_mut().take();
    harness
        .get_by_label("Name of a class or diagram")
        .type_text("Ord");
    assert_eq!(
        intents_after(&mut harness),
        [UiIntent::Search {
            scope: SearchScope::Objects,
            query: String::from("Ord")
        }]
    );
}

#[test]
fn results_should_name_kind_and_location_and_select_their_object() {
    let view = fixtures::object_search("Order");
    assert!(
        !view.object_search.results.is_empty(),
        "the fixture finds Order"
    );
    let result = view.object_search.results[0].clone();
    let mut harness = light(view, draw);
    let row = format!("{} — {}", result.label, result.location);

    assert_eq!(
        click(&mut harness, &row),
        [UiIntent::Select {
            target: result.target,
            toggle: false
        }]
    );
}

#[test]
fn no_match_should_say_so_in_words() {
    let view = fixtures::object_search("Invoice");
    let text = view.object_search.empty_text.unwrap_or_default();
    let harness = light(view, draw);

    assert!(!text.is_empty());
    assert!(harness.query_by_label(text).is_some());
}

#[test]
fn escape_in_the_field_or_close_should_close_the_search() {
    let mut harness = light(fixtures::object_search(""), draw);
    harness.run();
    harness.state_mut().take();
    harness.key_press(Key::Escape);
    assert!(intents_after(&mut harness).contains(&UiIntent::Close(Surface::ObjectSearch)));

    assert_eq!(
        click(&mut harness, search_view::CLOSE_SEARCH),
        [UiIntent::Close(Surface::ObjectSearch)]
    );
}

#[test]
fn object_search_should_pass_the_audit_at_every_size_and_theme() {
    audit_view_everywhere(
        "object search",
        &fixtures::object_search("Order"),
        draw,
        &[],
    );
    audit_view_everywhere(
        "object search, no match",
        &fixtures::object_search("Invoice"),
        draw,
        &[],
    );
}
