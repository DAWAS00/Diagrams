//! A-8: dialogs (brief §A9). Today's contract has the Resolve edit dialog.

mod common_ui;

use common_ui::{
    SIZES, THEMES, audit_layout, click, harness, intents_after, is_focused, light, report,
};
use diagram_desktop::intent::{ResolveChoice, UiIntent};
use diagram_desktop::presentation::DialogView;
use diagram_desktop::ui::{dialogs, fixtures};
use eframe::egui::{self, Key};
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Option<DialogView>, out: &mut Vec<UiIntent>) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.label("Workspace behind the dialog");
        dialogs::show(ui, view.as_ref(), out);
    });
}

#[test]
fn resolve_edit_should_show_its_message_and_focus_return_to_field() {
    let view = Some(fixtures::resolve_edit());
    let mut harness = light(view, draw);
    harness.run();

    harness.get_by_label(dialogs::RESOLVE_TITLE);
    assert!(harness.query_by_label_contains("cannot be kept").is_some());
    assert!(is_focused(&harness, dialogs::RETURN_TO_FIELD));
}

#[test]
fn resolve_edit_buttons_should_emit_their_choice() {
    let mut harness = light(Some(fixtures::resolve_edit()), draw);

    assert_eq!(
        click(&mut harness, dialogs::RETURN_TO_FIELD),
        [UiIntent::ResolveEdit(ResolveChoice::ReturnToField)]
    );
    assert_eq!(
        click(&mut harness, dialogs::DISCARD_EDIT),
        [UiIntent::ResolveEdit(ResolveChoice::DiscardEdit)]
    );
}

#[test]
fn escape_should_return_to_the_field_and_never_discard() {
    let mut harness = light(Some(fixtures::resolve_edit()), draw);
    harness.state_mut().take();

    harness.key_press(Key::Escape);
    let intents = intents_after(&mut harness);

    assert!(
        intents.contains(&UiIntent::ResolveEdit(ResolveChoice::ReturnToField)),
        "{intents:?}"
    );
    assert!(!intents.contains(&UiIntent::ResolveEdit(ResolveChoice::DiscardEdit)));
}

#[test]
fn tab_should_stay_inside_the_dialog() {
    let mut harness = light(Some(fixtures::resolve_edit()), draw);
    harness.run();
    for _ in 0..6 {
        harness.key_press(Key::Tab);
        harness.run();
        assert!(
            is_focused(&harness, dialogs::RETURN_TO_FIELD)
                || is_focused(&harness, dialogs::DISCARD_EDIT),
            "focus left the dialog"
        );
    }
}

#[test]
fn no_dialog_should_draw_nothing() {
    let harness = light(None, draw);

    assert!(harness.query_by_label(dialogs::RESOLVE_TITLE).is_none());
}

#[test]
fn resolve_edit_should_pass_the_audit_at_every_size_and_theme() {
    let mut findings = Vec::new();
    for size in SIZES {
        for theme in THEMES {
            let harness = harness(size, theme, Some(fixtures::resolve_edit()), draw);
            findings.extend(audit_layout(
                &format!("resolve edit at {size:?} {theme:?}"),
                &harness,
                size,
                true,
            ));
        }
    }
    report(findings);
}
