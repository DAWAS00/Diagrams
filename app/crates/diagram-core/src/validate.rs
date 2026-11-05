//! Domain rules (R1-02): typed errors for rejected commands, and a
//! whole-document check that reports blocking errors and warnings.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::ids::{DiagramId, ElementId, MemberId, RelationId, ViewId};
use crate::model::{BlankName, Document, MultiplicityError};
use crate::notation::RelationKind;

/// Why a command was rejected. The document is unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// No such element.
    #[error("the model element {0} does not exist")]
    MissingElement(ElementId),
    /// No such member.
    #[error("the member {0} does not exist on this class")]
    MissingMember(MemberId),
    /// No such relation.
    #[error("the relationship {0} does not exist")]
    MissingRelation(RelationId),
    /// No such diagram.
    #[error("the diagram {0} does not exist")]
    MissingDiagram(DiagramId),
    /// No such view in the diagram.
    #[error("the view {0} is not in this diagram")]
    MissingView(ViewId),
    /// A name was blank.
    #[error(transparent)]
    BlankName(#[from] BlankName),
    /// A multiplicity was invalid.
    #[error(transparent)]
    Multiplicity(#[from] MultiplicityError),
    /// A class cannot generalize itself.
    #[error("a class cannot be its own parent")]
    SelfGeneralization,
    /// The generalization would make a class its own ancestor.
    #[error("{child} already is an ancestor of {parent}; this generalization would create a cycle")]
    GeneralizationCycle {
        /// Proposed child.
        child: ElementId,
        /// Proposed parent.
        parent: ElementId,
    },
    /// The same generalization already exists.
    #[error("{child} already generalizes {parent}")]
    DuplicateGeneralization {
        /// Child.
        child: ElementId,
        /// Parent.
        parent: ElementId,
    },
    /// Self-associations wait for a readable loop route.
    #[error("connecting a class to itself is not supported yet: it needs a loop route")]
    SelfAssociationUnsupported,
    /// Ends (roles, multiplicities) belong to associations only.
    #[error("a generalization has no roles or multiplicities")]
    EndsOnGeneralization,
    /// The element already has a view in that diagram (v0.1 reveals it instead).
    #[error("{element} is already shown in {diagram}")]
    AlreadyInDiagram {
        /// Element.
        element: ElementId,
        /// Diagram.
        diagram: DiagramId,
    },
    /// A member position is outside the list.
    #[error("position {index} is outside the member list of {count}")]
    MemberIndex {
        /// Requested position.
        index: usize,
        /// Number of members.
        count: usize,
    },
    /// The command would change nothing.
    #[error("this change would not change anything")]
    NoChange,
    /// The document changed after the edit was prepared.
    #[error("the document changed while this edit was prepared; try again")]
    Stale,
}

/// Whether adding `child → parent` would close a generalization cycle.
pub fn would_cycle(document: &Document, child: ElementId, parent: ElementId) -> bool {
    reaches_ancestor(document, parent, child, None)
}

/// Whether `ancestor` is reachable by walking up generalizations from
/// `start`, ignoring the relation `skip`.
fn reaches_ancestor(
    document: &Document,
    start: ElementId,
    ancestor: ElementId,
    skip: Option<RelationId>,
) -> bool {
    let mut stack = vec![start];
    let mut seen = BTreeSet::new();
    while let Some(current) = stack.pop() {
        if current == ancestor {
            return true;
        }
        if !seen.insert(current) {
            continue;
        }
        stack.extend(
            document
                .relations
                .values()
                .filter(|relation| {
                    Some(relation.id) != skip
                        && relation.kind == RelationKind::Generalization
                        && relation.source == current
                })
                .map(|relation| relation.target),
        );
    }
    false
}

/// Checks a new relationship before it is created.
///
/// # Errors
///
/// Returns the first rule the relationship breaks.
pub fn check_relation(
    document: &Document,
    kind: RelationKind,
    source: ElementId,
    target: ElementId,
) -> Result<(), DomainError> {
    for element in [source, target] {
        if !document.elements.contains_key(&element) {
            return Err(DomainError::MissingElement(element));
        }
    }
    match kind {
        RelationKind::Association if source == target => {
            Err(DomainError::SelfAssociationUnsupported)
        }
        RelationKind::Association => Ok(()),
        RelationKind::Generalization if source == target => Err(DomainError::SelfGeneralization),
        RelationKind::Generalization => {
            let duplicate = document.relations.values().any(|relation| {
                relation.kind == RelationKind::Generalization
                    && relation.source == source
                    && relation.target == target
            });
            if duplicate {
                Err(DomainError::DuplicateGeneralization {
                    child: source,
                    parent: target,
                })
            } else if would_cycle(document, source, target) {
                Err(DomainError::GeneralizationCycle {
                    child: source,
                    parent: target,
                })
            } else {
                Ok(())
            }
        }
    }
}

/// How serious a document issue is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// A modeling hint; never blocks saving.
    Warning,
    /// Structural damage (dangling identity, cycles); blocks saving.
    Error,
}

/// One finding of the whole-document check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Severity.
    pub severity: Severity,
    /// Explanation in words.
    pub message: String,
    /// The element it concerns, for Go to object.
    pub element: Option<ElementId>,
}

impl Issue {
    fn error(message: String, element: Option<ElementId>) -> Self {
        Self {
            severity: Severity::Error,
            message,
            element,
        }
    }
}

/// Checks the whole document: references, ownership, views and cycles.
pub fn check_document(document: &Document) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut member_ids = BTreeSet::new();
    for element in document.elements.values() {
        for member in &element.members {
            if !member_ids.insert(member.id()) {
                issues.push(Issue::error(
                    format!("member {} appears more than once", member.id()),
                    Some(element.id),
                ));
            }
        }
    }
    for relation in document.relations.values() {
        for end in [relation.source, relation.target] {
            if !document.elements.contains_key(&end) {
                issues.push(Issue::error(
                    format!("relationship {} points to missing {end}", relation.id),
                    None,
                ));
            }
        }
        if relation.kind == RelationKind::Generalization {
            if relation.source == relation.target {
                issues.push(Issue::error(
                    format!("{} is its own parent", relation.source),
                    Some(relation.source),
                ));
            } else if would_cycle_without(document, relation.id) {
                issues.push(Issue::error(
                    format!("generalization {} is part of a cycle", relation.id),
                    Some(relation.source),
                ));
            }
        }
    }
    for diagram in document.diagrams.values() {
        for node in &diagram.nodes {
            if !document.elements.contains_key(&node.element) {
                issues.push(Issue::error(
                    format!(
                        "view {} in {} shows missing {}",
                        node.id, diagram.name, node.element
                    ),
                    None,
                ));
            }
        }
        for edge in &diagram.edges {
            let relation = document.relations.get(&edge.relation);
            let source = diagram.node(edge.source);
            let target = diagram.node(edge.target);
            let consistent = match (relation, source, target) {
                (Some(relation), Some(source), Some(target)) => {
                    source.element == relation.source && target.element == relation.target
                }
                _ => false,
            };
            if !consistent {
                issues.push(Issue::error(
                    format!(
                        "connector {} in {} does not match its relationship",
                        edge.id, diagram.name
                    ),
                    None,
                ));
            }
        }
    }
    for element in document.elements.values() {
        if document.views_of(element.id).next().is_none() {
            issues.push(Issue {
                severity: Severity::Warning,
                message: format!("{} is not shown in any diagram", element.name),
                element: Some(element.id),
            });
        }
    }
    issues
}

/// Whether `relation` closes a cycle: its child is reachable from its
/// parent through the other generalizations.
fn would_cycle_without(document: &Document, relation: RelationId) -> bool {
    document
        .relations
        .get(&relation)
        .is_some_and(|this| reaches_ancestor(document, this.target, this.source, Some(relation)))
}
