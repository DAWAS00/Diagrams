//! Modal dialogs (spec §6 and §7, brief §A9).
//!
//! Today the contract has one dialog for this module: **Resolve edit**, shown
//! when an invalid draft would be lost. Its default focus is "Return to
//! field"; Escape or a click outside also returns to the field, so nothing is
//! discarded by accident. egui's `Modal` blocks the widgets behind it, so
//! Tab stays inside. The other dialogs wait for request 8 in AGENT-A-UI.md §6.

use eframe::egui::{self, Id, Modal};

use super::components;
use crate::intent::{ResolveChoice, UiIntent};
use crate::presentation::DialogView;

/// Title of the Resolve edit dialog.
pub const RESOLVE_TITLE: &str = "Resolve the edit first";
/// Its safe choice, focused when it opens.
pub const RETURN_TO_FIELD: &str = "Return to field";
/// Its destructive choice.
pub const DISCARD_EDIT: &str = "Discard edit";

/// Draws the open dialog, if this module owns it. Call every frame, also
/// with `None`, so the dialog's first-frame focus is set once per opening.
pub fn show(ui: &mut egui::Ui, view: Option<&DialogView>, out: &mut Vec<UiIntent>) {
    let opened = ui.scope_id().with("dialog-opened");
    let Some(DialogView::ResolveEdit { message }) = view else {
        ui.data_mut(|data| data.remove::<bool>(opened));
        return;
    };
    let first_frame = !ui.data(|data| data.get_temp::<bool>(opened).unwrap_or(false));
    let modal = Modal::new(Id::unique("resolve-edit")).show(ui.ctx(), |ui| {
        ui.set_max_width(440.0);
        components::panel_title(ui, RESOLVE_TITLE);
        ui.label(message.as_str());
        ui.add_space(super::theme::SPACING);
        ui.horizontal(|ui| {
            let back = components::action_button(ui, RETURN_TO_FIELD, None, Some("Escape"));
            if first_frame {
                back.request_focus();
            }
            if back.clicked() {
                out.push(UiIntent::ResolveEdit(ResolveChoice::ReturnToField));
            }
            if components::action_button(ui, DISCARD_EDIT, None, None).clicked() {
                out.push(UiIntent::ResolveEdit(ResolveChoice::DiscardEdit));
            }
        });
    });
    ui.data_mut(|data| data.insert_temp(opened, true));
    if modal.should_close() {
        out.push(UiIntent::ResolveEdit(ResolveChoice::ReturnToField));
    }
}
