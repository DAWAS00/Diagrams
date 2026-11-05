//! R1-03 and R1-04: atomic reversible commands, undo/redo restoring the
//! complete state, and saved-state identity through undo, redo and branching.

mod common;

use common::{point, rect, three_classes};
use diagram_core::command::{self, Change, ChangeSet, Command, EndInput, StaleChange};
use diagram_core::history::{HISTORY_LIMIT, Session, SessionId};
use diagram_core::ids::{ElementId, RelationId};
use diagram_core::model::{Member, Visibility};
use diagram_core::notation::RelationKind;
use diagram_core::validate::DomainError;

fn associate(session: &mut Session, ids: &common::Ids) {
    let result = session.execute(&Command::Connect {
        diagram: ids.diagram,
        kind: RelationKind::Association,
        source: ids.customer_view,
        target: ids.order_view,
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn create_class_should_add_model_and_view_together_and_undo_both() {
    let (mut session, ids) = three_classes();
    let before = session.document().clone();
    common::add_class(&mut session, ids.diagram, "Invoice", 820.0);
    let diagram = &session.document().diagrams[&ids.diagram];
    assert_eq!(diagram.nodes.len(), 4);
    assert_eq!(session.document().elements.len(), 4);

    assert_eq!(session.undo().map(|o| o.label), Some("Add class"));
    assert_eq!(session.document(), &before);
}

#[test]
fn rename_should_show_in_every_view_and_keep_geometry() {
    let (mut session, ids) = three_classes();
    let second = session.execute(&Command::AddDiagram {
        name: String::from("Overview"),
    });
    assert!(second.is_ok());
    let overview = session.document().diagram_order[1];
    let shown = session.execute(&Command::ShowInDiagram {
        element: ids.customer,
        diagram: overview,
        bounds: rect(10.0, 10.0),
    });
    assert!(shown.is_ok());
    let geometry: Vec<_> = session
        .document()
        .diagrams
        .values()
        .flat_map(|diagram| diagram.nodes.iter().map(|node| node.bounds))
        .collect();

    let renamed = session.execute(&Command::RenameElement {
        element: ids.customer,
        name: String::from("Client"),
    });
    assert!(renamed.is_ok());

    let document = session.document();
    assert_eq!(document.elements[&ids.customer].name.as_str(), "Client");
    assert_eq!(
        document.views_of(ids.customer).count(),
        2,
        "one model, two views"
    );
    let after: Vec<_> = document
        .diagrams
        .values()
        .flat_map(|diagram| diagram.nodes.iter().map(|node| node.bounds))
        .collect();
    assert_eq!(after, geometry);
}

#[test]
fn members_should_keep_their_identity_through_reordering() {
    let (mut session, ids) = three_classes();
    for name in ["id", "name"] {
        let added = session.execute(&Command::AddAttribute {
            element: ids.customer,
            name: name.to_owned(),
            type_text: String::from("String"),
            visibility: Visibility::Private,
            multiplicity: String::new(),
        });
        assert!(added.is_ok());
    }
    let added = session.execute(&Command::AddOperation {
        element: ids.customer,
        name: String::from("rename"),
        visibility: Visibility::Public,
        return_type: String::new(),
    });
    assert!(added.is_ok());
    let members = &session.document().elements[&ids.customer].members;
    let order: Vec<_> = members.iter().map(Member::id).collect();

    let moved = session.execute(&Command::MoveMember {
        element: ids.customer,
        member: order[2],
        to: 0,
    });
    assert!(moved.is_ok());
    let members = &session.document().elements[&ids.customer].members;
    let reordered: Vec<_> = members.iter().map(Member::id).collect();
    assert_eq!(reordered, [order[2], order[0], order[1]]);

    assert_eq!(
        session.execute(&Command::MoveMember {
            element: ids.customer,
            member: order[0],
            to: 9,
        }),
        Err(DomainError::MemberIndex { index: 9, count: 3 })
    );
    assert!(matches!(
        session.execute(&Command::AddAttribute {
            element: ids.customer,
            name: String::from("tags"),
            type_text: String::new(),
            visibility: Visibility::Public,
            multiplicity: String::from("3..1"),
        }),
        Err(DomainError::Multiplicity(_))
    ));
}

#[test]
fn association_ends_should_validate_and_generalizations_should_have_none() {
    let (mut session, ids) = three_classes();
    associate(&mut session, &ids);
    let relation = session
        .document()
        .relations
        .keys()
        .next()
        .copied()
        .unwrap_or(RelationId::from_raw(0));
    let set = session.execute(&Command::SetEnds {
        relation,
        source: EndInput {
            role: String::from("buyer"),
            multiplicity: String::from("1"),
        },
        target: EndInput {
            role: String::new(),
            multiplicity: String::from("0..*"),
        },
    });
    assert!(set.is_ok(), "{set:?}");
    let ends = &session.document().relations[&relation];
    assert_eq!(
        ends.target_end
            .multiplicity
            .map(|m| m.to_string())
            .as_deref(),
        Some("*")
    );

    let general = session.execute(&Command::Connect {
        diagram: ids.diagram,
        kind: RelationKind::Generalization,
        source: ids.preferred_view,
        target: ids.customer_view,
    });
    assert!(general.is_ok());
    let generalization = session
        .document()
        .relations
        .keys()
        .last()
        .copied()
        .unwrap_or(RelationId::from_raw(0));
    assert_eq!(
        session.execute(&Command::SetEnds {
            relation: generalization,
            source: EndInput::default(),
            target: EndInput::default(),
        }),
        Err(DomainError::EndsOnGeneralization)
    );
}

#[test]
fn delete_everywhere_should_remove_the_whole_graph_atomically_and_undo_exactly() {
    let (mut session, ids) = three_classes();
    associate(&mut session, &ids);
    let added = session.execute(&Command::AddDiagram {
        name: String::from("Overview"),
    });
    assert!(added.is_ok());
    let overview = session.document().diagram_order[1];
    for (element, x) in [(ids.customer, 0.0), (ids.order, 300.0)] {
        let shown = session.execute(&Command::ShowInDiagram {
            element,
            diagram: overview,
            bounds: rect(x, 0.0),
        });
        assert!(shown.is_ok());
    }
    let before = session.document().clone();

    let impact = command::delete_impact(session.document(), ids.customer);
    assert_eq!(
        impact.map(|i| (i.views, i.relations, i.connectors, i.diagrams)),
        Some((2, 1, 1, 2))
    );
    let deleted = session.execute(&Command::DeleteElement {
        element: ids.customer,
    });
    assert!(deleted.is_ok());
    let document = session.document();
    assert!(!document.elements.contains_key(&ids.customer));
    assert!(document.relations.is_empty());
    assert_eq!(document.views_of(ids.customer).count(), 0);
    assert!(document.diagrams.values().all(|d| d.edges.is_empty()));
    assert!(
        diagram_core::validate::check_document(document)
            .iter()
            .all(|issue| issue.severity == diagram_core::validate::Severity::Warning)
    );

    assert_eq!(
        session.undo().map(|o| o.label),
        Some("Delete model everywhere")
    );
    assert_eq!(
        session.document(),
        &before,
        "undo restores the complete graph"
    );
}

#[test]
fn remove_view_should_keep_the_model_and_its_other_views() {
    let (mut session, ids) = three_classes();
    associate(&mut session, &ids);
    let removed = session.execute(&Command::RemoveViews {
        diagram: ids.diagram,
        views: vec![ids.order_view],
    });
    assert!(removed.is_ok());
    let document = session.document();
    assert!(document.elements.contains_key(&ids.order));
    assert_eq!(document.relations.len(), 1, "the relation stays");
    assert!(
        document.diagrams[&ids.diagram].edges.is_empty(),
        "its connector goes"
    );
}

#[test]
fn a_failed_command_should_preserve_the_previous_document() {
    let (mut session, ids) = three_classes();
    let before = session.clone();
    let rejected = session.execute(&Command::MoveViews {
        diagram: ids.diagram,
        moves: vec![
            (ids.customer_view, point(5.0, 5.0)),
            (diagram_core::ids::ViewId::from_raw(4040), point(1.0, 1.0)),
        ],
    });
    assert!(matches!(rejected, Err(DomainError::MissingView(_))));
    assert_eq!(
        session, before,
        "document, history and identities unchanged"
    );
}

#[test]
fn a_stale_change_set_should_roll_back_completely() {
    let (session, ids) = three_classes();
    let mut document = session.document().clone();
    let before = document.clone();
    let valid = command::prepare(
        &document,
        &Command::RenameElement {
            element: ids.order,
            name: String::from("Purchase"),
        },
    );
    let Ok(valid) = valid else {
        return assert!(valid.is_ok());
    };
    // Append a patch that cannot match: removing an element that is absent.
    let mut broken: ChangeSet = valid.clone();
    broken.changes.push(Change::Element {
        id: ElementId::from_raw(777),
        before: document.elements.get(&ids.order).cloned(),
        after: None,
    });
    assert_eq!(command::apply(&mut document, &broken), Err(StaleChange));
    assert_eq!(document, before, "the first patch was rolled back");

    assert_eq!(command::apply(&mut document, &valid), Ok(()));
    assert_eq!(command::apply(&mut document, &valid.inverse()), Ok(()));
    assert_eq!(document, before);
}

#[test]
fn undo_and_redo_should_restore_every_state_and_label() {
    let (mut session, ids) = three_classes();
    let states = [session.document().clone()];
    let renamed = session.execute(&Command::RenameElement {
        element: ids.order,
        name: String::from("Purchase"),
    });
    assert!(renamed.is_ok());
    let after_rename = session.document().clone();
    associate(&mut session, &ids);
    let moved = session.execute(&Command::MoveViews {
        diagram: ids.diagram,
        moves: vec![(ids.order_view, point(400.0, 200.0))],
    });
    assert!(moved.is_ok());
    let last = session.document().clone();

    assert_eq!(session.undo_label(), Some("Move"));
    assert_eq!(session.undo().map(|o| o.label), Some("Move"));
    assert_eq!(session.undo().map(|o| o.label), Some("Add association"));
    assert_eq!(session.document(), &after_rename);
    assert_eq!(session.redo_label(), Some("Add association"));
    assert!(session.redo().is_some() && session.redo().is_some());
    assert_eq!(session.document(), &last);
    assert!(session.redo().is_none());
    for _ in 0..3 {
        assert!(session.undo().is_some());
    }
    assert_eq!(session.document().elements, states[0].elements);
}

#[test]
fn saved_state_should_survive_undo_redo_and_branching() {
    let (mut session, ids) = three_classes();
    assert!(session.is_dirty(), "a new project is unsaved");
    let saved = session.state();
    assert!(session.mark_saved(session.id(), saved));
    assert!(!session.is_dirty());

    let renamed = session.execute(&Command::RenameElement {
        element: ids.order,
        name: String::from("Purchase"),
    });
    assert!(renamed.is_ok());
    assert!(session.is_dirty());
    assert!(session.undo().is_some());
    assert!(!session.is_dirty(), "undo back to the saved state is clean");
    assert!(session.redo().is_some());
    assert!(session.is_dirty());

    // Branch: undo, then a different edit. The saved state is reachable by
    // undo, but the old redo future is gone.
    assert!(session.undo().is_some());
    let other = session.execute(&Command::RenameElement {
        element: ids.order,
        name: String::from("Basket"),
    });
    assert!(other.is_ok());
    assert!(session.redo().is_none());
    assert!(session.is_dirty());
    assert!(session.undo().is_some());
    assert!(!session.is_dirty());
}

#[test]
fn revisions_should_increase_and_stale_results_should_be_ignored() {
    let (mut session, ids) = three_classes();
    let start = session.revision();
    let stamp = session.state();
    let renamed = session.execute(&Command::RenameElement {
        element: ids.order,
        name: String::from("Purchase"),
    });
    assert!(renamed.is_ok());
    assert!(session.revision() > start);
    let before_undo = session.revision();
    assert!(session.undo().is_some());
    assert!(
        session.revision() > before_undo,
        "undo is a new revision too"
    );

    // A save result from another session is ignored.
    assert!(!session.mark_saved(SessionId::from_raw(99), session.state()));
    assert!(session.is_dirty());
    // A save of an older state leaves newer edits dirty.
    assert!(session.redo().is_some());
    assert!(session.mark_saved(session.id(), stamp));
    assert!(session.is_dirty());
}

#[test]
fn history_should_be_bounded_and_evict_whole_transactions() {
    let (mut session, ids) = three_classes();
    for step in 0..(HISTORY_LIMIT + 20) {
        #[expect(clippy::cast_precision_loss, reason = "small test values")]
        let x = step as f64;
        let moved = session.execute(&Command::MoveViews {
            diagram: ids.diagram,
            moves: vec![(ids.order_view, point(x + 1.0, 0.0))],
        });
        assert!(moved.is_ok());
    }
    assert_eq!(session.undo_depth(), HISTORY_LIMIT);
    let mut undone = 0;
    while session.undo().is_some() {
        undone += 1;
    }
    assert_eq!(undone, HISTORY_LIMIT);
    // The oldest moves were evicted whole: the view sits where the first
    // kept transaction found it, never half-way.
    let node = session.document().diagrams[&ids.diagram]
        .node(ids.order_view)
        .map(|node| node.bounds.left());
    assert_eq!(node, Some(20.0));
}
