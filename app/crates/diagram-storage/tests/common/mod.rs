//! Shared fixtures: a project built only through public core commands, plus
//! small file helpers.

#![allow(dead_code, reason = "each test file uses a different subset")]

use std::fs;
use std::path::{Path, PathBuf};

use diagram_core::command::{self, Command, EndInput};
use diagram_core::ids::ElementId;
use diagram_core::model::{Document, Member, Name, Parameter, Visibility};
use diagram_core::notation::RelationKind;
use diagram_core::{Point, Rect, Size};

/// A rectangle from literal values.
pub fn rect(x: f64, y: f64) -> Rect {
    Rect::new(
        Point::new(x, y).unwrap_or(Point::ORIGIN),
        Size::new(180.0, 80.0).unwrap_or(Size::UNIT),
    )
}

/// Executes `command`, failing the test if the core refuses it.
pub fn run(document: &mut Document, command: Command) {
    let outcome = command::execute(document, &command);
    assert!(outcome.is_ok(), "{command:?} was refused: {outcome:?}");
}

/// The first element with this name.
pub fn element(document: &Document, name: &str) -> Option<ElementId> {
    document
        .elements
        .values()
        .find(|element| element.name.as_str() == name)
        .map(|element| element.id)
}

/// Customer, Order and an Arabic-named subclass; attributes, an operation
/// with parameters, an association with roles and multiplicities, a
/// generalization, and a second diagram showing one class.
pub fn project() -> Document {
    let first = Name::new("Class diagram 1");
    assert!(first.is_ok());
    let Ok(first) = first else {
        return Document::default();
    };
    let (mut document, diagram) = Document::with_diagram(first);
    for (name, x, y) in [
        ("Customer", 48.0, 48.0),
        ("Order", 300.0, 96.0),
        ("عميل مميز", 48.0, 260.0),
    ] {
        run(
            &mut document,
            Command::CreateClass {
                diagram,
                name: name.to_owned(),
                bounds: rect(x, y),
            },
        );
    }
    let (Some(customer), Some(order), Some(premium)) = (
        element(&document, "Customer"),
        element(&document, "Order"),
        element(&document, "عميل مميز"),
    ) else {
        return document;
    };
    for (owner, name, type_text, multiplicity) in [
        (customer, "id", "CustomerId", ""),
        (customer, "email", "Text", "0..1"),
        (order, "total", "Money", ""),
    ] {
        run(
            &mut document,
            Command::AddAttribute {
                element: owner,
                name: name.to_owned(),
                type_text: type_text.to_owned(),
                visibility: Visibility::Private,
                multiplicity: multiplicity.to_owned(),
            },
        );
    }
    run(
        &mut document,
        Command::AddOperation {
            element: order,
            name: String::from("place"),
            visibility: Visibility::Public,
            return_type: String::from("Receipt"),
        },
    );
    // Parameters have no command yet, so add them to the operation directly.
    if let Some(order_element) = document.elements.get_mut(&order) {
        for member in &mut order_element.members {
            if let Member::Operation(operation) = member {
                for (name, type_text) in [("customer", "Customer"), ("note", "")] {
                    if let Ok(name) = Name::new(name) {
                        operation.parameters.push(Parameter {
                            name,
                            type_text: type_text.to_owned(),
                        });
                    }
                }
            }
        }
    }
    let view_of = |document: &Document, element: ElementId| {
        document
            .diagrams
            .get(&diagram)
            .and_then(|diagram| diagram.node_of(element))
            .map(|node| node.id)
    };
    let (Some(customer_view), Some(order_view), Some(premium_view)) = (
        view_of(&document, customer),
        view_of(&document, order),
        view_of(&document, premium),
    ) else {
        return document;
    };
    run(
        &mut document,
        Command::Connect {
            diagram,
            kind: RelationKind::Association,
            source: customer_view,
            target: order_view,
        },
    );
    let association = document.relations.keys().next().copied();
    if let Some(relation) = association {
        run(
            &mut document,
            Command::SetEnds {
                relation,
                source: EndInput {
                    role: String::from("buyer"),
                    multiplicity: String::from("1"),
                },
                target: EndInput {
                    role: String::from("purchases"),
                    multiplicity: String::from("0..*"),
                },
            },
        );
    }
    run(
        &mut document,
        Command::Connect {
            diagram,
            kind: RelationKind::Generalization,
            source: premium_view,
            target: customer_view,
        },
    );
    run(
        &mut document,
        Command::AddDiagram {
            name: String::from("Order details"),
        },
    );
    if let Some(second) = document.diagram_order.last().copied() {
        run(
            &mut document,
            Command::ShowInDiagram {
                element: order,
                diagram: second,
                bounds: rect(20.0, 30.0),
            },
        );
    }
    document
}

/// A second, different project (for "the old file survives a failed save").
pub fn other_project() -> Document {
    let mut document = project();
    if let Some(customer) = element(&document, "Customer") {
        run(
            &mut document,
            Command::RenameElement {
                element: customer,
                name: String::from("Client"),
            },
        );
    }
    document
}

/// The bytes of a file.
pub fn bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_default()
}

/// Names of the entries in a folder, sorted.
pub fn names_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// A fresh folder and a project path inside it.
///
/// # Errors
///
/// Returns the I/O error if the folder cannot be created.
pub fn folder() -> std::io::Result<(tempfile::TempDir, PathBuf)> {
    let folder = tempfile::tempdir()?;
    let path = folder.path().join("shop.dgram");
    Ok((folder, path))
}
