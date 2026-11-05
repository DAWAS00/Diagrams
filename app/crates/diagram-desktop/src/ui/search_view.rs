//! Object search (spec §3 "Find object", brief §A10), drawn in the project
//! panel while it is open (Ctrl+F). It is visibly separate from the command
//! search: its own titled field, "Find object", and results that name each
//! object's kind and location so duplicate names can be told apart.
//!
//! Typing pushes `Search { scope: Objects }`; a result pushes `Select`;
//! Escape in the field or "Close search" pushes `Close(ObjectSearch)`. The
//! command search modal still lives in `shell.rs` (request 9 in
//! AGENT-A-UI.md §6).

use eframe::egui::{self, Key, TextEdit, Vec2};

use super::components;
use super::theme::{CONTROL_HEIGHT, SPACING};
use crate::intent::{SearchScope, Surface, UiIntent};
use crate::presentation::ObjectSearchView;

/// Label and accessible name of the object search field.
pub const FIND_OBJECT: &str = "Find object";
/// Label of the button that closes the object search.
pub const CLOSE_SEARCH: &str = "Close search";

/// Draws the object search when it is open.
pub fn object_search(ui: &mut egui::Ui, view: &ObjectSearchView, out: &mut Vec<UiIntent>) {
    let opened = ui.scope_id().with("object-search-opened");
    if !view.open {
        ui.data_mut(|data| data.remove::<bool>(opened));
        return;
    }
    let first_frame = !ui.data(|data| data.get_temp::<bool>(opened).unwrap_or(false));
    ui.data_mut(|data| data.insert_temp(opened, true));

    components::panel_title(ui, FIND_OBJECT);
    let label = components::field_label(ui, "Name of a class or diagram");
    let mut query = view.query.clone();
    let field = ui
        .add(
            TextEdit::singleline(&mut query)
                .id_salt("object-search-field")
                .desired_width(f32::INFINITY)
                .min_size(Vec2::new(0.0, CONTROL_HEIGHT)),
        )
        .labelled_by(label.id);
    if first_frame {
        field.request_focus();
    }
    if field.changed() {
        out.push(UiIntent::Search {
            scope: SearchScope::Objects,
            query,
        });
    }
    let escape = field.lost_focus() && ui.input(|input| input.key_pressed(Key::Escape));
    if components::action_button(ui, CLOSE_SEARCH, None, Some("Escape")).clicked() || escape {
        out.push(UiIntent::Close(Surface::ObjectSearch));
    }
    if let Some(text) = view.empty_text {
        ui.label(text);
    }
    for result in &view.results {
        // Kind, name and location in one row, so equal names differ.
        let text = format!("{} — {}", result.label, result.location);
        if components::list_row(ui, false, &text).clicked() {
            out.push(UiIntent::Select {
                target: result.target,
                toggle: false,
            });
        }
    }
    ui.add_space(SPACING);
    ui.separator();
}
