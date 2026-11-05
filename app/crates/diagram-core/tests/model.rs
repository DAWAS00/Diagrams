//! R1-01 and R1-02: typed records, checked values and domain validation.
//! (That IDs cannot be interchanged is proved by `compile_fail` doc tests
//! in `ids.rs`.)

mod common;

use common::{point, rect, three_classes};
use diagram_core::command::{self, Command};
use diagram_core::ids::{ElementId, ViewId};
use diagram_core::model::{Multiplicity, MultiplicityError, Name, Upper, Visibility};
use diagram_core::notation::RelationKind;
use diagram_core::validate::{self, DomainError, Severity};
use diagram_core::{GeometryError, Rect, Size};

#[test]
fn checked_geometry_should_reject_invalid_values() {
    assert_eq!(
        Rect::from_xywh(0.0, 0.0, 0.0, 10.0),
        Err(GeometryError::NotPositive)
    );
    assert_eq!(
        Rect::from_xywh(0.0, 0.0, -5.0, 10.0),
        Err(GeometryError::NotPositive)
    );
    assert_eq!(
        Rect::from_xywh(f64::NAN, 0.0, 5.0, 10.0),
        Err(GeometryError::NotFinite)
    );
    assert_eq!(Size::new(1.0, f64::INFINITY), Err(GeometryError::NotFinite));
    assert!(point(1.0, 2.0).offset(f64::MAX, 0.0).is_ok());
    assert!(point(f64::MAX, 0.0).offset(f64::MAX, 0.0).is_err());
}

#[test]
fn names_should_be_nonblank_and_keep_their_spelling() {
    assert!(Name::new("  ").is_err());
    assert!(Name::new("").is_err());
    let name = Name::new(" Client عميل ").map(|name| name.as_str().to_owned());
    assert_eq!(name.as_deref(), Ok(" Client عميل "));
}

#[test]
fn multiplicity_should_accept_the_supported_grammar() {
    let parse = |text| Multiplicity::parse(text).map(|m| (m.lower(), m.upper()));
    assert_eq!(parse("1"), Ok((1, Upper::Bounded(1))));
    assert_eq!(parse("*"), Ok((0, Upper::Unbounded)));
    assert_eq!(parse("0..*"), Ok((0, Upper::Unbounded)));
    assert_eq!(parse(" 2..5 "), Ok((2, Upper::Bounded(5))));
    assert_eq!(parse("3..3"), Ok((3, Upper::Bounded(3))));
    for text in ["*", "1", "2..5", "1..*"] {
        let shown = Multiplicity::parse(text).map(|m| m.to_string());
        assert_eq!(shown.as_deref(), Ok(text), "round trip");
    }
    // 0..* and * mean the same; * is the canonical text.
    let shown = Multiplicity::parse("0..*").map(|m| m.to_string());
    assert_eq!(shown.as_deref(), Ok("*"));
}

#[test]
fn multiplicity_should_reject_inverted_and_malformed_text() {
    assert_eq!(
        Multiplicity::parse("5..2"),
        Err(MultiplicityError::Inverted { lower: 5, upper: 2 })
    );
    for text in ["", "-1", "*..3", "1..", "a..b", "1...3", "1.5"] {
        assert!(
            matches!(Multiplicity::parse(text), Err(MultiplicityError::Syntax(_))),
            "{text:?}"
        );
    }
}

#[test]
fn visibility_should_have_uml_symbols() {
    let symbols: Vec<_> = [
        Visibility::Public,
        Visibility::Private,
        Visibility::Protected,
        Visibility::Package,
    ]
    .iter()
    .map(|visibility| visibility.symbol())
    .collect();
    assert_eq!(symbols, ['+', '-', '#', '~']);
}

#[test]
fn ids_should_be_unique_and_never_reused() {
    let (mut session, ids) = three_classes();
    let raw = [ids.customer.raw(), ids.customer_view.raw(), ids.order.raw()];
    assert!(raw[0] < raw[1] && raw[1] < raw[2]);
    let removed = session.execute(&Command::DeleteElement { element: ids.order });
    assert!(removed.is_ok());
    let (again, _) = common::add_class(&mut session, ids.diagram, "Order", 300.0);
    assert_ne!(again, ids.order, "a deleted ID is not handed out again");
}

#[test]
fn model_and_views_should_be_separate_records() {
    let (session, ids) = three_classes();
    let document = session.document();
    // The element knows nothing about geometry; the view only refers to it.
    let views: Vec<_> = document.views_of(ids.customer).collect();
    assert_eq!(views, [(ids.diagram, ids.customer_view)]);
    assert_eq!(document.elements[&ids.customer].name.as_str(), "Customer");
}

#[test]
fn self_generalization_and_cycles_should_be_rejected_without_edits() {
    let (mut session, ids) = three_classes();
    let connect = |source, target| Command::Connect {
        diagram: ids.diagram,
        kind: RelationKind::Generalization,
        source,
        target,
    };
    // PreferredCustomer → Customer is fine.
    assert!(
        session
            .execute(&connect(ids.preferred_view, ids.customer_view))
            .is_ok()
    );
    let before = session.document().clone();

    assert_eq!(
        session.execute(&connect(ids.customer_view, ids.customer_view)),
        Err(DomainError::SelfGeneralization)
    );
    assert_eq!(
        session.execute(&connect(ids.customer_view, ids.preferred_view)),
        Err(DomainError::GeneralizationCycle {
            child: ids.customer,
            parent: ids.preferred,
        })
    );
    assert_eq!(
        session.execute(&connect(ids.preferred_view, ids.customer_view)),
        Err(DomainError::DuplicateGeneralization {
            child: ids.preferred,
            parent: ids.customer,
        })
    );
    assert_eq!(session.document(), &before);
    assert_eq!(session.undo_label(), Some("Add generalization"));
}

#[test]
fn longer_cycles_should_be_found() {
    let (mut session, ids) = three_classes();
    let connect = |source, target| Command::Connect {
        diagram: ids.diagram,
        kind: RelationKind::Generalization,
        source,
        target,
    };
    assert!(
        session
            .execute(&connect(ids.preferred_view, ids.customer_view))
            .is_ok()
    );
    assert!(
        session
            .execute(&connect(ids.customer_view, ids.order_view))
            .is_ok()
    );
    // Order → PreferredCustomer would close Order → Pref → Customer → Order.
    assert!(matches!(
        session.execute(&connect(ids.order_view, ids.preferred_view)),
        Err(DomainError::GeneralizationCycle { .. })
    ));
    assert!(validate::would_cycle(
        session.document(),
        ids.order,
        ids.preferred
    ));
}

#[test]
fn invalid_references_should_produce_typed_errors() {
    let (session, ids) = three_classes();
    let document = session.document();
    let ghost = ElementId::from_raw(999);
    assert_eq!(
        command::prepare(
            document,
            &Command::RenameElement {
                element: ghost,
                name: String::from("X"),
            }
        ),
        Err(DomainError::MissingElement(ghost))
    );
    let ghost_view = ViewId::from_raw(998);
    assert_eq!(
        command::prepare(
            document,
            &Command::Connect {
                diagram: ids.diagram,
                kind: RelationKind::Association,
                source: ids.customer_view,
                target: ghost_view,
            }
        ),
        Err(DomainError::MissingView(ghost_view))
    );
    assert_eq!(
        command::prepare(
            document,
            &Command::Connect {
                diagram: ids.diagram,
                kind: RelationKind::Association,
                source: ids.order_view,
                target: ids.order_view,
            }
        ),
        Err(DomainError::SelfAssociationUnsupported)
    );
    assert!(matches!(
        command::prepare(
            document,
            &Command::RenameElement {
                element: ids.order,
                name: String::from(" "),
            }
        ),
        Err(DomainError::BlankName(_))
    ));
    assert!(matches!(
        command::prepare(
            document,
            &Command::ShowInDiagram {
                element: ids.order,
                diagram: ids.diagram,
                bounds: rect(0.0, 0.0),
            }
        ),
        Err(DomainError::AlreadyInDiagram { .. })
    ));
}

#[test]
fn document_check_should_separate_errors_from_warnings() {
    let (mut session, ids) = three_classes();
    assert!(validate::check_document(session.document()).is_empty());

    // Removing the only view of Order leaves a nonblocking warning.
    let removed = session.execute(&Command::RemoveViews {
        diagram: ids.diagram,
        views: vec![ids.order_view],
    });
    assert!(removed.is_ok());
    let issues = validate::check_document(session.document());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Warning);
    assert_eq!(issues[0].element, Some(ids.order));

    // A dangling reference is a blocking error.
    let mut damaged = session.document().clone();
    damaged.elements.remove(&ids.customer);
    let issues = validate::check_document(&damaged);
    assert!(issues.iter().any(|issue| issue.severity == Severity::Error));
}
