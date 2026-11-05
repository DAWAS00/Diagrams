//! A-3: tool row with overflow (brief §A4).

mod common_ui;

use common_ui::{
    SMALL, audit_layout, audit_view_everywhere, click, harness, in_top_row, light, pressed, report,
};
use diagram_desktop::intent::UiIntent;
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::commands::Availability;
use diagram_desktop::ui::{fixtures, toolbar, workspace};
use eframe::egui::{self, ThemePreference};
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_top_row(ui, workspace::TOOL_ROW_HEIGHT, |ui| {
        toolbar::show(ui, &view.tools, out);
    });
}

/// The tool row squeezed to 420 points, narrower than its tools need.
fn squeezed(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_top_row(ui, workspace::TOOL_ROW_HEIGHT, |ui| {
        ui.set_max_width(420.0);
        ui.allocate_ui(egui::Vec2::new(420.0, workspace::TOOL_ROW_HEIGHT), |ui| {
            toolbar::show(ui, &view.tools, out);
        });
    });
}

#[test]
fn active_tool_and_shown_panels_should_be_pressed() {
    let view = fixtures::sample();
    let harness = light(view.clone(), draw);

    for tool in &view.tools.tools {
        assert_eq!(
            pressed(&harness, &tool.label),
            tool.active == Some(true),
            "{}",
            tool.label
        );
    }
    assert!(pressed(&harness, &view.tools.navigator_toggle.label));
    assert!(pressed(&harness, &view.tools.inspector_toggle.label));
}

#[test]
fn every_tool_should_fit_at_800_without_overflow() {
    let harness = harness(SMALL, ThemePreference::Light, fixtures::sample(), draw);

    assert!(harness.query_by_label(toolbar::MORE_TOOLS).is_none());
}

#[test]
fn tools_that_do_not_fit_should_move_into_more_tools_and_stay_reachable() {
    let mut view = fixtures::sample();
    for tool in &mut view.tools.tools {
        tool.availability = Availability::Enabled;
    }
    let mut harness = harness(SMALL, ThemePreference::Light, view.clone(), squeezed);
    report(audit_layout("squeezed tool row", &harness, SMALL, false));

    harness.get_by_label(toolbar::MORE_TOOLS).click();
    harness.run();
    for tool in &view.tools.tools {
        assert!(
            harness
                .query_all_by(|node| node
                    .data()
                    .label()
                    .is_some_and(|name| name.starts_with(&tool.label)))
                .next()
                .is_some(),
            "{} is neither in the row nor in More tools",
            tool.label
        );
    }
    // A hidden tool runs from the menu like from its button.
    let last = view
        .tools
        .tools
        .last()
        .map(|tool| (tool.id, tool.label.clone()));
    if let Some((id, label)) = last {
        assert_eq!(click(&mut harness, &label), [UiIntent::Command(id)]);
    }
}

#[test]
fn panel_toggle_should_report_itself_as_the_opener() {
    let view = fixtures::sample();
    let mut harness = light(view.clone(), |ui, view, out| {
        in_top_row(ui, workspace::TOOL_ROW_HEIGHT, |ui| {
            let row = toolbar::show(ui, &view.tools, out);
            if row.opener.is_some() {
                out.push(UiIntent::FocusRegion(Default::default()));
            }
        });
    });
    let intents = click(&mut harness, &view.tools.inspector_toggle.label);
    assert_eq!(intents.len(), 2, "{intents:?}");
    assert_eq!(
        intents[0],
        UiIntent::Command(view.tools.inspector_toggle.id)
    );
}

#[test]
fn tool_row_should_pass_the_audit_at_every_size_and_theme() {
    audit_view_everywhere("tool row", &fixtures::sample(), draw, &[]);
    audit_view_everywhere(
        "tool row, panels hidden",
        &fixtures::panels_hidden(),
        draw,
        &[],
    );
}
