//! R2-04 (core half): adding and editing members as text through reversible
//! commands. Edits keep a member's ID, position and kind; refusals change
//! nothing at all.

mod common;

use common::{rect, three_classes};
use diagram_core::command::{self, Change, Command};
use diagram_core::history::Session;
use diagram_core::ids::{ElementId, MemberId};
use diagram_core::member_text::MemberTextError;
use diagram_core::model::{Member, MemberKind, Operation, Visibility};
use diagram_core::validate::{DomainError, check_document};
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;

fn add(session: &mut Session, element: ElementId, text: &str) -> MemberId {
    let result = session.execute(&Command::AddMember {
        element,
        text: text.to_owned(),
    });
    assert!(result.is_ok(), "{text:?}: {result:?}");
    session.document().elements[&element]
        .members
        .last()
        .map_or(MemberId::from_raw(0), Member::id)
}

fn update(element: ElementId, member: MemberId, text: &str) -> Command {
    Command::UpdateMember {
        element,
        member,
        text: text.to_owned(),
    }
}

fn members(session: &Session, element: ElementId) -> Vec<Member> {
    session.document().elements[&element].members.clone()
}

#[test]
fn add_member_should_create_an_operation_with_parameters_and_undo_and_redo_it() -> TestResult {
    let (mut session, ids) = three_classes();
    let before = session.document().clone();

    let id = add(
        &mut session,
        ids.order,
        "# place(customer: CustomerId, lines: List<Line>): Order",
    );

    let added = members(&session, ids.order);
    let Some(Member::Operation(operation)) = added.last() else {
        return Err(format!("an operation was added: {added:?}").into());
    };
    assert_eq!(operation.id, id);
    assert_eq!(operation.visibility, Visibility::Protected);
    let parameters: Vec<(&str, &str)> = operation
        .parameters
        .iter()
        .map(|parameter| (parameter.name.as_str(), parameter.type_text.as_str()))
        .collect();
    assert_eq!(
        parameters,
        [("customer", "CustomerId"), ("lines", "List<Line>")]
    );
    assert_eq!(operation.return_type.as_deref(), Some("Order"));
    assert_eq!(session.undo_label(), Some("Add member"));
    let after = session.document().clone();

    assert!(session.undo().is_some());
    assert_eq!(session.document(), &before);
    assert!(session.redo().is_some());
    assert_eq!(session.document(), &after);
    Ok(())
}

#[test]
fn update_member_should_edit_in_place_with_one_patch_and_exact_undo_and_redo() -> TestResult {
    let (mut session, ids) = three_classes();
    let first = add(&mut session, ids.customer, "id: CustomerId");
    let middle = add(&mut session, ids.customer, "- name: String");
    let last = add(&mut session, ids.customer, "rename(to: String)");
    let before = session.document().clone();

    let command = update(ids.customer, middle, "# fullName: Text [0..1]");
    let prepared = command::prepare(session.document(), &command);
    assert!(prepared.is_ok(), "{prepared:?}");
    if let Ok(set) = &prepared {
        assert_eq!(set.label, "Edit member");
        assert_eq!(set.changes.len(), 1, "one element patch");
        assert!(matches!(set.changes[0], Change::Element { .. }));
        assert!(
            !set.changes
                .iter()
                .any(|change| matches!(change, Change::NextId { .. }))
        );
    }
    let result = session.execute(&command);
    assert!(result.is_ok(), "{result:?}");

    let edited = members(&session, ids.customer);
    let ids_in_order: Vec<MemberId> = edited.iter().map(Member::id).collect();
    assert_eq!(ids_in_order, [first, middle, last], "ID and position kept");
    assert_eq!(edited[0], before.elements[&ids.customer].members[0]);
    assert_eq!(edited[2], before.elements[&ids.customer].members[2]);
    let Member::Attribute(attribute) = &edited[1] else {
        return Err("still an attribute".into());
    };
    assert_eq!(attribute.name.as_str(), "fullName");
    assert_eq!(attribute.type_text, "Text");
    assert_eq!(attribute.visibility, Visibility::Protected);
    assert_eq!(session.document().next_id, before.next_id, "no ID used");
    let after = session.document().clone();

    assert!(session.undo().is_some());
    assert_eq!(session.document(), &before);
    assert!(session.redo().is_some());
    assert_eq!(session.document(), &after);
    Ok(())
}

#[test]
fn an_operation_should_be_editable_with_new_parameters() -> TestResult {
    let (mut session, ids) = three_classes();
    let id = add(&mut session, ids.order, "total(): Money");

    let result = session.execute(&update(
        ids.order,
        id,
        "total(currency: Currency, at: Date): Money",
    ));

    assert!(result.is_ok(), "{result:?}");
    let edited = members(&session, ids.order);
    let Some(Member::Operation(Operation { parameters, .. })) = edited.first() else {
        return Err("still an operation".into());
    };
    assert_eq!(parameters.len(), 2);
    Ok(())
}

#[test]
fn a_kind_change_should_be_refused_in_both_directions() {
    let (mut session, ids) = three_classes();
    let attribute = add(&mut session, ids.order, "total: Money");
    let operation = add(&mut session, ids.order, "total(): Money");
    let before = session.clone();

    assert_eq!(
        session.execute(&update(ids.order, attribute, "total(): Money")),
        Err(DomainError::MemberKindChange {
            from: MemberKind::Attribute,
            to: MemberKind::Operation
        })
    );
    assert_eq!(
        session.execute(&update(ids.order, operation, "total: Money")),
        Err(DomainError::MemberKindChange {
            from: MemberKind::Operation,
            to: MemberKind::Attribute
        })
    );
    assert_eq!(session, before);
    assert_eq!(
        DomainError::MemberKindChange {
            from: MemberKind::Attribute,
            to: MemberKind::Operation
        }
        .to_string(),
        "an attribute cannot become an operation by editing; add an operation instead"
    );
}

#[test]
fn the_same_member_typed_again_should_be_no_change() {
    let (mut session, ids) = three_classes();
    let id = add(&mut session, ids.order, "- total: Money [0..1]");
    let before = session.clone();

    for text in ["- total: Money [0..1]", "  -total :Money  [0..1] "] {
        assert_eq!(
            session.execute(&update(ids.order, id, text)),
            Err(DomainError::NoChange),
            "{text:?}"
        );
    }
    assert_eq!(session, before, "revision, saved state and redo unchanged");
}

#[test]
fn refusals_should_leave_the_whole_session_unchanged() {
    let (mut session, ids) = three_classes();
    let id = add(&mut session, ids.order, "total: Money");
    // Leave something on the redo stack so its survival is checked too.
    let _ = add(&mut session, ids.order, "count: Int");
    assert!(session.undo().is_some());
    let before = session.clone();

    let refusals = [
        // A member of Order named through Customer.
        (
            update(ids.customer, id, "x"),
            DomainError::MissingMember(id),
        ),
        (
            update(ids.order, MemberId::from_raw(9999), "x"),
            DomainError::MissingMember(MemberId::from_raw(9999)),
        ),
        (
            update(ElementId::from_raw(9999), id, "x"),
            DomainError::MissingElement(ElementId::from_raw(9999)),
        ),
        (
            update(ids.order, id, "total: Money = 0"),
            DomainError::MemberText(MemberTextError::DefaultValue),
        ),
        (
            Command::AddMember {
                element: ids.order,
                text: String::from("a(b"),
            },
            DomainError::MemberText(MemberTextError::Unclosed('(')),
        ),
        (
            Command::AddMember {
                element: ElementId::from_raw(9999),
                text: String::from("a"),
            },
            DomainError::MissingElement(ElementId::from_raw(9999)),
        ),
    ];
    for (command, expected) in refusals {
        assert_eq!(session.execute(&command), Err(expected), "{command:?}");
    }
    assert_eq!(
        session, before,
        "document, revision, saved state and redo kept"
    );
    assert_eq!(session.redo_label(), Some("Add member"));
}

#[test]
fn create_undo_create_should_keep_live_ids_unique_and_history_exact() {
    let (mut session, ids) = three_classes();
    let created = session.execute(&Command::CreateClass {
        diagram: ids.diagram,
        name: String::from("Invoice"),
        bounds: rect(820.0, 40.0),
    });
    assert!(created.is_ok());
    let first = session.document().elements.len();
    assert!(session.undo().is_some());
    let undone = session.document().clone();

    let again = session.execute(&Command::CreateClass {
        diagram: ids.diagram,
        name: String::from("Receipt"),
        bounds: rect(820.0, 40.0),
    });
    assert!(again.is_ok());

    // The counter came back with the undo, so the new class may reuse the
    // undone class's ID (documented in model.rs). What must hold: live IDs
    // are unique, the document is valid and history is still exact.
    assert_eq!(session.redo_label(), None, "the old future is gone");
    let document = session.document();
    assert_eq!(document.elements.len(), first);
    let view_ids: Vec<_> = document.diagrams[&ids.diagram]
        .nodes
        .iter()
        .map(|node| node.id.raw())
        .collect();
    let mut every: Vec<u64> = document.elements.keys().map(|id| id.raw()).collect();
    every.extend(view_ids);
    let count = every.len();
    every.sort_unstable();
    every.dedup();
    assert_eq!(every.len(), count, "no two live records share an ID");
    assert!(
        check_document(document).is_empty(),
        "{:?}",
        check_document(document)
    );
    assert!(session.undo().is_some());
    assert_eq!(session.document(), &undone);
}
