//! R1-05 / R1-08: the round trip is exact, and damaged or tampered files are
//! refused with a reason, return nothing, and are never written.

mod common;
use std::error::Error;
use std::fs;

use common::{bytes, element, folder, project, rect, run};
use diagram_core::command::{self, Command};
use diagram_core::ids::ElementId;
use diagram_storage::dto::ProjectDto;
use diagram_storage::{LoadError, load, save};
use rusqlite::Connection;

#[test]
fn the_round_trip_should_be_exact() -> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    let original = project();

    save(&path, "متجر Shop", &original)?;
    let loaded = load(&path)?;

    assert_eq!(loaded.name, "متجر Shop");
    assert_eq!(loaded.document, original, "every record, order and bound");
    assert_eq!(loaded.document.next_id, original.next_id);
    // Spot checks that the comparison above is not vacuous.
    assert_eq!(loaded.document.elements.len(), 3);
    assert_eq!(loaded.document.relations.len(), 2);
    assert_eq!(loaded.document.diagram_order.len(), 2);
    assert!(element(&loaded.document, "عميل مميز").is_some());
    Ok(())
}

#[test]
fn dto_rows_should_rebuild_the_same_document_without_a_file() -> Result<(), Box<dyn Error>> {
    let original = project();

    let dto = ProjectDto::from_document("Shop", &original);
    let (name, rebuilt) = dto.clone().into_document()?;

    assert_eq!(name, "Shop");
    assert_eq!(rebuilt, original);
    assert_eq!(dto.elements.len(), 3);
    assert_eq!(dto.parameters.len(), 2, "the operation's two parameters");
    Ok(())
}

#[test]
fn ids_should_never_be_reused_after_reopening() -> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    let original = project();
    save(&path, "Shop", &original)?;
    let mut loaded = load(&path)?.document;
    let largest = loaded
        .elements
        .keys()
        .map(|id| id.raw())
        .max()
        .unwrap_or_default();
    let diagram = loaded.diagram_order[0];

    run(
        &mut loaded,
        Command::CreateClass {
            diagram,
            name: String::from("Later"),
            bounds: rect(500.0, 500.0),
        },
    );

    let later = element(&loaded, "Later")
        .map(|id| id.raw())
        .unwrap_or_default();
    assert!(later > original.next_id, "{later} reuses an old ID");
    assert!(later > largest);
    Ok(())
}

#[test]
fn loading_should_not_write_the_file() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let before = bytes(&path);

    let _ = load(&path)?;

    assert_eq!(bytes(&path), before);
    assert_eq!(
        common::names_in(dir.path()),
        ["shop.dgram"],
        "no journal or temp file"
    );
    Ok(())
}

#[test]
fn a_truncated_file_should_be_refused() -> Result<(), Box<dyn Error>> {
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let whole = bytes(&path);

    for keep in [whole.len() / 2, 4096, 60, 0] {
        let cut = dir.path().join(format!("cut-{keep}.dgram"));
        fs::write(&cut, &whole[..keep])?;
        let loaded = load(&cut);
        assert!(
            matches!(
                loaded,
                Err(LoadError::Damaged { .. } | LoadError::NotAProject { .. })
            ),
            "cut at {keep}: {loaded:?}"
        );
    }
    Ok(())
}

/// Saves the project, changes the file with `sql` (checks and foreign keys
/// off, as a hand edit or another program might), and loads it.
struct Tampered {
    result: Result<(), LoadError>,
    before: Vec<u8>,
    after: Vec<u8>,
}

fn tampered(sql: &str) -> Result<Tampered, Box<dyn Error>> {
    let (_dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let connection = Connection::open(&path)?;
    connection.execute_batch(&format!(
        "PRAGMA foreign_keys = OFF; PRAGMA ignore_check_constraints = ON; {sql}"
    ))?;
    connection.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);

    let result = load(&path).map(|_| ());

    Ok(Tampered {
        result,
        before,
        after: bytes(&path),
    })
}

fn assert_invalid(sql: &str, expected: &str) -> Result<(), Box<dyn Error>> {
    let Tampered {
        result,
        before,
        after,
    } = tampered(sql)?;
    match result {
        Err(LoadError::Invalid(problems)) => assert!(
            problems.iter().any(|problem| problem.contains(expected)),
            "{sql}: {problems:?} does not mention {expected:?}"
        ),
        other => panic_free_fail(sql, &other),
    }
    assert_eq!(before, after, "a refused file is not rewritten");
    Ok(())
}

fn panic_free_fail(sql: &str, other: &Result<(), LoadError>) {
    assert!(
        matches!(other, Err(LoadError::Invalid(_))),
        "{sql}: expected invalid content, got {other:?}"
    );
}

#[test]
fn a_view_of_a_missing_class_should_be_refused_with_the_reason() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE node_views SET element_id = 999999 WHERE id = (SELECT min(id) FROM node_views);",
        "missing",
    )
}

#[test]
fn a_blank_class_name_should_be_refused() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE elements SET name = '   ' WHERE id = (SELECT min(id) FROM elements);",
        "blank name",
    )
}

#[test]
fn an_invalid_multiplicity_should_be_refused() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE relations SET target_multiplicity = 'many' WHERE kind = 'association';",
        "not a multiplicity",
    )
}

#[test]
fn a_class_that_is_its_own_parent_should_be_refused() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE relations SET target = source WHERE kind = 'generalization';",
        "own parent",
    )
}

#[test]
fn an_unknown_visibility_or_kind_should_be_refused() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE members SET visibility = 'secret' WHERE position = 0;",
        "visibility",
    )?;
    assert_invalid(
        "UPDATE relations SET kind = 'aggregation' WHERE kind = 'association';",
        "unknown kind",
    )
}

#[test]
fn a_view_with_an_impossible_size_should_be_refused() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE node_views SET width = -5 WHERE position = 0;",
        "view",
    )
}

#[test]
fn an_id_used_twice_should_be_refused() -> Result<(), Box<dyn Error>> {
    assert_invalid(
        "UPDATE diagrams SET id = (SELECT min(id) FROM elements) WHERE position = 1;",
        "twice",
    )
}

#[test]
fn an_id_counter_below_the_largest_id_should_be_damaged() -> Result<(), Box<dyn Error>> {
    let Tampered {
        result,
        before,
        after,
    } = tampered("UPDATE meta SET value = '1' WHERE key = 'next_id';")?;

    assert!(
        matches!(result, Err(LoadError::Damaged { .. })),
        "{result:?}"
    );
    assert!(
        result
            .err()
            .is_some_and(|error| error.to_string().contains("reused"))
    );
    assert_eq!(before, after);
    Ok(())
}

#[test]
fn a_missing_table_should_be_damaged() -> Result<(), Box<dyn Error>> {
    let Tampered { result, .. } = tampered("DROP TABLE node_views;")?;

    assert!(
        matches!(result, Err(LoadError::Damaged { .. })),
        "{result:?}"
    );
    Ok(())
}

#[test]
fn a_loaded_document_should_pass_the_core_checks_and_allow_further_commands()
-> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let mut document = load(&path)?.document;
    let Some(customer) = element(&document, "Customer") else {
        return Err("Customer is missing".into());
    };

    let renamed = command::execute(
        &mut document,
        &Command::RenameElement {
            element: customer,
            name: String::from("Client"),
        },
    );

    assert!(renamed.is_ok());
    assert!(
        diagram_core::validate::check_document(&document)
            .iter()
            .all(|issue| issue.severity != diagram_core::validate::Severity::Error)
    );
    assert_eq!(ElementId::from_raw(customer.raw()), customer);
    Ok(())
}

#[test]
fn a_write_ahead_log_file_should_be_refused_without_creating_sidecars() -> Result<(), Box<dyn Error>>
{
    let (dir, path) = folder()?;
    save(&path, "Shop", &project())?;
    let writer = Connection::open(&path)?;
    let _mode: String = writer.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    writer.close().map_err(|(_, error)| error)?;
    let before = bytes(&path);
    let names_before = common::names_in(dir.path());

    let loaded = load(&path);

    assert!(
        matches!(loaded, Err(LoadError::Damaged { .. })),
        "{loaded:?}"
    );
    assert!(loaded.is_err_and(|error| error.to_string().contains("journal")));
    assert_eq!(bytes(&path), before);
    assert_eq!(
        common::names_in(dir.path()),
        names_before,
        "no -wal or -shm was created"
    );
    Ok(())
}

#[test]
fn a_table_with_the_wrong_columns_should_be_damaged() -> Result<(), Box<dyn Error>> {
    let Tampered { result, .. } = tampered("ALTER TABLE node_views RENAME COLUMN width TO w;")?;

    assert!(
        matches!(result, Err(LoadError::Damaged { .. })),
        "{result:?}"
    );
    Ok(())
}

fn rows() -> ProjectDto {
    ProjectDto::from_document("Shop", &project())
}

fn assert_rows_invalid(dto: ProjectDto, expected: &str) {
    let outcome = dto.into_document();
    match outcome {
        Err(LoadError::Invalid(problems)) => assert!(
            problems.iter().any(|problem| problem.contains(expected)),
            "{problems:?} does not mention {expected:?}"
        ),
        other => assert!(
            matches!(other, Err(LoadError::Invalid(_))),
            "expected invalid rows for {expected:?}, got {:?}",
            other.map(|(name, _)| name)
        ),
    }
}

#[test]
fn rows_with_a_position_used_twice_should_be_refused() {
    let mut members = rows();
    if let Some(second) = members.members.get(1).cloned() {
        members.members[0].element_id = second.element_id;
        members.members[0].position = second.position;
    }
    assert_rows_invalid(members, "share the position");

    let mut views = rows();
    if views.nodes.len() > 1 && views.nodes[0].diagram_id == views.nodes[1].diagram_id {
        views.nodes[1].position = views.nodes[0].position;
    }
    assert_rows_invalid(views, "share the position");

    let mut diagrams = rows();
    if diagrams.diagrams.len() > 1 {
        diagrams.diagrams[1].position = diagrams.diagrams[0].position;
    }
    assert_rows_invalid(diagrams, "share the position");
}

#[test]
fn a_class_shown_twice_in_one_diagram_should_be_refused() {
    let mut dto = rows();
    if let Some(first) = dto.nodes.first().cloned() {
        let mut copy = first;
        copy.id = dto.next_id + 1;
        copy.position = 99;
        dto.next_id += 1;
        dto.nodes.push(copy);
    }
    assert_rows_invalid(dto, "shown twice");
}

#[test]
fn a_repeated_generalization_and_ends_on_a_generalization_should_be_refused() {
    let mut repeated = rows();
    if let Some(general) = repeated
        .relations
        .iter()
        .find(|relation| relation.kind == "generalization")
        .cloned()
    {
        let mut copy = general;
        copy.id = repeated.next_id + 1;
        repeated.next_id += 1;
        repeated.relations.push(copy);
    }
    assert_rows_invalid(repeated, "more than once");

    let mut ends = rows();
    for relation in &mut ends.relations {
        if relation.kind == "generalization" {
            relation.source_role = Some(String::from("child"));
        }
    }
    assert_rows_invalid(ends, "only associations have");
}

#[test]
fn parameters_on_an_attribute_or_a_missing_member_should_be_refused() {
    let mut dto = rows();
    if let Some(attribute) = dto
        .members
        .iter()
        .find(|row| row.kind == "attribute")
        .cloned()
    {
        dto.parameters.push(diagram_storage::dto::ParameterRow {
            member_id: attribute.id,
            position: 0,
            name: String::from("x"),
            type_text: String::new(),
        });
    }
    assert_rows_invalid(dto, "not an operation");

    let mut orphan = rows();
    orphan.parameters.push(diagram_storage::dto::ParameterRow {
        member_id: 424_242,
        position: 0,
        name: String::from("x"),
        type_text: String::new(),
    });
    assert_rows_invalid(orphan, "missing operation");
}

#[test]
fn the_id_counter_should_be_kept_verbatim_after_deleting_the_highest_ids()
-> Result<(), Box<dyn Error>> {
    let (_dir, path) = folder()?;
    let mut document = project();
    let diagram = document.diagram_order[0];
    run(
        &mut document,
        Command::CreateClass {
            diagram,
            name: String::from("Temporary"),
            bounds: rect(600.0, 40.0),
        },
    );
    let Some(temporary) = element(&document, "Temporary") else {
        return Err("the class was not created".into());
    };
    let highest = temporary.raw();
    run(&mut document, Command::DeleteElement { element: temporary });
    assert!(
        document.next_id >= highest,
        "deleting does not give IDs back"
    );

    save(&path, "Shop", &document)?;
    let mut loaded = load(&path)?.document;
    assert_eq!(
        loaded.next_id, document.next_id,
        "stored verbatim, not recomputed"
    );
    run(
        &mut loaded,
        Command::CreateClass {
            diagram,
            name: String::from("Next"),
            bounds: rect(600.0, 40.0),
        },
    );

    let next = element(&loaded, "Next")
        .map(|id| id.raw())
        .unwrap_or_default();
    assert!(
        next > highest,
        "{next} reuses the deleted class's ID {highest}"
    );
    Ok(())
}

#[test]
fn an_id_counter_above_the_signed_range_should_be_damaged() {
    let mut dto = rows();
    dto.next_id = u64::MAX;

    let outcome = dto.into_document();

    assert!(
        matches!(&outcome, Err(LoadError::Damaged { reason }) if reason.contains("too large")),
        "{:?}",
        outcome.map(|(name, _)| name)
    );
}
