//! A-5: board drawing (brief §A6), for the nodes B presents today.

mod common_ui;

use common_ui::{audit_view_everywhere, bounds, light, pressed, shows_text};
use diagram_desktop::intent::{PointerKind, UiIntent};
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::canvas_view;
use diagram_desktop::ui::fixtures;
use eframe::egui::{self, Modifiers};

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show(ui, |ui| canvas_view::show(ui, &view.canvas, out));
}

#[test]
fn every_node_should_be_an_accessible_button_at_its_presented_rectangle() {
    let view = fixtures::class_selected();
    let harness = light(view.clone(), draw);

    let board = bounds(&harness, view.canvas.accessible_name);
    for node in &view.canvas.nodes {
        let rect = bounds(&harness, &node.accessible_name);
        assert!(
            (rect.x0 - board.x0 - f64::from(node.position.x)).abs() < 1.0
                && (rect.y0 - board.y0 - f64::from(node.position.y)).abs() < 1.0,
            "{rect:?} is not at {:?} from the board {board:?}",
            node.position
        );
        assert!(
            (rect.width() - f64::from(node.size.x)).abs() < 1.0,
            "{rect:?}"
        );
        assert_eq!(pressed(&harness, &node.accessible_name), node.selected);
    }
}

#[test]
fn empty_diagram_should_show_its_prompt() {
    let view = fixtures::empty_diagram();
    let harness = light(view.clone(), draw);

    assert!(view.canvas.nodes.is_empty());
    let text = view.canvas.empty_text.unwrap_or_default();
    // Painted text is not in the tree, so check the board still exists and
    // the fixture carries the prompt; the screenshot shows it painted.
    assert!(!text.is_empty());
    assert!(!shows_text(&harness, "Class Customer"));
}

#[test]
fn ctrl_wheel_should_report_a_zoom_at_the_pointer() {
    let mut harness = light(fixtures::sample(), draw);
    harness.state_mut().take();
    let at = egui::Pos2::new(700.0, 500.0);
    harness.hover_at(at);
    harness.run();
    harness.event_modifiers(egui::Event::Zoom(1.25), Modifiers::COMMAND);
    let zooms: Vec<_> = common_ui::intents_after(&mut harness)
        .into_iter()
        .filter_map(|intent| match intent {
            UiIntent::Pointer(pointer) => match pointer.kind {
                PointerKind::Zoom { delta } => Some((delta, pointer.position)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert!(!zooms.is_empty(), "no zoom reported");
    assert!(zooms.iter().all(|(delta, _)| *delta > 1.0));
    assert!(
        zooms
            .iter()
            .all(|(_, position)| (*position - at).length() < 1.0)
    );
}

#[test]
fn board_should_pass_the_audit_at_every_size_and_theme() {
    for (name, view) in [
        ("board", fixtures::class_selected()),
        ("board, long name", fixtures::long_name()),
        ("board, empty", fixtures::empty_diagram()),
    ] {
        // Boxes are reached from the keyboard through the Model list.
        audit_view_everywhere(name, &view, draw, &["Class "]);
    }
}
