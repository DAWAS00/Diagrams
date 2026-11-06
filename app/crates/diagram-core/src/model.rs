//! Document records: shared model (elements, members, relations) kept apart
//! from diagram views (R1-01).
//!
//! Records live in ordered maps keyed by typed IDs, so iteration order is
//! deterministic. Order that matters to the user (members, diagram order,
//! z-order of views) is an explicit vector.

use std::collections::BTreeMap;
use std::fmt;

use thiserror::Error;

use crate::geometry::Rect;
use crate::ids::{DiagramId, ElementId, MemberId, RelationId, ViewId};
use crate::notation::{NotationId, RelationKind};

/// A nonblank display name. Spelling and Unicode are preserved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

/// A name was empty or only whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("a name must contain a non-whitespace character")]
pub struct BlankName;

impl Name {
    /// A checked name.
    ///
    /// # Errors
    ///
    /// Returns [`BlankName`] for an empty or whitespace-only name.
    pub fn new(value: impl Into<String>) -> Result<Self, BlankName> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(BlankName)
        } else {
            Ok(Self(value))
        }
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Member visibility, shown with its readable name and UML symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    /// `+`
    #[default]
    Public,
    /// `-`
    Private,
    /// `#`
    Protected,
    /// `~`
    Package,
}

impl Visibility {
    /// The UML symbol.
    pub const fn symbol(self) -> char {
        match self {
            Self::Public => '+',
            Self::Private => '-',
            Self::Protected => '#',
            Self::Package => '~',
        }
    }
}

/// Upper bound of a multiplicity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Upper {
    /// A number.
    Bounded(u32),
    /// `*`
    Unbounded,
}

/// A multiplicity: `n`, `*`, or `lower..upper` with upper an integer or `*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Multiplicity {
    lower: u32,
    upper: Upper,
}

/// Why a multiplicity was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MultiplicityError {
    /// The text is not `n`, `*` or `lower..upper`.
    #[error("{0:?} is not a multiplicity; use a number, *, or lower..upper such as 0..*")]
    Syntax(String),
    /// The lower bound is above the upper bound.
    #[error("the lower bound {lower} is greater than the upper bound {upper}")]
    Inverted {
        /// Lower bound.
        lower: u32,
        /// Upper bound.
        upper: u32,
    },
}

impl Multiplicity {
    /// Parses `n`, `*`, `lower..upper` or `lower..*`, ignoring surrounding spaces.
    ///
    /// # Errors
    ///
    /// Returns [`MultiplicityError`] for other text or an inverted range.
    pub fn parse(text: &str) -> Result<Self, MultiplicityError> {
        let text = text.trim();
        let syntax = || MultiplicityError::Syntax(text.to_owned());
        let number = |part: &str| part.trim().parse::<u32>().map_err(|_| syntax());
        let upper = |part: &str| match part.trim() {
            "*" => Ok(Upper::Unbounded),
            other => number(other).map(Upper::Bounded),
        };
        let (lower, upper) = match text.split_once("..") {
            Some((lower, rest)) => (number(lower)?, upper(rest)?),
            None if text == "*" => (0, Upper::Unbounded),
            None => {
                let exact = number(text)?;
                (exact, Upper::Bounded(exact))
            }
        };
        if let Upper::Bounded(upper) = upper
            && lower > upper
        {
            return Err(MultiplicityError::Inverted { lower, upper });
        }
        Ok(Self { lower, upper })
    }

    /// Lower bound.
    pub const fn lower(self) -> u32 {
        self.lower
    }

    /// Upper bound.
    pub const fn upper(self) -> Upper {
        self.upper
    }
}

impl fmt::Display for Multiplicity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.upper {
            Upper::Unbounded if self.lower == 0 => formatter.write_str("*"),
            Upper::Unbounded => write!(formatter, "{}..*", self.lower),
            Upper::Bounded(upper) if upper == self.lower => write!(formatter, "{upper}"),
            Upper::Bounded(upper) => write!(formatter, "{}..{upper}", self.lower),
        }
    }
}

/// An attribute of a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Identity.
    pub id: MemberId,
    /// Name.
    pub name: Name,
    /// Descriptive type text; not type-checked in v0.1.
    pub type_text: String,
    /// Visibility.
    pub visibility: Visibility,
    /// Optional multiplicity.
    pub multiplicity: Option<Multiplicity>,
}

/// A parameter of an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    /// Name.
    pub name: Name,
    /// Descriptive type text.
    pub type_text: String,
}

/// An operation of a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    /// Identity.
    pub id: MemberId,
    /// Name.
    pub name: Name,
    /// Visibility.
    pub visibility: Visibility,
    /// Parameters in order.
    pub parameters: Vec<Parameter>,
    /// Optional return type text.
    pub return_type: Option<String>,
}

/// A member of a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Member {
    /// An attribute.
    Attribute(Attribute),
    /// An operation.
    Operation(Operation),
}

impl Member {
    /// The member's identity.
    pub const fn id(&self) -> MemberId {
        match self {
            Self::Attribute(attribute) => attribute.id,
            Self::Operation(operation) => operation.id,
        }
    }

    /// Attribute or operation.
    pub const fn kind(&self) -> MemberKind {
        match self {
            Self::Attribute(_) => MemberKind::Attribute,
            Self::Operation(_) => MemberKind::Operation,
        }
    }
}

/// The two kinds of member. Editing never turns one into the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberKind {
    /// An attribute.
    Attribute,
    /// An operation.
    Operation,
}

impl fmt::Display for MemberKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Attribute => "attribute",
            Self::Operation => "operation",
        })
    }
}

impl MemberKind {
    /// The kind with its article: "an attribute", "an operation".
    pub const fn with_article(self) -> &'static str {
        match self {
            Self::Attribute => "an attribute",
            Self::Operation => "an operation",
        }
    }
}

/// A model element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Identity.
    pub id: ElementId,
    /// Name, shared by every view.
    pub name: Name,
    /// Members in display order (attributes and operations).
    pub members: Vec<Member>,
}

/// One end of an association.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssociationEnd {
    /// Optional role name.
    pub role: Option<Name>,
    /// Optional multiplicity.
    pub multiplicity: Option<Multiplicity>,
}

/// A semantic relationship. For a generalization, `source` is the child
/// and `target` the parent, and the ends stay empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    /// Identity.
    pub id: RelationId,
    /// Association or generalization.
    pub kind: RelationKind,
    /// Source element (child for a generalization).
    pub source: ElementId,
    /// Target element (parent for a generalization).
    pub target: ElementId,
    /// Source end (associations only).
    pub source_end: AssociationEnd,
    /// Target end (associations only).
    pub target_end: AssociationEnd,
}

/// An element shown in a diagram.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeView {
    /// Identity of this appearance.
    pub id: ViewId,
    /// The element shown.
    pub element: ElementId,
    /// Bounds in world units.
    pub bounds: Rect,
}

/// A relation shown in a diagram, between two node views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeView {
    /// Identity of this appearance.
    pub id: ViewId,
    /// The relation shown.
    pub relation: RelationId,
    /// Node view at the source end.
    pub source: ViewId,
    /// Node view at the target end.
    pub target: ViewId,
}

/// A diagram: views only, never model data.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagram {
    /// Identity.
    pub id: DiagramId,
    /// Name.
    pub name: Name,
    /// Notation.
    pub notation: NotationId,
    /// Node views in z-order (last is on top).
    pub nodes: Vec<NodeView>,
    /// Edge views.
    pub edges: Vec<EdgeView>,
}

impl Diagram {
    /// The node view with this ID.
    pub fn node(&self, id: ViewId) -> Option<&NodeView> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// The node view showing `element`, if the element is in this diagram.
    pub fn node_of(&self, element: ElementId) -> Option<&NodeView> {
        self.nodes.iter().find(|node| node.element == element)
    }
}

/// The authoritative project document.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    /// Elements by ID.
    pub elements: BTreeMap<ElementId, Element>,
    /// Relations by ID.
    pub relations: BTreeMap<RelationId, Relation>,
    /// Diagrams by ID.
    pub diagrams: BTreeMap<DiagramId, Diagram>,
    /// Diagram order in the project.
    pub diagram_order: Vec<DiagramId>,
    /// The last raw ID handed out. While a record exists its ID is never
    /// handed out again, but undoing a creation also restores this counter,
    /// so the undone record's ID can be handed out again later. Code that
    /// keeps an ID across an Undo must fence it by document revision.
    pub next_id: u64,
}

impl Document {
    /// An empty document with one empty class diagram named `diagram_name`.
    pub fn with_diagram(diagram_name: Name) -> (Self, DiagramId) {
        let mut document = Self::default();
        let id = DiagramId::from_raw(document.allocate());
        document.diagrams.insert(
            id,
            Diagram {
                id,
                name: diagram_name,
                notation: NotationId::Class,
                nodes: Vec::new(),
                edges: Vec::new(),
            },
        );
        document.diagram_order.push(id);
        (document, id)
    }

    /// Takes the next raw ID.
    pub fn allocate(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Every node view of `element`, as (diagram, view).
    pub fn views_of(&self, element: ElementId) -> impl Iterator<Item = (DiagramId, ViewId)> + '_ {
        self.diagrams.values().flat_map(move |diagram| {
            diagram
                .nodes
                .iter()
                .filter(move |node| node.element == element)
                .map(move |node| (diagram.id, node.id))
        })
    }

    /// Relations that touch `element`.
    pub fn relations_of(&self, element: ElementId) -> impl Iterator<Item = &Relation> + '_ {
        self.relations
            .values()
            .filter(move |relation| relation.source == element || relation.target == element)
    }
}
