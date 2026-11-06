//! The open project: the authoritative document with its undo history.
//!
//! [`ProjectSession`] wraps [`diagram_core::history::Session`] so the UI has
//! one place to execute commands, undo, redo and ask whether work would be
//! lost. Nothing here draws or knows about egui. The window reads the
//! document through [`ProjectSession::document`] and changes it only by
//! executing a [`Command`] (R2-03, UI-13).

use diagram_core::command::{self, Command, EndInput};
use diagram_core::history::{Outcome, Revision, Session, SessionId};
use diagram_core::ids::ElementId;
use diagram_core::model::{Document, Name, Visibility};
use diagram_core::notation::RelationKind;
use diagram_core::validate::DomainError;
use diagram_core::{Point, Rect, Size};

/// Name of the diagram every new project starts with.
pub const FIRST_DIAGRAM: &str = "Class diagram 1";

/// Name of the built-in sample project.
pub const SAMPLE_PROJECT: &str = "Sample project";

/// Where the open project came from, which decides when closing it needs a prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    /// Created in this run and never saved.
    Fresh,
    /// Opened from a file.
    File,
}

/// An open project: name, document, history and save identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSession {
    name: String,
    origin: Origin,
    session: Session,
}

/// Source of session identities, so a late result for an old session is recognisable.
fn next_session_id() -> SessionId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    SessionId::from_raw(NEXT.fetch_add(1, Ordering::Relaxed))
}

impl ProjectSession {
    /// A new, empty project with one empty class diagram.
    pub fn blank(name: &str) -> Self {
        // FIRST_DIAGRAM is not blank; the empty fallback only keeps this total.
        let document = Name::new(FIRST_DIAGRAM)
            .map(|diagram| Document::with_diagram(diagram).0)
            .unwrap_or_default();
        Self::from_document(name, document, Origin::Fresh)
    }

    /// The labelled sample project (Customer and Order), built through the
    /// same commands a user's edits use, with an empty undo history.
    ///
    /// If the core ever refuses a sample command, the sample is an empty
    /// project (and `build_sample`'s test fails), never a half-built one.
    pub fn sample() -> Self {
        let document = build_sample().unwrap_or_default();
        Self::from_document(SAMPLE_PROJECT, document, Origin::Fresh)
    }

    fn from_document(name: &str, document: Document, origin: Origin) -> Self {
        let session = Session::new(next_session_id(), document, origin == Origin::File);
        Self {
            name: name.to_owned(),
            origin,
            session,
        }
    }

    /// The project name shown in the header.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The authoritative document, read-only.
    pub const fn document(&self) -> &Document {
        self.session.document()
    }

    /// Current revision; it grows with every edit, undo and redo.
    pub const fn revision(&self) -> Revision {
        self.session.revision()
    }

    /// Validates and applies `command` as one undoable action.
    ///
    /// # Errors
    ///
    /// Returns the domain error when the command is invalid; the document is
    /// then unchanged.
    pub fn execute(&mut self, command: &Command) -> Result<Outcome, DomainError> {
        self.session.execute(command)
    }

    /// Reverses the last action, if there is one.
    pub fn undo(&mut self) -> Option<Outcome> {
        self.session.undo()
    }

    /// Repeats the last undone action, if there is one.
    pub fn redo(&mut self) -> Option<Outcome> {
        self.session.redo()
    }

    /// Label of the action Undo would reverse.
    pub fn undo_label(&self) -> Option<&'static str> {
        self.session.undo_label()
    }

    /// Label of the action Redo would repeat.
    pub fn redo_label(&self) -> Option<&'static str> {
        self.session.redo_label()
    }

    /// Whether closing the project would lose work the user made. A new
    /// project nobody has edited has nothing to lose.
    pub fn has_unsaved_changes(&self) -> bool {
        match self.origin {
            Origin::Fresh => self.session.revision().raw() > 0,
            Origin::File => self.session.is_dirty(),
        }
    }
}

/// Builds the sample document by executing commands on an empty document.
///
/// # Errors
///
/// Returns the first command the core refuses; the fixed sample commands are
/// valid, so a failure means the core rules changed.
fn build_sample() -> Result<Document, DomainError> {
    let first = Name::new(FIRST_DIAGRAM).map_err(DomainError::from)?;
    let (mut document, diagram) = Document::with_diagram(first);
    let at = |x: f64, y: f64| {
        Rect::new(
            Point::new(x, y).unwrap_or(Point::ORIGIN),
            Size::new(180.0, 80.0).unwrap_or(Size::UNIT),
        )
    };
    let class = |document: &mut Document, name: &str, x: f64, y: f64| {
        command::execute(
            document,
            &Command::CreateClass {
                diagram,
                name: name.to_owned(),
                bounds: at(x, y),
            },
        )
    };
    class(&mut document, "Customer", 48.0, 48.0)?;
    class(&mut document, "Order", 300.0, 96.0)?;
    let find = |document: &Document, wanted: &str| {
        document
            .elements
            .values()
            .find(|element| element.name.as_str() == wanted)
            .map(|element| element.id)
    };
    let (Some(customer), Some(order)) = (find(&document, "Customer"), find(&document, "Order"))
    else {
        return Err(DomainError::MissingElement(ElementId::from_raw(0)));
    };
    for (element, name, type_text) in [
        (customer, "id", "CustomerId"),
        (customer, "name", "String"),
        (order, "id", "OrderId"),
        (order, "total", "Money"),
    ] {
        command::execute(
            &mut document,
            &Command::AddAttribute {
                element,
                name: name.to_owned(),
                type_text: type_text.to_owned(),
                visibility: Visibility::Public,
                multiplicity: String::new(),
            },
        )?;
    }
    let views = document.diagrams.get(&diagram).map(|diagram| {
        (
            diagram.node_of(customer).map(|node| node.id),
            diagram.node_of(order).map(|node| node.id),
        )
    });
    let Some((Some(source), Some(target))) = views else {
        return Err(DomainError::MissingElement(customer));
    };
    command::execute(
        &mut document,
        &Command::Connect {
            diagram,
            kind: RelationKind::Association,
            source,
            target,
        },
    )?;
    let Some(relation) = document.relations.keys().next().copied() else {
        return Err(DomainError::MissingElement(customer));
    };
    command::execute(
        &mut document,
        &Command::SetEnds {
            relation,
            source: EndInput {
                role: "customer".to_owned(),
                multiplicity: "1".to_owned(),
            },
            target: EndInput {
                role: "orders".to_owned(),
                multiplicity: "0..*".to_owned(),
            },
        },
    )?;
    command::execute(
        &mut document,
        &Command::AddDiagram {
            name: "Order details".to_owned(),
        },
    )?;
    let Some(second) = document.diagram_order.last().copied() else {
        return Err(DomainError::MissingElement(order));
    };
    command::execute(
        &mut document,
        &Command::ShowInDiagram {
            element: order,
            diagram: second,
            bounds: at(48.0, 48.0),
        },
    )?;
    Ok(document)
}
