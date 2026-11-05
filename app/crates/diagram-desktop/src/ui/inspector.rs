//! Contextual inspector (UI-08): what the inspector edits and the rules for
//! committing it.
//!
//! Fields edit a **draft**. Enter, or moving focus away, commits a valid
//! draft as a typed intent; Escape cancels and restores the committed value;
//! an invalid draft stays editable with its error and is never committed.
//! Widgets never write the class or relationship records themselves.

/// A relationship between two classes (prototype fixture until M1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRelation {
    /// Relationship name, such as "places".
    pub name: String,
    /// Class at the source end.
    pub source: usize,
    /// Class at the target end.
    pub target: usize,
    /// Role name at the source end.
    pub source_role: String,
    /// Multiplicity at the source end, such as "1".
    pub source_multiplicity: String,
    /// Role name at the target end.
    pub target_role: String,
    /// Multiplicity at the target end, such as "0..*".
    pub target_multiplicity: String,
}

/// An editable field of a relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RelationField {
    /// The relationship name.
    Name,
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
    pub const ALL: [Self; 5] = [
        Self::Name,
        Self::SourceRole,
        Self::SourceMultiplicity,
        Self::TargetRole,
        Self::TargetMultiplicity,
    ];

    /// The field's current value in `relation`.
    pub fn value(self, relation: &PreviewRelation) -> &str {
        match self {
            Self::Name => &relation.name,
            Self::SourceRole => &relation.source_role,
            Self::SourceMultiplicity => &relation.source_multiplicity,
            Self::TargetRole => &relation.target_role,
            Self::TargetMultiplicity => &relation.target_multiplicity,
        }
    }

    /// Mutable access, used only by the shell's coordinator after validation.
    pub fn value_mut(self, relation: &mut PreviewRelation) -> &mut String {
        match self {
            Self::Name => &mut relation.name,
            Self::SourceRole => &mut relation.source_role,
            Self::SourceMultiplicity => &mut relation.source_multiplicity,
            Self::TargetRole => &mut relation.target_role,
            Self::TargetMultiplicity => &mut relation.target_multiplicity,
        }
    }

    /// The error for `value`, if it cannot be committed.
    pub fn error(self, value: &str) -> Option<String> {
        match self {
            Self::SourceMultiplicity | Self::TargetMultiplicity => multiplicity_error(value),
            Self::Name | Self::SourceRole | Self::TargetRole => None,
        }
    }
}

/// The sample relationship: Customer places 0..* Orders.
pub fn sample_relation() -> PreviewRelation {
    PreviewRelation {
        name: String::from("places"),
        source: 0,
        target: 1,
        source_role: String::from("customer"),
        source_multiplicity: String::from("1"),
        target_role: String::from("orders"),
        target_multiplicity: String::from("0..*"),
    }
}

/// Error for a UML multiplicity such as `1`, `0..1`, `2..5`, `1..*` or `*`.
pub fn multiplicity_error(value: &str) -> Option<String> {
    const RULE: &str = "A multiplicity is a number, *, or a range such as 0..1, 1..* or 2..5.";
    let value = value.trim();
    let number = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    let valid = match value.split_once("..") {
        None => value == "*" || number(value),
        Some((low, "*")) => number(low),
        Some((low, high)) => {
            number(low)
                && number(high)
                && low
                    .parse::<u64>()
                    .ok()
                    .zip(high.parse::<u64>().ok())
                    .is_some_and(|(low, high)| low <= high && high > 0)
        }
    };
    (!valid).then(|| String::from(RULE))
}

/// Error for a class name draft, if it cannot be committed.
pub fn class_name_error(value: &str) -> Option<String> {
    value
        .trim()
        .is_empty()
        .then(|| String::from(crate::shell::BLANK_NAME_WARNING))
}

/// Key of a draft in the inspector's draft table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DraftKey {
    /// A class name.
    ClassName(usize),
    /// A field of a relationship.
    Relation(usize, RelationField),
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
