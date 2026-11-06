//! R2-03 / UI-13: the session wrapper around the core history. Executing a
//! command changes the document; a refused command changes nothing; dirty
//! and close-prompt state follow the history.

use diagram_core::command::Command;
use diagram_core::ids::ElementId;
use diagram_desktop::session::{FIRST_DIAGRAM, ProjectSession, SAMPLE_PROJECT};

fn customer(session: &ProjectSession) -> Option<ElementId> {
    session
        .document()
        .elements
        .values()
        .find(|element| element.name.as_str() == "Customer")
        .map(|element| element.id)
}

fn rename(element: ElementId, name: &str) -> Command {
    Command::RenameElement {
        element,
        name: name.to_owned(),
    }
}

#[test]
fn executing_a_command_should_change_the_document_and_the_history() {
    let mut session = ProjectSession::sample();
    let element = customer(&session);
    assert!(element.is_some(), "the sample has a Customer class");
    let Some(element) = element else { return };
    let revision = session.revision();

    let outcome = session.execute(&rename(element, "Client"));

    assert!(outcome.is_ok());
    assert_eq!(
        session
            .document()
            .elements
            .get(&element)
            .map(|class| class.name.as_str()),
        Some("Client")
    );
    assert!(session.revision() > revision);
    assert_eq!(session.undo_label(), Some("Rename class"));
    assert_eq!(session.redo_label(), None);
}

#[test]
fn a_refused_command_should_leave_the_document_unchanged() {
    let mut session = ProjectSession::sample();
    let Some(element) = customer(&session) else {
        return;
    };
    let before = session.document().clone();
    let revision = session.revision();

    let blank = session.execute(&rename(element, "   "));
    let missing = session.execute(&rename(ElementId::from_raw(999), "Ghost"));

    assert!(blank.is_err());
    assert!(missing.is_err());
    assert_eq!(session.document(), &before);
    assert_eq!(session.revision(), revision, "a refusal is not an edit");
    assert_eq!(session.undo_label(), None);
}

#[test]
fn undo_and_redo_should_restore_the_exact_document() {
    let mut session = ProjectSession::sample();
    let Some(element) = customer(&session) else {
        return;
    };
    let before = session.document().clone();
    let _ = session.execute(&rename(element, "Client"));
    let after = session.document().clone();

    assert!(session.undo().is_some());
    assert_eq!(session.document(), &before);
    assert_eq!(session.redo_label(), Some("Rename class"));
    assert!(session.redo().is_some());
    assert_eq!(session.document(), &after);
    assert!(session.redo().is_none(), "nothing left to redo");
}

#[test]
fn closing_should_need_a_prompt_only_after_an_edit() {
    let mut session = ProjectSession::blank("Orders");
    assert!(!session.has_unsaved_changes(), "an untouched new project");

    let _ = session.execute(&Command::AddDiagram {
        name: String::from("Second"),
    });
    assert!(session.has_unsaved_changes());
}

#[test]
fn a_new_project_should_start_with_one_empty_class_diagram() {
    let session = ProjectSession::blank("  Orders ");

    assert_eq!(session.name(), "  Orders ", "the spelling is kept");
    let document = session.document();
    assert_eq!(document.diagram_order.len(), 1);
    assert!(document.elements.is_empty());
    assert_eq!(
        document
            .diagram_order
            .first()
            .and_then(|id| document.diagrams.get(id))
            .map(|diagram| diagram.name.as_str()),
        Some(FIRST_DIAGRAM)
    );
}

#[test]
fn the_sample_project_should_equal_the_document_built_by_commands() {
    let session = ProjectSession::sample();
    let document = session.document();

    assert_eq!(session.name(), SAMPLE_PROJECT);
    assert_eq!(session.undo_label(), None, "no undo history on open");
    assert!(!session.has_unsaved_changes());
    // Two classes with the four sample attributes, one association with its
    // roles and multiplicities, and two diagrams, the second showing Order.
    assert_eq!(document.elements.len(), 2);
    let attributes: usize = document
        .elements
        .values()
        .map(|element| element.members.len())
        .sum();
    assert_eq!(attributes, 4);
    assert_eq!(document.relations.len(), 1);
    assert_eq!(document.diagram_order.len(), 2);
    let shown: Vec<usize> = document
        .diagram_order
        .iter()
        .filter_map(|id| document.diagrams.get(id))
        .map(|diagram| diagram.nodes.len())
        .collect();
    assert_eq!(shown, [2, 1]);
    // The fixture is itself valid: re-running the same commands on an empty
    // document would produce the same shape (checked through the core rules).
    assert!(diagram_core::validate::check_document(document).is_empty());
}

#[test]
fn the_sample_should_equal_an_independently_built_document() {
    use diagram_core::command::{self, EndInput};
    use diagram_core::model::{Document, Name, Visibility};
    use diagram_core::notation::RelationKind;
    use diagram_core::{Point, Rect, Size};

    let at = |x: f64, y: f64| {
        Rect::new(
            Point::new(x, y).unwrap_or(Point::ORIGIN),
            Size::new(180.0, 80.0).unwrap_or(Size::UNIT),
        )
    };
    let first = Name::new(FIRST_DIAGRAM);
    assert!(first.is_ok());
    let Ok(first) = first else { return };
    let (mut expected, diagram) = Document::with_diagram(first);
    let steps = |expected: &mut Document, command: Command| {
        assert!(command::execute(expected, &command).is_ok(), "{command:?}");
    };
    steps(
        &mut expected,
        Command::CreateClass {
            diagram,
            name: "Customer".into(),
            bounds: at(48.0, 48.0),
        },
    );
    steps(
        &mut expected,
        Command::CreateClass {
            diagram,
            name: "Order".into(),
            bounds: at(300.0, 96.0),
        },
    );
    let ids: Vec<_> = expected.elements.keys().copied().collect();
    let (customer, order) = (ids[0], ids[1]);
    for (element, name, type_text) in [
        (customer, "id", "CustomerId"),
        (customer, "name", "String"),
        (order, "id", "OrderId"),
        (order, "total", "Money"),
    ] {
        steps(
            &mut expected,
            Command::AddAttribute {
                element,
                name: name.into(),
                type_text: type_text.into(),
                visibility: Visibility::Public,
                multiplicity: String::new(),
            },
        );
    }
    let view_of = |expected: &Document, element| {
        expected
            .diagrams
            .get(&diagram)
            .and_then(|diagram| diagram.node_of(element))
            .map(|node| node.id)
    };
    let (Some(source), Some(target)) = (view_of(&expected, customer), view_of(&expected, order))
    else {
        return;
    };
    steps(
        &mut expected,
        Command::Connect {
            diagram,
            kind: RelationKind::Association,
            source,
            target,
        },
    );
    let relation = expected.relations.keys().next().copied();
    let Some(relation) = relation else { return };
    steps(
        &mut expected,
        Command::SetEnds {
            relation,
            source: EndInput {
                role: "customer".into(),
                multiplicity: "1".into(),
            },
            target: EndInput {
                role: "orders".into(),
                multiplicity: "0..*".into(),
            },
        },
    );
    steps(
        &mut expected,
        Command::AddDiagram {
            name: "Order details".into(),
        },
    );
    let second = expected.diagram_order.last().copied();
    let Some(second) = second else { return };
    steps(
        &mut expected,
        Command::ShowInDiagram {
            element: order,
            diagram: second,
            bounds: at(48.0, 48.0),
        },
    );

    assert_eq!(ProjectSession::sample().document(), &expected);
}
