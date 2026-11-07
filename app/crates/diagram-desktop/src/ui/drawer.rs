//! The drawer that shows the project panel or the inspector over the canvas
//! on narrow windows.
//!
//! Moved out of `shell.rs` unchanged.

use eframe::egui::{self, Area, Frame, Id, Order, Pos2, Rect, Stroke, Vec2};

use crate::shell::ShellState;
use crate::ui::components::{self, scrolling};
use crate::ui::inspector_view::inspector;
use crate::ui::intents::UiIntent;
use crate::ui::project_panel::sidebar;
use crate::ui::theme::{self, Palette};
use crate::ui::workspace::Drawer;

/// The open drawer, over the canvas only, never over the header, rows or
/// status bar.
pub(crate) fn drawer(
    ctx: &egui::Context,
    state: &mut ShellState,
    canvas_rect: Rect,
    palette: &Palette,
) {
    let Some(which) = state.drawer else {
        return;
    };
    let width = which.width(canvas_rect.width());
    let rect = match which {
        Drawer::Navigator => {
            Rect::from_min_size(canvas_rect.min, Vec2::new(width, canvas_rect.height()))
        }
        Drawer::Inspector => Rect::from_min_max(
            Pos2::new(canvas_rect.max.x - width, canvas_rect.min.y),
            canvas_rect.max,
        ),
    };
    let close_label = match which {
        Drawer::Navigator => "Close project panel",
        Drawer::Inspector => "Close inspector",
    };
    let frame = Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0, palette.separator))
        .inner_margin(theme::SPACING);
    let inner_size = rect.size() - Vec2::splat(2.0 * (theme::SPACING + 1.0));
    let mut close = false;
    Area::new(Id::unique("drawer"))
        .order(Order::Foreground)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            frame.show(ui, |ui| {
                ui.set_min_size(inner_size);
                ui.set_max_size(inner_size);
                if components::action_button(ui, close_label, None, Some("Escape")).clicked() {
                    close = true;
                }
                ui.separator();
                scrolling(ui, "drawer", "Drawer contents", |ui| match which {
                    Drawer::Navigator => sidebar(ui, state),
                    Drawer::Inspector => inspector(ui, state),
                });
            });
        });
    if close {
        state.queue(UiIntent::CloseDrawer);
    }
}
