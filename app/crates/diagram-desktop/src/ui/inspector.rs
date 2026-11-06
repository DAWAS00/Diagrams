//! Contextual inspector (UI-08): what the inspector edits and the rules for
//! committing it.
//!
//! Fields edit a **draft**. Enter, or moving focus away, commits a valid
//! draft as a typed intent; Escape cancels and restores the committed value;
//! an invalid draft stays editable with its error and is never committed.
//! Widgets never write the class or relationship records themselves: the
//! coordinator turns a committed draft into a `diagram-core` command.

use diagram_core::command::EndInput;
use diagram_core::ids::{ElementId, RelationId};
use diagram_core::model::{Multiplicity, Name, Relation};

/// An editable field of an association.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RelationField {
    /// Role name at the source end.
    SourceRole,
    /// Multiplicity at the source end.
    SourceMultiplicity,
    /// Role name at the target end.
    TargetRole,
    /// Multiplicity at the target end.
    TargetMultiplicity,
}

impl RelationField {
    /// Every field, in inspector order.
    pub const ALL: [Self; 4] = [
        Self::SourceRole,
        Self::SourceMultiplicity,
        Self::TargetRole,
        Self::TargetMultiplicity,
    ];

    /// The field's current value in `relation`; empty when it has none.
    pub fn value(self, relation: &Relation) -> String {
        match self {
            Self::SourceRole => role_text(relation.source_end.role.as_ref()),
            Self::SourceMultiplicity => multiplicity_text(relation.source_end.multiplicity),
            Self::TargetRole => role_text(relation.target_end.role.as_ref()),
            Self::TargetMultiplicity => multiplicity_text(relation.target_end.multiplicity),
        }
    }

    /// The error for `value`, if it cannot be committed.
    pub fn error(self, value: &str) -> Option<String> {
        match self {
            Self::SourceMultiplicity | Self::TargetMultiplicity => multiplicity_error(value),
            Self::SourceRole | Self::TargetRole => None,
        }
    }

    /// Whether committing `value` would leave the field as it is. Multiplicities
    /// compare by meaning, so `0..*` and `*` are the same.
    pub fn unchanged(self, relation: &Relation, value: &str) -> bool {
        let current = self.value(relation);
        let value = value.trim();
        match self {
            Self::SourceMultiplicity | Self::TargetMultiplicity => {
                match (Multiplicity::parse(&current), Multiplicity::parse(value)) {
                    (Ok(left), Ok(right)) => left == right,
                    _ => current == value,
                }
            }
            Self::SourceRole | Self::TargetRole => current == value,
        }
    }

    /// Both ends of `relation` as command input, with this field set to `value`.
    pub fn ends_with(self, relation: &Relation, value: &str) -> (EndInput, EndInput) {
        let mut source = end_input(&Self::SourceRole, &Self::SourceMultiplicity, relation);
        let mut target = end_input(&Self::TargetRole, &Self::TargetMultiplicity, relation);
        let value = value.trim().to_owned();
        match self {
            Self::SourceRole => source.role = value,
            Self::SourceMultiplicity => source.multiplicity = value,
            Self::TargetRole => target.role = value,
            Self::TargetMultiplicity => target.multiplicity = value,
        }
        (source, target)
    }
}

fn end_input(role: &RelationField, multiplicity: &RelationField, relation: &Relation) -> EndInput {
    EndInput {
        role: role.value(relation),
        multiplicity: multiplicity.value(relation),
    }
}

fn role_text(role: Option<&Name>) -> String {
    role.map(|name| name.as_str().to_owned())
        .unwrap_or_default()
}

fn multiplicity_text(multiplicity: Option<Multiplicity>) -> String {
    multiplicity
        .map(|value| value.to_string())
        .unwrap_or_default()
}

/// Error for a multiplicity draft. Blank is allowed and means none; anything
/// else must be a number, `*`, or a range such as `0..1`, `1..*` or `2..5`
/// (the rule lives in `diagram-core`).
pub fn multiplicity_error(value: &str) -> Option<String> {
    const RULE: &str = "A multiplicity is a number, *, or a range such as 0..1, 1..* or 2..5.";
    let value = value.trim();
    (!value.is_empty() && Multiplicity::parse(value).is_err()).then(|| String::from(RULE))
}

/// Error for a class name draft, if it cannot be committed.
pub fn class_name_error(value: &str) -> Option<String> {
    Name::new(value)
        .is_err()
        .then(|| String::from(crate::shell::BLANK_NAME_WARNING))
}

/// Key of a draft in the inspector's draft table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DraftKey {
    /// The name of a class.
    ClassName(ElementId),
    /// A field of an association.
    Relation(RelationId, RelationField),
}

/// What happened to a draft field this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftOutcome {
    /// Still editing, or nothing happened.
    Editing,
    /// The user committed this valid value.
    Commit(String),
    /// The user cancelled; the committed value is restored.
    Cancel,
}
