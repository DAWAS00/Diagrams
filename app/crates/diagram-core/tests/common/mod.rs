//! Shared fixture: the Customer / Order / PreferredCustomer project from the
//! M2 acceptance walkthrough, built only through public commands.

#![allow(dead_code, reason = "each test file uses a different subset")]

use diagram_core::command::Command;
use diagram_core::history::{Session, SessionId};
use diagram_core::ids::{DiagramId, ElementId, ViewId};
use diagram_core::model::{Document, Name};
use diagram_core::{Point, Rect, Size};

/// IDs of the fixture's records.
pub struct Ids {
    /// The diagram.
    pub diagram: DiagramId,
    /// Customer, Order and PreferredCustomer.
    pub customer: ElementId,
    /// Order.
    pub order: ElementId,
    /// PreferredCustomer.
    pub preferred: ElementId,
    /// Views of the three classes.
    pub customer_view: ViewId,
    /// Order's view.
    pub order_view: ViewId,
    /// PreferredCustomer's view.
    pub preferred_view: ViewId,
}

/// A rectangle from literal values (every literal used here is valid).
pub fn rect(x: f64, y: f64) -> Rect {
    Rect::from_xywh(x, y, 180.0, 80.0).unwrap_or(Rect::new(Point::ORIGIN, Size::UNIT))
}

/// A point from literal values.
pub fn point(x: f64, y: f64) -> Point {
    Point::new(x, y).unwrap_or(Point::ORIGIN)
}

/// An empty document with one diagram.
pub fn empty() -> (Document, DiagramId) {
    let name = Name::new("Class diagram 1").unwrap_or_else(|_| fallback_name());
    Document::with_diagram(name)
}

fn fallback_name() -> Name {
    // Never used: the literal above is not blank.
    Name::new("Diagram").unwrap_or_else(|_| fallback_name())
}

/// Adds a class through the public command and returns its element and view.
pub fn add_class(
    session: &mut Session,
    diagram: DiagramId,
    name: &str,
    x: f64,
) -> (ElementId, ViewId) {
    let result = session.execute(&Command::CreateClass {
        diagram,
        name: name.to_owned(),
        bounds: rect(x, 40.0),
    });
    assert!(result.is_ok(), "{result:?}");
    let diagram = &session.document().diagrams[&diagram];
    let node = diagram.nodes.last();
    node.map(|node| (node.element, node.id))
        .unwrap_or((ElementId::from_raw(0), ViewId::from_raw(0)))
}

/// A new (unsaved) session with the three fixture classes.
pub fn three_classes() -> (Session, Ids) {
    let (document, diagram) = empty();
    let mut session = Session::new(SessionId::from_raw(1), document, false);
    let (customer, customer_view) = add_class(&mut session, diagram, "Customer", 40.0);
    let (order, order_view) = add_class(&mut session, diagram, "Order", 300.0);
    let (preferred, preferred_view) = add_class(&mut session, diagram, "PreferredCustomer", 560.0);
    (
        session,
        Ids {
            diagram,
            customer,
            order,
            preferred,
            customer_view,
            order_view,
            preferred_view,
        },
    )
}
