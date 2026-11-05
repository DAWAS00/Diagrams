//! What each diagram notation offers: tools, inspector sections,
//! relationship kinds and capabilities (UI-11).
//!
//! Notations are compiled-in descriptions, not runtime plugins. Only
//! [`Notation::delivered`] notations can be created; the others record the
//! surfaces they will need, so unsupported families never appear in the
//! creation UI.

/// A diagram notation family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotationId {
    /// UML class diagram (v0.1).
    Class,
    /// Flowchart (later).
    Flowchart,
    /// Use case diagram (later).
    UseCase,
    /// Entity-relationship diagram (later).
    EntityRelationship,
    /// Sequence diagram (later; needs its own interaction strategy).
    Sequence,
}

/// A tool a notation puts in the tool row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolKind {
    /// Select and move.
    Select,
    /// Pan the view.
    Hand,
    /// Place a class.
    Class,
    /// Connect two elements; the kind is chosen separately.
    Relationship,
    /// Arrange: align and distribute.
    Arrange,
}

/// A relationship a notation can create.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// Association between two classes, with ends.
    Association,
    /// Child-to-parent generalization; cycles are rejected.
    Generalization,
}

/// A section of the inspector for a selected element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorSection {
    /// Name and other shared model fields.
    Model,
    /// Attributes and operations.
    Members,
    /// View-only appearance: position, size, fill, border.
    Appearance,
    /// Relationship ends, roles and multiplicities.
    RelationEnds,
}

/// What one notation offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notation {
    /// The notation.
    pub id: NotationId,
    /// Name in the Add diagram picker.
    pub display_name: &'static str,
    /// Delivered in this release: it can be created.
    pub delivered: bool,
    /// Tools in tool-row order.
    pub tools: &'static [ToolKind],
    /// Relationship kinds the Relationship tool offers.
    pub relation_kinds: &'static [RelationKind],
    /// Inspector sections for its elements.
    pub inspector_sections: &'static [InspectorSection],
    /// Interaction requirements not yet met, recorded for later notations.
    pub open_requirements: &'static str,
}

/// Every notation, delivered or planned.
pub const NOTATIONS: [Notation; 5] = [
    Notation {
        id: NotationId::Class,
        display_name: "Class diagram",
        delivered: true,
        tools: &[
            ToolKind::Select,
            ToolKind::Hand,
            ToolKind::Class,
            ToolKind::Relationship,
            ToolKind::Arrange,
        ],
        relation_kinds: &[RelationKind::Association, RelationKind::Generalization],
        inspector_sections: &[
            InspectorSection::Model,
            InspectorSection::Members,
            InspectorSection::Appearance,
            InspectorSection::RelationEnds,
        ],
        open_requirements: "",
    },
    Notation {
        id: NotationId::Flowchart,
        display_name: "Flowchart",
        delivered: false,
        tools: &[],
        relation_kinds: &[],
        inspector_sections: &[],
        open_requirements: "Process, decision and terminator shapes reusing class-diagram primitives.",
    },
    Notation {
        id: NotationId::UseCase,
        display_name: "Use case diagram",
        delivered: false,
        tools: &[],
        relation_kinds: &[],
        inspector_sections: &[],
        open_requirements: "Actors, use cases, system boundary and include/extend relations.",
    },
    Notation {
        id: NotationId::EntityRelationship,
        display_name: "Entity-relationship diagram",
        delivered: false,
        tools: &[],
        relation_kinds: &[],
        inspector_sections: &[],
        open_requirements: "Entities, keys and cardinality ends.",
    },
    Notation {
        id: NotationId::Sequence,
        display_name: "Sequence diagram",
        delivered: false,
        tools: &[],
        relation_kinds: &[],
        inspector_sections: &[],
        open_requirements: "Lifelines, ordered messages and activation bars; time-ordered \
                            layout instead of free placement, so it needs its own interaction \
                            strategy.",
    },
];

impl Notation {
    /// The description of `id`.
    pub fn get(id: NotationId) -> &'static Self {
        match id {
            NotationId::Class => &NOTATIONS[0],
            NotationId::Flowchart => &NOTATIONS[1],
            NotationId::UseCase => &NOTATIONS[2],
            NotationId::EntityRelationship => &NOTATIONS[3],
            NotationId::Sequence => &NOTATIONS[4],
        }
    }

    /// The notation offers `tool`.
    pub fn has_tool(&self, tool: ToolKind) -> bool {
        self.tools.contains(&tool)
    }
}

/// Notations that can be created now, for the Add diagram picker.
pub fn creatable() -> impl Iterator<Item = &'static Notation> {
    NOTATIONS.iter().filter(|notation| notation.delivered)
}
