//! Helpers shared by the headless UI test files.
//!
//! Each test file declares `mod common;` and imports what it needs. Controls
//! are always found by name or role; never add coordinates here.

#![allow(dead_code)] // each test binary uses only some of these helpers

use diagram_core::command::Command;
use diagram_core::ids::{DiagramId, ElementId};
use diagram_core::{Point, Rect, Size};
use diagram_desktop::session::ProjectSession;
use diagram_desktop::shell::{self, ShellState};
use eframe::egui::Vec2;
use egui_kittest::Harness;

/// Window size most tests use.
pub const LARGE: (f32, f32) = (1280.0, 800.0);

/// A harness of `size` showing `state`, with the first frame already run.
pub fn harness_with(size: (f32, f32), state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(size.0, size.1))
        .build_ui_state(shell::show, state);
    harness.run();
    harness
}

/// The default sample project at 1280×800.
pub fn harness() -> Harness<'static, ShellState> {
    harness_with(LARGE, ShellState::default())
}

/// The default sample project at `width` × `height`.
pub fn harness_at(width: f32, height: f32) -> Harness<'static, ShellState> {
    harness_with((width, height), ShellState::default())
}

/// The default sample project at 1280×800, built without an additional explicit `run()` (the builder still renders its start-up frames).
pub fn harness_unrun() -> Harness<'static, ShellState> {
    Harness::builder()
        .with_size(Vec2::new(LARGE.0, LARGE.1))
        .build_ui_state(shell::show, ShellState::default())
}

/// The workspace showing `session`.
pub fn state_with(session: ProjectSession) -> ShellState {
    let mut state = ShellState::welcome();
    state.open_project(session);
    state
}

/// The ID of the class named `name` in the sample project.
pub fn class_id(state: &ShellState, name: &str) -> ElementId {
    state
        .element_id(name)
        .unwrap_or(ElementId::from_raw(u64::MAX))
}

/// The ID of the diagram named `name`.
pub fn diagram_id(state: &ShellState, name: &str) -> DiagramId {
    state
        .diagram_id(name)
        .unwrap_or(DiagramId::from_raw(u64::MAX))
}

/// Executes `command` on `session`; the test then checks the document, so a
/// refusal shows up as a failed assertion later.
pub fn run(session: &mut ProjectSession, command: Command) {
    let _ = session.execute(&command);
}

/// A rectangle for a new view.
pub fn bounds(x: f64, y: f64) -> Rect {
    Rect::new(
        Point::new(x, y).unwrap_or(Point::ORIGIN),
        Size::new(180.0, 80.0).unwrap_or(Size::UNIT),
    )
}

/// The sample project with the class `name` renamed to `new_name`, as the
/// workspace state.
pub fn sample_with_renamed(name: &str, new_name: &str) -> ShellState {
    let mut session = ProjectSession::sample();
    let element = session
        .document()
        .elements
        .values()
        .find(|element| element.name.as_str() == name)
        .map(|element| element.id);
    if let Some(element) = element {
        run(
            &mut session,
            Command::RenameElement {
                element,
                name: new_name.to_owned(),
            },
        );
    }
    state_with(session)
}

/// A project of `count` class diagrams, each showing Customer and Order, all
/// open in tabs (the last one active).
pub fn state_with_many_diagrams(count: usize) -> ShellState {
    let mut session = ProjectSession::sample();
    let (customer, order) = {
        let document = session.document();
        let find = |wanted: &str| {
            document
                .elements
                .values()
                .find(|element| element.name.as_str() == wanted)
                .map(|element| element.id)
        };
        (find("Customer"), find("Order"))
    };
    for number in 1..=count {
        run(
            &mut session,
            Command::AddDiagram {
                name: format!("Customer overview part {number}"),
            },
        );
        let Some(diagram) = session.document().diagram_order.last().copied() else {
            continue;
        };
        for element in [customer, order].into_iter().flatten() {
            run(
                &mut session,
                Command::ShowInDiagram {
                    element,
                    diagram,
                    bounds: bounds(48.0, 48.0),
                },
            );
        }
    }
    let mut state = state_with(session);
    let diagrams: Vec<DiagramId> = state.document().diagram_order.clone();
    for diagram in diagrams {
        state.open_diagram(diagram);
    }
    state
}

/// Selects the class named `name` through the public selection method.
pub fn select_class(harness: &mut Harness<'static, ShellState>, name: &str) {
    let element = class_id(harness.state(), name);
    harness.state_mut().select(element);
}

/// The name of the selected class, if exactly one is selected.
pub fn selected_name(harness: &Harness<'_, ShellState>) -> Option<String> {
    let state = harness.state();
    let element = state.selected_element()?;
    state
        .document()
        .elements
        .get(&element)
        .map(|element| element.name.as_str().to_owned())
}

/// The current name of `element`.
pub fn name_of(harness: &Harness<'_, ShellState>, element: ElementId) -> String {
    harness
        .state()
        .document()
        .elements
        .get(&element)
        .map(|element| element.name.as_str().to_owned())
        .unwrap_or_default()
}

/// The sample plus a class `Premium` that generalizes to `Customer`.
pub fn state_with_generalization() -> ShellState {
    use diagram_core::notation::RelationKind;

    let mut session = diagram_desktop::session::ProjectSession::sample();
    let diagram = session.document().diagram_order[0];
    run(
        &mut session,
        Command::CreateClass {
            diagram,
            name: String::from("Premium"),
            bounds: bounds(60.0, 260.0),
        },
    );
    let views: Vec<_> = session
        .document()
        .diagrams
        .get(&diagram)
        .map(|diagram| {
            diagram
                .nodes
                .iter()
                .map(|node| (node.element, node.id))
                .collect()
        })
        .unwrap_or_default();
    let view_of = |name: &str| {
        let state = state_with(session.clone());
        let element = class_id(&state, name);
        views
            .iter()
            .find(|(candidate, _)| *candidate == element)
            .map(|(_, view)| *view)
    };
    if let (Some(child), Some(parent)) = (view_of("Premium"), view_of("Customer")) {
        run(
            &mut session,
            Command::Connect {
                diagram,
                kind: RelationKind::Generalization,
                source: child,
                target: parent,
            },
        );
    }
    state_with(session)
}

/// Clicks the canvas with the pointer at (`x`, `y`) points from the canvas
/// node's top-left corner. The pointer stays there afterwards.
pub fn click_canvas_at(harness: &mut Harness<'static, ShellState>, x: f32, y: f32) {
    use egui_kittest::kittest::Queryable;
    let pos = harness.get_by_label("Diagram canvas").rect().min + Vec2::new(x, y);
    harness.hover_at(pos);
    for pressed in [true, false] {
        harness.event(eframe::egui::Event::PointerButton {
            pos,
            button: eframe::egui::PointerButton::Primary,
            pressed,
            modifiers: eframe::egui::Modifiers::NONE,
        });
    }
    harness.run();
}

/// Chooses the Class tool from the tool row.
pub fn choose_class_tool(harness: &mut Harness<'static, ShellState>) {
    use egui_kittest::kittest::Queryable;
    harness.get_by_label("Class").click();
    harness.run();
}
