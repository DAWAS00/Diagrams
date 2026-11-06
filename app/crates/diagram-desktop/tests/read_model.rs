//! UI-13: the read model turns a document into rows with typed IDs:
//! ordering, names, counts, relation labels and member lines.

mod common;
use common::{bounds, run};
use diagram_core::command::Command;
use diagram_core::ids::ElementId;
use diagram_core::model::Visibility;
use diagram_desktop::session::ProjectSession;
use diagram_desktop::ui::read_model::{self, CanvasModel};

fn sample() -> ProjectSession {
    ProjectSession::sample()
}

#[test]
fn diagram_rows_should_follow_project_order_with_names() {
    let session = sample();

    let rows = read_model::diagram_rows(session.document());

    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["Class diagram 1", "Order details"]);
    assert_eq!(rows[0].id, session.document().diagram_order[0]);
}

#[test]
fn element_rows_should_list_classes_in_creation_order() {
    let session = sample();

    let names: Vec<String> = read_model::element_rows(session.document())
        .into_iter()
        .map(|row| row.name)
        .collect();

    assert_eq!(names, ["Customer", "Order"]);
}

#[test]
fn relation_rows_should_label_both_ends_and_the_kind() {
    let session = sample();

    let rows = read_model::relation_rows(session.document());

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, "Customer – Order (association)");
}

#[test]
fn canvas_model_should_show_only_the_classes_of_that_diagram() {
    let session = sample();
    let document = session.document();
    let [first, second] = [document.diagram_order[0], document.diagram_order[1]];

    let all = read_model::canvas_model(document, first);
    let details = read_model::canvas_model(document, second);

    assert_eq!(all.classes.len(), 2);
    assert_eq!(all.edges.len(), 1);
    assert_eq!(details.classes.len(), 1);
    assert_eq!(details.classes[0].name, "Order");
    assert!(
        details.edges.is_empty(),
        "the connector needs both classes in the diagram"
    );
    assert_eq!(all.classes[0].lines, ["id: CustomerId", "name: String"]);
    assert_eq!(all.edges[0].target_multiplicity, "*");
    assert_eq!(all.edges[0].source_multiplicity, "1");
}

#[test]
fn canvas_model_of_a_missing_diagram_should_be_empty() {
    let session = sample();

    let model = read_model::canvas_model(
        session.document(),
        diagram_core::ids::DiagramId::from_raw(999),
    );

    assert_eq!(model, CanvasModel::default());
}

#[test]
fn shown_in_should_use_the_real_diagram_names() {
    let session = sample();
    let order = read_model::element_rows(session.document())[1].id;

    assert_eq!(
        read_model::shown_in(session.document(), order),
        "Shown in Class diagram 1 and Order details"
    );
    assert_eq!(
        read_model::shown_in(session.document(), ElementId::from_raw(999)),
        "Not shown in any diagram"
    );
}

#[test]
fn member_lines_should_mark_visibility_and_show_operations() {
    let mut session = sample();
    let customer = read_model::element_rows(session.document())[0].id;
    run(
        &mut session,
        Command::AddAttribute {
            element: customer,
            name: String::from("secret"),
            type_text: String::from("Text"),
            visibility: Visibility::Private,
            multiplicity: String::from("0..1"),
        },
    );
    run(
        &mut session,
        Command::AddOperation {
            element: customer,
            name: String::from("place"),
            visibility: Visibility::Public,
            return_type: String::from("Order"),
        },
    );

    let attributes = read_model::attribute_lines(session.document(), customer);
    let operations = read_model::operation_lines(session.document(), customer);

    assert_eq!(
        attributes,
        ["id: CustomerId", "name: String", "- secret: Text [0..1]"]
    );
    assert_eq!(operations, ["place(): Order"]);
    let card = &read_model::canvas_model(session.document(), session.document().diagram_order[0])
        .classes[0];
    assert_eq!(card.lines.len(), 4, "attributes then operations on the box");
}

#[test]
fn a_new_view_should_appear_where_the_document_put_it() {
    let mut session = sample();
    let order = read_model::element_rows(session.document())[1].id;
    let details = session.document().diagram_order[1];
    let views: Vec<_> = session
        .document()
        .diagrams
        .get(&details)
        .map(|diagram| diagram.nodes.iter().map(|node| node.id).collect())
        .unwrap_or_default();
    run(
        &mut session,
        Command::RemoveViews {
            diagram: details,
            views,
        },
    );
    assert!(read_model::elements_in(session.document(), details).is_empty());

    run(
        &mut session,
        Command::ShowInDiagram {
            element: order,
            diagram: details,
            bounds: bounds(120.0, 40.0),
        },
    );

    let card = &read_model::canvas_model(session.document(), details).classes[0];
    assert_eq!(card.position, (120.0, 40.0));
}

#[test]
fn text_helpers_should_format_members_labels_and_multiplicities() {
    use diagram_core::model::{Multiplicity, Parameter};
    let session = sample();
    let document = session.document();
    let association = document.relations.values().next();
    assert!(association.is_some());
    let Some(association) = association else {
        return;
    };

    assert_eq!(
        read_model::relation_label(document, association),
        "Customer – Order (association)"
    );
    assert_eq!(read_model::multiplicity_text(None), "");
    assert_eq!(
        read_model::multiplicity_text(Multiplicity::parse("2..5").ok()),
        "2..5"
    );
    assert_eq!(
        read_model::multiplicity_text(Multiplicity::parse("0..*").ok()),
        "*"
    );
    let first_member = document
        .elements
        .values()
        .next()
        .and_then(|element| element.members.first());
    assert!(first_member.is_some());
    if let Some(member) = first_member {
        assert_eq!(read_model::member_line(member), "id: CustomerId");
        if let diagram_core::model::Member::Attribute(attribute) = member {
            assert_eq!(read_model::attribute_line(attribute), "id: CustomerId");
        }
    }
    let name = diagram_core::model::Name::new("find");
    assert!(name.is_ok());
    let Ok(name) = name else { return };
    let parameter = |text: &str, type_text: &str| {
        diagram_core::model::Name::new(text).map(|name| Parameter {
            name,
            type_text: type_text.to_owned(),
        })
    };
    let (Ok(id), Ok(untyped)) = (parameter("id", "OrderId"), parameter("flag", "")) else {
        return;
    };
    let operation = diagram_core::model::Operation {
        id: diagram_core::ids::MemberId::from_raw(1),
        name,
        visibility: Visibility::Protected,
        parameters: vec![id, untyped],
        return_type: Some(String::from("Order")),
    };
    assert_eq!(
        read_model::operation_line(&operation),
        "# find(id: OrderId, flag): Order"
    );
}

#[test]
fn classes_shown_should_count_in_words() {
    assert_eq!(read_model::classes_shown(0), "No classes shown");
    assert_eq!(read_model::classes_shown(1), "1 class shown");
    assert_eq!(read_model::classes_shown(2), "2 classes shown");
}
