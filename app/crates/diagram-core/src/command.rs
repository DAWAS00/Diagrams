//! Reversible, atomic document commands (R1-03).
//!
//! A [`Command`] is validated against the current document and turned into
//! a [`ChangeSet`]: a labelled list of record-level before/after patches.
//! Applying a change set is all-or-nothing, and its inverse restores the
//! exact previous document. History stores change sets, never whole
//! document copies.

use crate::geometry::{Point, Rect};
use crate::ids::{DiagramId, ElementId, MemberId, RelationId, ViewId};
use crate::model::{
    AssociationEnd, Attribute, Diagram, Document, EdgeView, Element, Member, Multiplicity, Name,
    NodeView, Operation, Relation, Visibility,
};
use crate::notation::{NotationId, RelationKind};
use crate::validate::{DomainError, check_relation};

/// Role and multiplicity text for one association end, as typed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EndInput {
    /// Role name; blank means none.
    pub role: String,
    /// Multiplicity text; blank means none.
    pub multiplicity: String,
}

/// One user action on the document.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Create a class and its view in `diagram` together.
    CreateClass {
        /// Diagram to place it in.
        diagram: DiagramId,
        /// Class name.
        name: String,
        /// View bounds.
        bounds: Rect,
    },
    /// Rename an element; every view shows the new name.
    RenameElement {
        /// Element.
        element: ElementId,
        /// New name.
        name: String,
    },
    /// Append an attribute.
    AddAttribute {
        /// Owning class.
        element: ElementId,
        /// Name.
        name: String,
        /// Type text.
        type_text: String,
        /// Visibility.
        visibility: Visibility,
        /// Multiplicity text; blank means none.
        multiplicity: String,
    },
    /// Append an operation without parameters.
    AddOperation {
        /// Owning class.
        element: ElementId,
        /// Name.
        name: String,
        /// Visibility.
        visibility: Visibility,
        /// Return type; blank means none.
        return_type: String,
    },
    /// Remove a member from its class.
    RemoveMember {
        /// Owning class.
        element: ElementId,
        /// Member.
        member: MemberId,
    },
    /// Move a member to position `to`.
    MoveMember {
        /// Owning class.
        element: ElementId,
        /// Member.
        member: MemberId,
        /// New position.
        to: usize,
    },
    /// Create a relationship and its connector between two views of `diagram`.
    Connect {
        /// Diagram.
        diagram: DiagramId,
        /// Kind.
        kind: RelationKind,
        /// Source view (child for a generalization).
        source: ViewId,
        /// Target view (parent for a generalization).
        target: ViewId,
    },
    /// Set an association's roles and multiplicities.
    SetEnds {
        /// Relation.
        relation: RelationId,
        /// Source end.
        source: EndInput,
        /// Target end.
        target: EndInput,
    },
    /// Move node views to new top-left positions (one drag or nudge).
    MoveViews {
        /// Diagram.
        diagram: DiagramId,
        /// New positions.
        moves: Vec<(ViewId, Point)>,
    },
    /// Remove node views and their connectors from this diagram only.
    RemoveViews {
        /// Diagram.
        diagram: DiagramId,
        /// Node views.
        views: Vec<ViewId>,
    },
    /// Show an existing element in another diagram.
    ShowInDiagram {
        /// Element.
        element: ElementId,
        /// Target diagram.
        diagram: DiagramId,
        /// View bounds.
        bounds: Rect,
    },
    /// Delete an element, its relations and all their views everywhere.
    DeleteElement {
        /// Element.
        element: ElementId,
    },
    /// Add an empty class diagram.
    AddDiagram {
        /// Name.
        name: String,
    },
    /// Rename a diagram; no model it shows is renamed.
    RenameDiagram {
        /// Diagram.
        diagram: DiagramId,
        /// New name.
        name: String,
    },
    /// Delete a diagram and its views; shared models stay.
    DeleteDiagram {
        /// Diagram.
        diagram: DiagramId,
    },
}

impl Command {
    /// The action label shown on Undo and Redo, such as "Rename class".
    pub fn label(&self) -> &'static str {
        match self {
            Self::CreateClass { .. } => "Add class",
            Self::RenameElement { .. } => "Rename class",
            Self::AddAttribute { .. } => "Add attribute",
            Self::AddOperation { .. } => "Add operation",
            Self::RemoveMember { .. } => "Remove member",
            Self::MoveMember { .. } => "Move member",
            Self::Connect {
                kind: RelationKind::Association,
                ..
            } => "Add association",
            Self::Connect {
                kind: RelationKind::Generalization,
                ..
            } => "Add generalization",
            Self::SetEnds { .. } => "Edit relationship ends",
            Self::MoveViews { .. } => "Move",
            Self::RemoveViews { .. } => "Remove from diagram",
            Self::ShowInDiagram { .. } => "Show in diagram",
            Self::DeleteElement { .. } => "Delete model everywhere",
            Self::AddDiagram { .. } => "Add diagram",
            Self::RenameDiagram { .. } => "Rename diagram",
            Self::DeleteDiagram { .. } => "Delete diagram",
        }
    }
}

/// One record-level patch. `None` means "absent": before `None` and after
/// `Some` inserts, the reverse removes, both `Some` replaces.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// An element record.
    Element {
        /// ID.
        id: ElementId,
        /// Value before.
        before: Option<Element>,
        /// Value after.
        after: Option<Element>,
    },
    /// A relation record.
    Relation {
        /// ID.
        id: RelationId,
        /// Value before.
        before: Option<Relation>,
        /// Value after.
        after: Option<Relation>,
    },
    /// A node view at a z-order position.
    Node {
        /// Diagram.
        diagram: DiagramId,
        /// Position in the node list.
        index: usize,
        /// Value before.
        before: Option<NodeView>,
        /// Value after.
        after: Option<NodeView>,
    },
    /// An edge view at a position.
    Edge {
        /// Diagram.
        diagram: DiagramId,
        /// Position in the edge list.
        index: usize,
        /// Value before.
        before: Option<EdgeView>,
        /// Value after.
        after: Option<EdgeView>,
    },
    /// A diagram record (with its views) at a position in the diagram order.
    Diagram {
        /// Position in the diagram order.
        index: usize,
        /// Value before.
        before: Option<Diagram>,
        /// Value after.
        after: Option<Diagram>,
    },
    /// The ID counter.
    NextId {
        /// Value before.
        before: u64,
        /// Value after.
        after: u64,
    },
}

impl Change {
    /// The patch that undoes this one.
    pub fn inverse(&self) -> Self {
        match self.clone() {
            Self::Element { id, before, after } => Self::Element {
                id,
                before: after,
                after: before,
            },
            Self::Relation { id, before, after } => Self::Relation {
                id,
                before: after,
                after: before,
            },
            Self::Node {
                diagram,
                index,
                before,
                after,
            } => Self::Node {
                diagram,
                index,
                before: after,
                after: before,
            },
            Self::Edge {
                diagram,
                index,
                before,
                after,
            } => Self::Edge {
                diagram,
                index,
                before: after,
                after: before,
            },
            Self::Diagram {
                index,
                before,
                after,
            } => Self::Diagram {
                index,
                before: after,
                after: before,
            },
            Self::NextId { before, after } => Self::NextId {
                before: after,
                after: before,
            },
        }
    }
}

/// A validated, labelled set of patches: one undoable user action.
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeSet {
    /// Action label.
    pub label: &'static str,
    /// Patches in application order.
    pub changes: Vec<Change>,
}

impl ChangeSet {
    /// The change set that undoes this one.
    pub fn inverse(&self) -> Self {
        Self {
            label: self.label,
            changes: self.changes.iter().rev().map(Change::inverse).collect(),
        }
    }
}

/// A patch did not match the document it was applied to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the change no longer matches the document")]
pub struct StaleChange;

/// Replaces `slot` if it holds `before`.
fn swap<T: PartialEq + Clone>(slot: &mut Option<T>, before: &Option<T>, after: &Option<T>) -> bool {
    if slot == before {
        slot.clone_from(after);
        true
    } else {
        false
    }
}

/// Applies a list patch at `index`: insert, remove or replace.
fn patch_list<T: PartialEq + Clone>(
    list: &mut Vec<T>,
    index: usize,
    before: &Option<T>,
    after: &Option<T>,
) -> bool {
    match (before, after) {
        (None, Some(value)) if index <= list.len() => {
            list.insert(index, value.clone());
            true
        }
        (Some(old), None) if list.get(index) == Some(old) => {
            list.remove(index);
            true
        }
        (Some(old), Some(new)) if list.get(index) == Some(old) => {
            list[index] = new.clone();
            true
        }
        _ => false,
    }
}

fn apply_change(document: &mut Document, change: &Change) -> bool {
    match change {
        Change::Element { id, before, after } => {
            let mut slot = document.elements.remove(id);
            let ok = swap(&mut slot, before, after);
            if let Some(element) = slot {
                document.elements.insert(*id, element);
            }
            ok
        }
        Change::Relation { id, before, after } => {
            let mut slot = document.relations.remove(id);
            let ok = swap(&mut slot, before, after);
            if let Some(relation) = slot {
                document.relations.insert(*id, relation);
            }
            ok
        }
        Change::Node {
            diagram,
            index,
            before,
            after,
        } => document
            .diagrams
            .get_mut(diagram)
            .is_some_and(|diagram| patch_list(&mut diagram.nodes, *index, before, after)),
        Change::Edge {
            diagram,
            index,
            before,
            after,
        } => document
            .diagrams
            .get_mut(diagram)
            .is_some_and(|diagram| patch_list(&mut diagram.edges, *index, before, after)),
        Change::Diagram {
            index,
            before,
            after,
        } => {
            let id = before.as_ref().or(after.as_ref()).map(|diagram| diagram.id);
            let Some(id) = id else {
                return false;
            };
            let in_order = document.diagram_order.get(*index) == Some(&id);
            let current = document.diagrams.get(&id);
            match (before, after) {
                (None, Some(new))
                    if current.is_none() && *index <= document.diagram_order.len() =>
                {
                    document.diagram_order.insert(*index, id);
                    document.diagrams.insert(id, new.clone());
                    true
                }
                (Some(old), None) if in_order && current == Some(old) => {
                    document.diagram_order.remove(*index);
                    document.diagrams.remove(&id);
                    true
                }
                (Some(old), Some(new)) if in_order && current == Some(old) => {
                    document.diagrams.insert(id, new.clone());
                    true
                }
                _ => false,
            }
        }
        Change::NextId { before, after } => {
            if document.next_id == *before {
                document.next_id = *after;
                true
            } else {
                false
            }
        }
    }
}

/// Applies every patch of `set`, or none of them.
///
/// # Errors
///
/// Returns [`StaleChange`] when a patch does not match the document; the
/// patches already applied are rolled back, so the document is unchanged.
pub fn apply(document: &mut Document, set: &ChangeSet) -> Result<(), StaleChange> {
    for (applied, change) in set.changes.iter().enumerate() {
        if !apply_change(document, change) {
            for done in set.changes[..applied].iter().rev() {
                // Undoing a patch that was just applied cannot fail.
                let _ = apply_change(document, &done.inverse());
            }
            return Err(StaleChange);
        }
    }
    Ok(())
}

/// Builds patches against a read-only document, allocating IDs as it goes.
struct Builder<'a> {
    document: &'a Document,
    next_id: u64,
    changes: Vec<Change>,
}

impl<'a> Builder<'a> {
    fn new(document: &'a Document) -> Self {
        Self {
            document,
            next_id: document.next_id,
            changes: Vec::new(),
        }
    }

    fn id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn finish(mut self, label: &'static str) -> ChangeSet {
        if self.next_id != self.document.next_id {
            self.changes.push(Change::NextId {
                before: self.document.next_id,
                after: self.next_id,
            });
        }
        ChangeSet {
            label,
            changes: self.changes,
        }
    }

    fn element(&self, id: ElementId) -> Result<&'a Element, DomainError> {
        self.document
            .elements
            .get(&id)
            .ok_or(DomainError::MissingElement(id))
    }

    fn diagram(&self, id: DiagramId) -> Result<&'a Diagram, DomainError> {
        self.document
            .diagrams
            .get(&id)
            .ok_or(DomainError::MissingDiagram(id))
    }

    fn replace_element(&mut self, before: &Element, after: Element) {
        self.changes.push(Change::Element {
            id: before.id,
            before: Some(before.clone()),
            after: Some(after),
        });
    }

    /// Removes nodes (by index, highest first so indices stay valid) and
    /// every edge in `diagram` that touches them or is listed in `edges`.
    fn remove_views(&mut self, diagram: &Diagram, nodes: &[ViewId], relations: &[RelationId]) {
        let mut edge_indices: Vec<usize> = diagram
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| {
                nodes.contains(&edge.source)
                    || nodes.contains(&edge.target)
                    || relations.contains(&edge.relation)
            })
            .map(|(index, _)| index)
            .collect();
        edge_indices.reverse();
        for index in edge_indices {
            self.changes.push(Change::Edge {
                diagram: diagram.id,
                index,
                before: Some(diagram.edges[index].clone()),
                after: None,
            });
        }
        let mut node_indices: Vec<usize> = diagram
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| nodes.contains(&node.id))
            .map(|(index, _)| index)
            .collect();
        node_indices.reverse();
        for index in node_indices {
            self.changes.push(Change::Node {
                diagram: diagram.id,
                index,
                before: Some(diagram.nodes[index].clone()),
                after: None,
            });
        }
    }
}

fn optional_name(text: &str) -> Result<Option<Name>, DomainError> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(Name::new(text)?))
    }
}

fn optional_multiplicity(text: &str) -> Result<Option<Multiplicity>, DomainError> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(Multiplicity::parse(text)?))
    }
}

fn optional_text(text: &str) -> Option<String> {
    (!text.trim().is_empty()).then(|| text.to_owned())
}

/// Validates `command` against `document` and returns its change set.
/// Nothing is changed.
///
/// # Errors
///
/// Returns the [`DomainError`] for the first rule the command breaks.
pub fn prepare(document: &Document, command: &Command) -> Result<ChangeSet, DomainError> {
    let mut builder = Builder::new(document);
    match command {
        Command::CreateClass {
            diagram,
            name,
            bounds,
        } => {
            let target = builder.diagram(*diagram)?;
            let name = Name::new(name.as_str())?;
            let element = ElementId::from_raw(builder.id());
            let view = ViewId::from_raw(builder.id());
            builder.changes.push(Change::Element {
                id: element,
                before: None,
                after: Some(Element {
                    id: element,
                    name,
                    members: Vec::new(),
                }),
            });
            builder.changes.push(Change::Node {
                diagram: *diagram,
                index: target.nodes.len(),
                before: None,
                after: Some(NodeView {
                    id: view,
                    element,
                    bounds: *bounds,
                }),
            });
        }
        Command::RenameElement { element, name } => {
            let current = builder.element(*element)?;
            let name = Name::new(name.as_str())?;
            if current.name == name {
                return Err(DomainError::NoChange);
            }
            let renamed = Element {
                name,
                ..current.clone()
            };
            builder.replace_element(current, renamed);
        }
        Command::AddAttribute {
            element,
            name,
            type_text,
            visibility,
            multiplicity,
        } => {
            let current = builder.element(*element)?;
            let attribute = Attribute {
                id: MemberId::from_raw(builder.id()),
                name: Name::new(name.as_str())?,
                type_text: type_text.trim().to_owned(),
                visibility: *visibility,
                multiplicity: optional_multiplicity(multiplicity)?,
            };
            let mut changed = current.clone();
            changed.members.push(Member::Attribute(attribute));
            builder.replace_element(current, changed);
        }
        Command::AddOperation {
            element,
            name,
            visibility,
            return_type,
        } => {
            let current = builder.element(*element)?;
            let operation = Operation {
                id: MemberId::from_raw(builder.id()),
                name: Name::new(name.as_str())?,
                visibility: *visibility,
                parameters: Vec::new(),
                return_type: optional_text(return_type),
            };
            let mut changed = current.clone();
            changed.members.push(Member::Operation(operation));
            builder.replace_element(current, changed);
        }
        Command::RemoveMember { element, member } => {
            let current = builder.element(*element)?;
            let mut changed = current.clone();
            let position = changed
                .members
                .iter()
                .position(|candidate| candidate.id() == *member)
                .ok_or(DomainError::MissingMember(*member))?;
            changed.members.remove(position);
            builder.replace_element(current, changed);
        }
        Command::MoveMember {
            element,
            member,
            to,
        } => {
            let current = builder.element(*element)?;
            let count = current.members.len();
            let from = current
                .members
                .iter()
                .position(|candidate| candidate.id() == *member)
                .ok_or(DomainError::MissingMember(*member))?;
            if *to >= count {
                return Err(DomainError::MemberIndex { index: *to, count });
            }
            if *to == from {
                return Err(DomainError::NoChange);
            }
            let mut changed = current.clone();
            let moved = changed.members.remove(from);
            changed.members.insert(*to, moved);
            builder.replace_element(current, changed);
        }
        Command::Connect {
            diagram,
            kind,
            source,
            target,
        } => {
            let target_diagram = builder.diagram(*diagram)?;
            let source_node = target_diagram
                .node(*source)
                .ok_or(DomainError::MissingView(*source))?;
            let target_node = target_diagram
                .node(*target)
                .ok_or(DomainError::MissingView(*target))?;
            check_relation(document, *kind, source_node.element, target_node.element)?;
            let relation = RelationId::from_raw(builder.id());
            let edge = ViewId::from_raw(builder.id());
            builder.changes.push(Change::Relation {
                id: relation,
                before: None,
                after: Some(Relation {
                    id: relation,
                    kind: *kind,
                    source: source_node.element,
                    target: target_node.element,
                    source_end: AssociationEnd::default(),
                    target_end: AssociationEnd::default(),
                }),
            });
            builder.changes.push(Change::Edge {
                diagram: *diagram,
                index: target_diagram.edges.len(),
                before: None,
                after: Some(EdgeView {
                    id: edge,
                    relation,
                    source: *source,
                    target: *target,
                }),
            });
        }
        Command::SetEnds {
            relation,
            source,
            target,
        } => {
            let current = document
                .relations
                .get(relation)
                .ok_or(DomainError::MissingRelation(*relation))?;
            if current.kind != RelationKind::Association {
                return Err(DomainError::EndsOnGeneralization);
            }
            let end = |input: &EndInput| -> Result<AssociationEnd, DomainError> {
                Ok(AssociationEnd {
                    role: optional_name(&input.role)?,
                    multiplicity: optional_multiplicity(&input.multiplicity)?,
                })
            };
            let changed = Relation {
                source_end: end(source)?,
                target_end: end(target)?,
                ..current.clone()
            };
            if &changed == current {
                return Err(DomainError::NoChange);
            }
            builder.changes.push(Change::Relation {
                id: *relation,
                before: Some(current.clone()),
                after: Some(changed),
            });
        }
        Command::MoveViews { diagram, moves } => {
            let target = builder.diagram(*diagram)?;
            for (view, origin) in moves {
                let index = target
                    .nodes
                    .iter()
                    .position(|node| node.id == *view)
                    .ok_or(DomainError::MissingView(*view))?;
                let before = &target.nodes[index];
                if before.bounds.origin() == *origin {
                    continue;
                }
                builder.changes.push(Change::Node {
                    diagram: *diagram,
                    index,
                    before: Some(before.clone()),
                    after: Some(NodeView {
                        bounds: before.bounds.moved_to(*origin),
                        ..before.clone()
                    }),
                });
            }
            if builder.changes.is_empty() {
                return Err(DomainError::NoChange);
            }
        }
        Command::RemoveViews { diagram, views } => {
            let target = builder.diagram(*diagram)?;
            if let Some(missing) = views.iter().find(|view| target.node(**view).is_none()) {
                return Err(DomainError::MissingView(*missing));
            }
            if views.is_empty() {
                return Err(DomainError::NoChange);
            }
            builder.remove_views(target, views, &[]);
        }
        Command::ShowInDiagram {
            element,
            diagram,
            bounds,
        } => {
            builder.element(*element)?;
            let target = builder.diagram(*diagram)?;
            if target.node_of(*element).is_some() {
                return Err(DomainError::AlreadyInDiagram {
                    element: *element,
                    diagram: *diagram,
                });
            }
            let view = ViewId::from_raw(builder.id());
            builder.changes.push(Change::Node {
                diagram: *diagram,
                index: target.nodes.len(),
                before: None,
                after: Some(NodeView {
                    id: view,
                    element: *element,
                    bounds: *bounds,
                }),
            });
        }
        Command::DeleteElement { element } => {
            let current = builder.element(*element)?;
            let relations: Vec<RelationId> = document
                .relations_of(*element)
                .map(|relation| relation.id)
                .collect();
            for diagram in document.diagrams.values() {
                let nodes: Vec<ViewId> = diagram
                    .nodes
                    .iter()
                    .filter(|node| node.element == *element)
                    .map(|node| node.id)
                    .collect();
                builder.remove_views(diagram, &nodes, &relations);
            }
            for id in &relations {
                builder.changes.push(Change::Relation {
                    id: *id,
                    before: document.relations.get(id).cloned(),
                    after: None,
                });
            }
            builder.changes.push(Change::Element {
                id: *element,
                before: Some(current.clone()),
                after: None,
            });
        }
        Command::AddDiagram { name } => {
            let name = Name::new(name.as_str())?;
            let id = DiagramId::from_raw(builder.id());
            builder.changes.push(Change::Diagram {
                index: document.diagram_order.len(),
                before: None,
                after: Some(Diagram {
                    id,
                    name,
                    notation: NotationId::Class,
                    nodes: Vec::new(),
                    edges: Vec::new(),
                }),
            });
        }
        Command::RenameDiagram { diagram, name } => {
            let current = builder.diagram(*diagram)?;
            let name = Name::new(name.as_str())?;
            if current.name == name {
                return Err(DomainError::NoChange);
            }
            let index = order_index(document, *diagram)?;
            builder.changes.push(Change::Diagram {
                index,
                before: Some(current.clone()),
                after: Some(Diagram {
                    name,
                    ..current.clone()
                }),
            });
        }
        Command::DeleteDiagram { diagram } => {
            let current = builder.diagram(*diagram)?;
            let index = order_index(document, *diagram)?;
            builder.changes.push(Change::Diagram {
                index,
                before: Some(current.clone()),
                after: None,
            });
        }
    }
    Ok(builder.finish(command.label()))
}

fn order_index(document: &Document, diagram: DiagramId) -> Result<usize, DomainError> {
    document
        .diagram_order
        .iter()
        .position(|id| *id == diagram)
        .ok_or(DomainError::MissingDiagram(diagram))
}

/// Validates and applies `command`, returning the applied change set.
///
/// # Errors
///
/// Returns the [`DomainError`] when the command is rejected; the document
/// is then unchanged.
pub fn execute(document: &mut Document, command: &Command) -> Result<ChangeSet, DomainError> {
    let set = prepare(document, command)?;
    // A change set prepared against this very document always matches it.
    apply(document, &set).map_err(|_| DomainError::Stale)?;
    Ok(set)
}

/// What deleting an element everywhere would remove, for the impact review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Impact {
    /// Members of the element.
    pub members: usize,
    /// Relations touching it.
    pub relations: usize,
    /// Node views of it, across diagrams.
    pub views: usize,
    /// Connectors removed with them.
    pub connectors: usize,
    /// Diagrams affected.
    pub diagrams: usize,
}

/// Counts what [`Command::DeleteElement`] would remove.
pub fn delete_impact(document: &Document, element: ElementId) -> Option<Impact> {
    let current = document.elements.get(&element)?;
    let relations: Vec<RelationId> = document
        .relations_of(element)
        .map(|relation| relation.id)
        .collect();
    let mut impact = Impact {
        members: current.members.len(),
        relations: relations.len(),
        ..Impact::default()
    };
    for diagram in document.diagrams.values() {
        let views = diagram
            .nodes
            .iter()
            .filter(|node| node.element == element)
            .count();
        let connectors = diagram
            .edges
            .iter()
            .filter(|edge| relations.contains(&edge.relation))
            .count();
        impact.views += views;
        impact.connectors += connectors;
        impact.diagrams += usize::from(views + connectors > 0);
    }
    Some(impact)
}
