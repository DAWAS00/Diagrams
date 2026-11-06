//! Rebuilding a validated document from rows.
//!
//! Every row goes back through the core's own rules (`Name`, `Multiplicity`,
//! `Rect`, `check_document`), so a damaged or tampered file can never put an
//! invalid document into live state.

use std::collections::BTreeSet;

use diagram_core::geometry::Rect;
use diagram_core::ids::{DiagramId, ElementId, MemberId, RelationId, ViewId};
use diagram_core::model::{
    AssociationEnd, Attribute, Diagram, Document, EdgeView, Element, Member, Multiplicity, Name,
    NodeView, Operation, Parameter, Relation,
};
use diagram_core::validate::{Severity, check_document};

use crate::dto::{
    DiagramRow, EdgeRow, MemberRow, NodeRow, ParameterRow, ProjectDto, parse_kind, parse_notation,
    parse_visibility,
};
use crate::error::LoadError;

impl ProjectDto {
    /// Checks the shape of the rows before any record is built: positions
    /// used twice, parameters on attributes, a class shown twice in one
    /// diagram, repeated generalizations and ends on a generalization.
    fn check_structure(&self, problems: &mut Vec<String>) {
        let mut twice = |what: &str, keys: Vec<(u64, u32)>| {
            let mut seen = BTreeSet::new();
            for key in keys {
                if !seen.insert(key) {
                    problems.push(format!("two {what} share the position {}", key.1));
                }
            }
        };
        twice(
            "members of one class",
            self.members
                .iter()
                .map(|row| (row.element_id, row.position))
                .collect(),
        );
        twice(
            "parameters of one operation",
            self.parameters
                .iter()
                .map(|row| (row.member_id, row.position))
                .collect(),
        );
        twice(
            "diagrams",
            self.diagrams.iter().map(|row| (0, row.position)).collect(),
        );
        twice(
            "views of one diagram",
            self.nodes
                .iter()
                .map(|row| (row.diagram_id, row.position))
                .collect(),
        );
        twice(
            "connectors of one diagram",
            self.edges
                .iter()
                .map(|row| (row.diagram_id, row.position))
                .collect(),
        );
        for parameter in &self.parameters {
            let owner = self
                .members
                .iter()
                .find(|row| row.id == parameter.member_id);
            if owner.is_some_and(|row| row.kind != "operation") {
                problems.push(format!(
                    "member {} is not an operation but has parameters",
                    parameter.member_id
                ));
            }
        }
        let mut shown = BTreeSet::new();
        for node in &self.nodes {
            if !shown.insert((node.diagram_id, node.element_id)) {
                problems.push(format!(
                    "class {} is shown twice in diagram {}",
                    node.element_id, node.diagram_id
                ));
            }
        }
        let mut generalizations = BTreeSet::new();
        for relation in &self.relations {
            if relation.kind != "generalization" {
                continue;
            }
            if !generalizations.insert((relation.source, relation.target)) {
                problems.push(format!(
                    "class {} generalizes class {} more than once",
                    relation.source, relation.target
                ));
            }
            if relation.source_role.is_some()
                || relation.target_role.is_some()
                || relation.source_multiplicity.is_some()
                || relation.target_multiplicity.is_some()
            {
                problems.push(format!(
                    "generalization {} has roles or multiplicities, which only associations have",
                    relation.id
                ));
            }
        }
    }

    /// Rebuilds and validates the document.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError::Invalid`] listing every problem found, or
    /// [`LoadError::Damaged`] when the rows cannot be a project at all (an
    /// ID used twice, or an ID counter below the largest ID).
    pub fn into_document(self) -> Result<(String, Document), LoadError> {
        let mut problems = Vec::new();
        let mut seen = BTreeSet::new();
        let mut largest = 0_u64;
        let mut claim = |id: u64, what: &str, problems: &mut Vec<String>| {
            largest = largest.max(id);
            if !seen.insert(id) {
                problems.push(format!("the ID {id} is used twice ({what})"));
            }
        };
        let mut document = Document::default();
        self.check_structure(&mut problems);

        for row in &self.elements {
            claim(row.id, "class", &mut problems);
            match Name::new(row.name.clone()) {
                Ok(name) => {
                    document.elements.insert(
                        ElementId::from_raw(row.id),
                        Element {
                            id: ElementId::from_raw(row.id),
                            name,
                            members: Vec::new(),
                        },
                    );
                }
                Err(_) => problems.push(format!("class {} has a blank name", row.id)),
            }
        }

        let mut members: Vec<&MemberRow> = self.members.iter().collect();
        members.sort_by_key(|row| (row.element_id, row.position));
        for row in members {
            claim(row.id, "member", &mut problems);
            let Some(element) = document
                .elements
                .get_mut(&ElementId::from_raw(row.element_id))
            else {
                problems.push(format!(
                    "member {} belongs to the missing class {}",
                    row.id, row.element_id
                ));
                continue;
            };
            let (Ok(name), Some(visibility)) = (
                Name::new(row.name.clone()),
                parse_visibility(&row.visibility),
            ) else {
                problems.push(format!(
                    "member {} has a blank name or unknown visibility",
                    row.id
                ));
                continue;
            };
            let id = MemberId::from_raw(row.id);
            match row.kind.as_str() {
                "attribute" => {
                    let multiplicity = match row.multiplicity.as_deref() {
                        None => None,
                        Some(text) => match Multiplicity::parse(text) {
                            Ok(value) => Some(value),
                            Err(error) => {
                                problems.push(format!("attribute {}: {error}", row.id));
                                continue;
                            }
                        },
                    };
                    element.members.push(Member::Attribute(Attribute {
                        id,
                        name,
                        type_text: row.type_text.clone().unwrap_or_default(),
                        visibility,
                        multiplicity,
                    }));
                }
                "operation" => {
                    let mut parameters: Vec<&ParameterRow> = self
                        .parameters
                        .iter()
                        .filter(|parameter| parameter.member_id == row.id)
                        .collect();
                    parameters.sort_by_key(|parameter| parameter.position);
                    let mut list = Vec::new();
                    for parameter in parameters {
                        match Name::new(parameter.name.clone()) {
                            Ok(name) => list.push(Parameter {
                                name,
                                type_text: parameter.type_text.clone(),
                            }),
                            Err(_) => problems.push(format!(
                                "a parameter of operation {} has a blank name",
                                row.id
                            )),
                        }
                    }
                    element.members.push(Member::Operation(Operation {
                        id,
                        name,
                        visibility,
                        parameters: list,
                        return_type: row.return_type.clone(),
                    }));
                }
                other => problems.push(format!("member {} has the unknown kind {other:?}", row.id)),
            }
        }
        for parameter in &self.parameters {
            if !self.members.iter().any(|row| row.id == parameter.member_id) {
                problems.push(format!(
                    "a parameter belongs to the missing operation {}",
                    parameter.member_id
                ));
            }
        }

        for row in &self.relations {
            claim(row.id, "relationship", &mut problems);
            let Some(kind) = parse_kind(&row.kind) else {
                problems.push(format!(
                    "relationship {} has the unknown kind {:?}",
                    row.id, row.kind
                ));
                continue;
            };
            let end = |role: &Option<String>,
                       multiplicity: &Option<String>,
                       problems: &mut Vec<String>| {
                let role = match role.as_deref() {
                    None => None,
                    Some(text) => match Name::new(text) {
                        Ok(name) => Some(name),
                        Err(_) => {
                            problems.push(format!("relationship {} has a blank role", row.id));
                            None
                        }
                    },
                };
                let multiplicity = match multiplicity.as_deref() {
                    None => None,
                    Some(text) => match Multiplicity::parse(text) {
                        Ok(value) => Some(value),
                        Err(error) => {
                            problems.push(format!("relationship {}: {error}", row.id));
                            None
                        }
                    },
                };
                AssociationEnd { role, multiplicity }
            };
            let source_end = end(&row.source_role, &row.source_multiplicity, &mut problems);
            let target_end = end(&row.target_role, &row.target_multiplicity, &mut problems);
            document.relations.insert(
                RelationId::from_raw(row.id),
                Relation {
                    id: RelationId::from_raw(row.id),
                    kind,
                    source: ElementId::from_raw(row.source),
                    target: ElementId::from_raw(row.target),
                    source_end,
                    target_end,
                },
            );
        }

        let mut diagrams: Vec<&DiagramRow> = self.diagrams.iter().collect();
        diagrams.sort_by_key(|row| row.position);
        for row in diagrams {
            claim(row.id, "diagram", &mut problems);
            let (Ok(name), Some(notation)) =
                (Name::new(row.name.clone()), parse_notation(&row.notation))
            else {
                problems.push(format!(
                    "diagram {} has a blank name or unknown notation",
                    row.id
                ));
                continue;
            };
            let id = DiagramId::from_raw(row.id);
            let mut nodes = Vec::new();
            let mut node_rows: Vec<&NodeRow> = self
                .nodes
                .iter()
                .filter(|node| node.diagram_id == row.id)
                .collect();
            node_rows.sort_by_key(|node| node.position);
            for node in node_rows {
                claim(node.id, "view", &mut problems);
                match Rect::from_xywh(node.x, node.y, node.width, node.height) {
                    Ok(bounds) => nodes.push(NodeView {
                        id: ViewId::from_raw(node.id),
                        element: ElementId::from_raw(node.element_id),
                        bounds,
                    }),
                    Err(error) => problems.push(format!("view {}: {error}", node.id)),
                }
            }
            let mut edge_rows: Vec<&EdgeRow> = self
                .edges
                .iter()
                .filter(|edge| edge.diagram_id == row.id)
                .collect();
            edge_rows.sort_by_key(|edge| edge.position);
            let edges = edge_rows
                .into_iter()
                .map(|edge| {
                    claim(edge.id, "connector", &mut problems);
                    EdgeView {
                        id: ViewId::from_raw(edge.id),
                        relation: RelationId::from_raw(edge.relation_id),
                        source: ViewId::from_raw(edge.source_view),
                        target: ViewId::from_raw(edge.target_view),
                    }
                })
                .collect();
            document.diagrams.insert(
                id,
                Diagram {
                    id,
                    name,
                    notation,
                    nodes,
                    edges,
                },
            );
            document.diagram_order.push(id);
        }
        for node in &self.nodes {
            if !self.diagrams.iter().any(|row| row.id == node.diagram_id) {
                problems.push(format!(
                    "view {} is in the missing diagram {}",
                    node.id, node.diagram_id
                ));
            }
        }
        for edge in &self.edges {
            if !self.diagrams.iter().any(|row| row.id == edge.diagram_id) {
                problems.push(format!(
                    "connector {} is in the missing diagram {}",
                    edge.id, edge.diagram_id
                ));
            }
        }

        if i64::try_from(self.next_id).is_err() {
            return Err(LoadError::Damaged {
                reason: format!("the ID counter {} is too large to be real", self.next_id),
            });
        }
        if self.next_id < largest {
            return Err(LoadError::Damaged {
                reason: format!(
                    "the ID counter {} is below the largest ID {largest}, so IDs could be reused",
                    self.next_id
                ),
            });
        }
        document.next_id = self.next_id;
        if problems.is_empty() {
            problems.extend(
                check_document(&document)
                    .into_iter()
                    .filter(|issue| issue.severity == Severity::Error)
                    .map(|issue| issue.message),
            );
        }
        if problems.is_empty() {
            Ok((self.name, document))
        } else {
            Err(LoadError::Invalid(problems))
        }
    }
}
