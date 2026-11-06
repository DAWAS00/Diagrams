//! Data transfer objects for schema v1: plain rows that mirror the tables,
//! kept apart from the in-memory model.
//!
//! [`ProjectDto::from_document`] copies the model out. `ProjectDto::into_document`
//! (in `rebuild.rs`) rebuilds it through the core's own rules (`Name`, `Multiplicity`, `Rect`,
//! `check_document`), so a damaged or tampered file can never put an invalid
//! document into live state.

use diagram_core::model::{AssociationEnd, Document, Member, Visibility};
use diagram_core::notation::{NotationId, RelationKind};

/// A class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementRow {
    /// Identity.
    pub id: u64,
    /// Name.
    pub name: String,
}

/// An attribute or operation, in display order within its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRow {
    /// Identity.
    pub id: u64,
    /// Owning class.
    pub element_id: u64,
    /// Position within the class.
    pub position: u32,
    /// `attribute` or `operation`.
    pub kind: String,
    /// Name.
    pub name: String,
    /// `public`, `private`, `protected` or `package`.
    pub visibility: String,
    /// Attribute type text.
    pub type_text: Option<String>,
    /// Attribute multiplicity text.
    pub multiplicity: Option<String>,
    /// Operation return type text.
    pub return_type: Option<String>,
}

/// A parameter of an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterRow {
    /// Owning operation.
    pub member_id: u64,
    /// Position in the parameter list.
    pub position: u32,
    /// Name.
    pub name: String,
    /// Type text.
    pub type_text: String,
}

/// A relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationRow {
    /// Identity.
    pub id: u64,
    /// `association` or `generalization`.
    pub kind: String,
    /// Source class (the child of a generalization).
    pub source: u64,
    /// Target class (the parent of a generalization).
    pub target: u64,
    /// Role at the source end.
    pub source_role: Option<String>,
    /// Multiplicity at the source end.
    pub source_multiplicity: Option<String>,
    /// Role at the target end.
    pub target_role: Option<String>,
    /// Multiplicity at the target end.
    pub target_multiplicity: Option<String>,
}

/// A diagram, in project order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramRow {
    /// Identity.
    pub id: u64,
    /// Position in the project.
    pub position: u32,
    /// Name.
    pub name: String,
    /// `class`, `flowchart`, `use_case`, `entity_relationship` or `sequence`.
    pub notation: String,
}

/// A class shown in a diagram, in z-order.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeRow {
    /// Identity.
    pub id: u64,
    /// Diagram it is in.
    pub diagram_id: u64,
    /// Z-order position (the last is on top).
    pub position: u32,
    /// Class shown.
    pub element_id: u64,
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// A connector in a diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeRow {
    /// Identity.
    pub id: u64,
    /// Diagram it is in.
    pub diagram_id: u64,
    /// Position among the diagram's connectors.
    pub position: u32,
    /// Relationship shown.
    pub relation_id: u64,
    /// Node view at the source end.
    pub source_view: u64,
    /// Node view at the target end.
    pub target_view: u64,
}

/// A whole project as rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectDto {
    /// Project name.
    pub name: String,
    /// The next ID the document will allocate.
    pub next_id: u64,
    /// Classes.
    pub elements: Vec<ElementRow>,
    /// Members.
    pub members: Vec<MemberRow>,
    /// Operation parameters.
    pub parameters: Vec<ParameterRow>,
    /// Relationships.
    pub relations: Vec<RelationRow>,
    /// Diagrams.
    pub diagrams: Vec<DiagramRow>,
    /// Node views.
    pub nodes: Vec<NodeRow>,
    /// Edge views.
    pub edges: Vec<EdgeRow>,
}

const fn visibility_text(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "public",
        Visibility::Private => "private",
        Visibility::Protected => "protected",
        Visibility::Package => "package",
    }
}

pub(crate) fn parse_visibility(text: &str) -> Option<Visibility> {
    Some(match text {
        "public" => Visibility::Public,
        "private" => Visibility::Private,
        "protected" => Visibility::Protected,
        "package" => Visibility::Package,
        _ => return None,
    })
}

const fn kind_text(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::Association => "association",
        RelationKind::Generalization => "generalization",
    }
}

pub(crate) fn parse_kind(text: &str) -> Option<RelationKind> {
    match text {
        "association" => Some(RelationKind::Association),
        "generalization" => Some(RelationKind::Generalization),
        _ => None,
    }
}

const fn notation_text(notation: NotationId) -> &'static str {
    match notation {
        NotationId::Class => "class",
        NotationId::Flowchart => "flowchart",
        NotationId::UseCase => "use_case",
        NotationId::EntityRelationship => "entity_relationship",
        NotationId::Sequence => "sequence",
    }
}

pub(crate) fn parse_notation(text: &str) -> Option<NotationId> {
    Some(match text {
        "class" => NotationId::Class,
        "flowchart" => NotationId::Flowchart,
        "use_case" => NotationId::UseCase,
        "entity_relationship" => NotationId::EntityRelationship,
        "sequence" => NotationId::Sequence,
        _ => return None,
    })
}

fn position(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

fn role(end: &AssociationEnd) -> Option<String> {
    end.role.as_ref().map(|name| name.as_str().to_owned())
}

fn multiplicity(end: &AssociationEnd) -> Option<String> {
    end.multiplicity.map(|value| value.to_string())
}

impl ProjectDto {
    /// Copies `document` out as rows. Elements, relations and views come out
    /// in ID order; members, diagrams, views and parameters keep their order.
    pub fn from_document(name: &str, document: &Document) -> Self {
        let mut dto = Self {
            name: name.to_owned(),
            next_id: document.next_id,
            elements: Vec::new(),
            members: Vec::new(),
            parameters: Vec::new(),
            relations: Vec::new(),
            diagrams: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        for element in document.elements.values() {
            dto.elements.push(ElementRow {
                id: element.id.raw(),
                name: element.name.as_str().to_owned(),
            });
            for (index, member) in element.members.iter().enumerate() {
                match member {
                    Member::Attribute(attribute) => dto.members.push(MemberRow {
                        id: attribute.id.raw(),
                        element_id: element.id.raw(),
                        position: position(index),
                        kind: String::from("attribute"),
                        name: attribute.name.as_str().to_owned(),
                        visibility: visibility_text(attribute.visibility).to_owned(),
                        type_text: Some(attribute.type_text.clone()),
                        multiplicity: attribute.multiplicity.map(|value| value.to_string()),
                        return_type: None,
                    }),
                    Member::Operation(operation) => {
                        dto.members.push(MemberRow {
                            id: operation.id.raw(),
                            element_id: element.id.raw(),
                            position: position(index),
                            kind: String::from("operation"),
                            name: operation.name.as_str().to_owned(),
                            visibility: visibility_text(operation.visibility).to_owned(),
                            type_text: None,
                            multiplicity: None,
                            return_type: operation.return_type.clone(),
                        });
                        for (slot, parameter) in operation.parameters.iter().enumerate() {
                            dto.parameters.push(ParameterRow {
                                member_id: operation.id.raw(),
                                position: position(slot),
                                name: parameter.name.as_str().to_owned(),
                                type_text: parameter.type_text.clone(),
                            });
                        }
                    }
                }
            }
        }
        for relation in document.relations.values() {
            dto.relations.push(RelationRow {
                id: relation.id.raw(),
                kind: kind_text(relation.kind).to_owned(),
                source: relation.source.raw(),
                target: relation.target.raw(),
                source_role: role(&relation.source_end),
                source_multiplicity: multiplicity(&relation.source_end),
                target_role: role(&relation.target_end),
                target_multiplicity: multiplicity(&relation.target_end),
            });
        }
        for (index, id) in document.diagram_order.iter().enumerate() {
            let Some(diagram) = document.diagrams.get(id) else {
                continue;
            };
            dto.diagrams.push(DiagramRow {
                id: diagram.id.raw(),
                position: position(index),
                name: diagram.name.as_str().to_owned(),
                notation: notation_text(diagram.notation).to_owned(),
            });
            for (slot, node) in diagram.nodes.iter().enumerate() {
                dto.nodes.push(NodeRow {
                    id: node.id.raw(),
                    diagram_id: diagram.id.raw(),
                    position: position(slot),
                    element_id: node.element.raw(),
                    x: node.bounds.left(),
                    y: node.bounds.top(),
                    width: node.bounds.size().width(),
                    height: node.bounds.size().height(),
                });
            }
            for (slot, edge) in diagram.edges.iter().enumerate() {
                dto.edges.push(EdgeRow {
                    id: edge.id.raw(),
                    diagram_id: diagram.id.raw(),
                    position: position(slot),
                    relation_id: edge.relation.raw(),
                    source_view: edge.source.raw(),
                    target_view: edge.target.raw(),
                });
            }
        }
        dto
    }
}
