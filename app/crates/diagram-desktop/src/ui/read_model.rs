//! Read model: pure functions that turn a [`Document`] into the rows the
//! project panel, canvas and inspector show (UI-13).
//!
//! Nothing here draws, and nothing changes the document. Every row carries
//! the typed ID of the record it shows, so a click can name its target
//! without a list index. If a record is deleted, the next frame simply has
//! no row for it.

use diagram_core::ids::{DiagramId, ElementId, RelationId};
use diagram_core::model::{
    Attribute, Document, Member, Multiplicity, Operation, Relation, Visibility,
};
use diagram_core::notation::{NotationId, RelationKind};

/// A diagram in the project panel and on a tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramRow {
    /// The diagram.
    pub id: DiagramId,
    /// Its name.
    pub name: String,
    /// Its notation, which decides its tools and inspector sections.
    pub notation: NotationId,
}

/// A model element (class) in the project panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementRow {
    /// The element.
    pub id: ElementId,
    /// Its name.
    pub name: String,
}

/// A relationship in the project panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationRow {
    /// The relation.
    pub id: RelationId,
    /// "Customer – Order (association)".
    pub label: String,
}

/// A class box to draw: where it is and the text lines it shows.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassCard {
    /// The element shown.
    pub element: ElementId,
    /// Class name.
    pub name: String,
    /// One line per member, attributes and operations in member order.
    pub lines: Vec<String>,
    /// Top-left corner in the diagram's world units.
    pub position: (f32, f32),
}

/// A connector to draw between two class cards of the same diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeCard {
    /// The relation shown.
    pub relation: RelationId,
    /// Element at the source end (the child for a generalization).
    pub source: ElementId,
    /// Element at the target end (the parent for a generalization).
    pub target: ElementId,
    /// Association or generalization.
    pub kind: RelationKind,
    /// Multiplicity text at the source end, empty if none.
    pub source_multiplicity: String,
    /// Multiplicity text at the target end, empty if none.
    pub target_multiplicity: String,
}

/// Everything the canvas draws for one diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanvasModel {
    /// Class boxes in z-order (the last is on top).
    pub classes: Vec<ClassCard>,
    /// Connectors.
    pub edges: Vec<EdgeCard>,
}

/// Diagrams in project order.
pub fn diagram_rows(document: &Document) -> Vec<DiagramRow> {
    document
        .diagram_order
        .iter()
        .filter_map(|id| document.diagrams.get(id))
        .map(|diagram| DiagramRow {
            id: diagram.id,
            name: diagram.name.as_str().to_owned(),
            notation: diagram.notation,
        })
        .collect()
}

/// Model elements in creation order.
pub fn element_rows(document: &Document) -> Vec<ElementRow> {
    document
        .elements
        .values()
        .map(|element| ElementRow {
            id: element.id,
            name: element.name.as_str().to_owned(),
        })
        .collect()
}

/// Relationships in creation order, with a readable label.
pub fn relation_rows(document: &Document) -> Vec<RelationRow> {
    document
        .relations
        .values()
        .map(|relation| RelationRow {
            id: relation.id,
            label: relation_label(document, relation),
        })
        .collect()
}

/// The name of `element`, if it exists.
pub fn element_name(document: &Document, element: ElementId) -> Option<&str> {
    document
        .elements
        .get(&element)
        .map(|element| element.name.as_str())
}

/// "Customer – Order (association)" or "Dog – Animal (generalization)".
pub fn relation_label(document: &Document, relation: &Relation) -> String {
    let name = |element| element_name(document, element).unwrap_or("?");
    format!(
        "{} – {} ({})",
        name(relation.source),
        name(relation.target),
        kind_word(relation.kind)
    )
}

/// The kind in words.
pub const fn kind_word(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::Association => "association",
        RelationKind::Generalization => "generalization",
    }
}

/// Diagrams that show `element`, in project order.
pub fn diagrams_showing(document: &Document, element: ElementId) -> Vec<DiagramId> {
    document
        .diagram_order
        .iter()
        .filter(|id| {
            document
                .diagrams
                .get(id)
                .is_some_and(|diagram| diagram.node_of(element).is_some())
        })
        .copied()
        .collect()
}

/// Model-versus-view wording for a class: which diagrams show it.
pub fn shown_in(document: &Document, element: ElementId) -> String {
    let names: Vec<&str> = diagrams_showing(document, element)
        .into_iter()
        .filter_map(|id| document.diagrams.get(&id))
        .map(|diagram| diagram.name.as_str())
        .collect();
    match names.as_slice() {
        [] => String::from("Not shown in any diagram"),
        [only] => format!("Shown in {only}"),
        [first @ .., last] => format!("Shown in {} and {last}", first.join(", ")),
    }
}

/// Elements shown in `diagram`, in z-order.
pub fn elements_in(document: &Document, diagram: DiagramId) -> Vec<ElementId> {
    document
        .diagrams
        .get(&diagram)
        .map(|diagram| diagram.nodes.iter().map(|node| node.element).collect())
        .unwrap_or_default()
}

/// What the canvas draws for `diagram`; empty if the diagram does not exist.
pub fn canvas_model(document: &Document, diagram: DiagramId) -> CanvasModel {
    let Some(diagram) = document.diagrams.get(&diagram) else {
        return CanvasModel::default();
    };
    let classes = diagram
        .nodes
        .iter()
        .filter_map(|node| {
            let element = document.elements.get(&node.element)?;
            Some(ClassCard {
                element: element.id,
                name: element.name.as_str().to_owned(),
                lines: element.members.iter().map(member_line).collect(),
                position: (node.bounds.left() as f32, node.bounds.top() as f32),
            })
        })
        .collect();
    let edges = diagram
        .edges
        .iter()
        .filter_map(|edge| {
            let relation = document.relations.get(&edge.relation)?;
            Some(EdgeCard {
                relation: relation.id,
                source: relation.source,
                target: relation.target,
                kind: relation.kind,
                source_multiplicity: multiplicity_text(relation.source_end.multiplicity),
                target_multiplicity: multiplicity_text(relation.target_end.multiplicity),
            })
        })
        .collect();
    CanvasModel { classes, edges }
}

/// Multiplicity as text; empty when there is none.
pub fn multiplicity_text(multiplicity: Option<Multiplicity>) -> String {
    multiplicity
        .map(|value| value.to_string())
        .unwrap_or_default()
}

/// One line of a class box for a member.
pub fn member_line(member: &Member) -> String {
    match member {
        Member::Attribute(attribute) => attribute_line(attribute),
        Member::Operation(operation) => operation_line(operation),
    }
}

/// "id: CustomerId". A visibility mark is shown only when it is not public.
pub fn attribute_line(attribute: &Attribute) -> String {
    let mut line = format!(
        "{}{}",
        visibility_mark(attribute.visibility),
        attribute.name
    );
    if !attribute.type_text.is_empty() {
        line.push_str(": ");
        line.push_str(&attribute.type_text);
    }
    if let Some(multiplicity) = attribute.multiplicity {
        line.push_str(&format!(" [{multiplicity}]"));
    }
    line
}

/// "place(id: OrderId): Order".
pub fn operation_line(operation: &Operation) -> String {
    let parameters: Vec<String> = operation
        .parameters
        .iter()
        .map(|parameter| {
            if parameter.type_text.is_empty() {
                parameter.name.to_string()
            } else {
                format!("{}: {}", parameter.name, parameter.type_text)
            }
        })
        .collect();
    let mut line = format!(
        "{}{}({})",
        visibility_mark(operation.visibility),
        operation.name,
        parameters.join(", ")
    );
    if let Some(return_type) = operation
        .return_type
        .as_deref()
        .filter(|text| !text.is_empty())
    {
        line.push_str(": ");
        line.push_str(return_type);
    }
    line
}

fn visibility_mark(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "",
        Visibility::Private => "- ",
        Visibility::Protected => "# ",
        Visibility::Package => "~ ",
    }
}

/// Attribute lines of an element, for the inspector.
pub fn attribute_lines(document: &Document, element: ElementId) -> Vec<String> {
    document
        .elements
        .get(&element)
        .map(|element| {
            element
                .members
                .iter()
                .filter_map(|member| match member {
                    Member::Attribute(attribute) => Some(attribute_line(attribute)),
                    Member::Operation(_) => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Operation lines of an element, for the inspector.
pub fn operation_lines(document: &Document, element: ElementId) -> Vec<String> {
    document
        .elements
        .get(&element)
        .map(|element| {
            element
                .members
                .iter()
                .filter_map(|member| match member {
                    Member::Operation(operation) => Some(operation_line(operation)),
                    Member::Attribute(_) => None,
                })
                .collect()
        })
        .unwrap_or_default()
}
