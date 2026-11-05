//! A-2: diagram tab row with overflow (brief §A3).

mod common_ui;

use common_ui::{SMALL, WIDE, audit_view_everywhere, harness, in_top_row, is_focused, pressed};
use diagram_desktop::intent::UiIntent;
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::{fixtures, tabs, workspace};
use eframe::egui::{self, ThemePreference};
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_top_row(ui, workspace::TAB_ROW_HEIGHT, |ui| {
        tabs::show(ui, &view.tabs, out)
    });
}

#[test]
fn active_tab_should_be_pressed_and_others_not() {
    let view = fixtures::many_tabs();
    let harness = harness(WIDE, ThemePreference::Light, view.clone(), draw);

    for tab in view.tabs.tabs.iter().take(2) {
        assert_eq!(pressed(&harness, &tab.label), tab.active, "{}", tab.label);
    }
}

#[test]
fn one_tab_should_not_offer_more_tabs() {
    let harness = harness(SMALL, ThemePreference::Light, fixtures::sample(), draw);

    assert!(harness.query_by_label(tabs::MORE_TABS).is_none());
}

#[test]
fn overflowing_tabs_should_offer_more_tabs_that_reveals_and_focuses_a_tab() {
    let view = fixtures::many_tabs();
    let mut harness = harness(SMALL, ThemePreference::Light, view.clone(), draw);
    harness.run();

    harness.get_by_label(tabs::MORE_TABS).click();
    harness.run();
    let last = &view
        .tabs
        .tabs
        .last()
        .map(|tab| tab.label.clone())
        .unwrap_or_default();
    // The menu row is the second node with the tab's name.
    if let Some(row) = harness.get_all_by_label(last).last() {
        row.click();
    }
    harness.run();
    harness.run();

    assert!(is_focused(&harness, last), "{last} not focused");
}

#[test]
fn tab_row_should_pass_the_audit_at_every_size_and_theme() {
    audit_view_everywhere("tab row", &fixtures::sample(), draw, &[]);
    // Regression: tabs outside the visible window used to stay in the
    // accessibility tree at their unclipped rectangles (outside the window
    // and under "New diagram"). Only whole tabs are drawn now.
    audit_view_everywhere("tab row, many tabs", &fixtures::many_tabs(), draw, &[]);
}
