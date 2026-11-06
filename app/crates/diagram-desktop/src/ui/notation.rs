//! Notation capability and surface contract (UI-11).
//!
//! One compile-time table says what each diagram type offers: tools,
//! inspector sections, allowed relationships and its interaction model. The
//! shell reads the table; it is not a runtime plugin framework. Only
//! delivered notations can be created; planned ones document the surfaces
//! their milestones must provide.

use super::commands::CommandId;

/// The diagram types the product knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotationId {
    /// UML class diagram (v0.1).
    Class,
    /// Flowchart (M4).
    Flowchart,
    /// UML use case diagram (M4).
    UseCase,
    /// Entity-relationship diagram (M5).
    EntityRelationship,
    /// UML sequence diagram (M5).
    Sequence,
}

impl NotationId {
    /// The UI's notation for a document diagram's notation.
    pub const fn from_core(id: diagram_core::notation::NotationId) -> Self {
        use diagram_core::notation::NotationId as Core;
        match id {
            Core::Class => Self::Class,
            Core::Flowchart => Self::Flowchart,
            Core::UseCase => Self::UseCase,
            Core::EntityRelationship => Self::EntityRelationship,
            Core::Sequence => Self::Sequence,
        }
    }
}

/// Whether a notation can be used yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Implemented: offered when creating a diagram.
    Delivered,
    /// Planned for a milestone: never offered for creation.
    Planned(&'static str),
}

/// A tool in a notation's tool row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// A catalog command (delivered notations only).
    Command(CommandId),
    /// A tool a planned notation will need.
    Planned(&'static str),
}

/// A section of the inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorSection {
    /// Model element fields such as the name.
    Model,
    /// Ordered members (attributes, operations, columns).
    Members,
    /// Relationship ends: roles and multiplicities.
    Endpoints,
    /// Settings of this view only.
    Appearance,
    /// Rarely used settings.
    Advanced,
}

/// How shapes are placed and moved on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    /// Boxes placed freely; connections between them.
    Freeform,
    /// Time runs down the page; positions carry meaning (sequence diagrams).
    Temporal(&'static [&'static str]),
}

/// One notation's contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notation {
    /// Which notation.
    pub id: NotationId,
    /// Name shown when creating a diagram.
    pub name: &'static str,
    /// Delivered or planned.
    pub delivery: Delivery,
    /// Tool row contents, in order.
    pub tools: &'static [Tool],
    /// Inspector sections for an element, in order.
    pub element_sections: &'static [InspectorSection],
    /// Relationship kinds the notation allows.
    pub relationships: &'static [&'static str],
    /// How the canvas behaves.
    pub interaction: Interaction,
}

/// Requirements a sequence editor must meet; kept explicit so the freeform
/// class canvas is never forced onto it.
pub const SEQUENCE_REQUIREMENTS: [&str; 5] = [
    "Messages are ordered top to bottom; moving a message changes its order, not a free position",
    "Lifelines stay aligned in one row and extend down the whole diagram",
    "Activation bars follow the messages that start and end them",
    "Creation and destruction messages start or end a lifeline",
    "Combined fragments (alt, loop, opt) enclose a contiguous range of messages",
];

/// Every notation, delivered first.
pub const NOTATIONS: [Notation; 5] = [
    Notation {
        id: NotationId::Class,
        name: "Class diagram",
        delivery: Delivery::Delivered,
        tools: &[
            Tool::Command(CommandId::SelectTool),
            Tool::Command(CommandId::PanTool),
            Tool::Command(CommandId::AddClass),
            Tool::Command(CommandId::Connect),
            Tool::Command(CommandId::AddNote),
            Tool::Command(CommandId::Arrange),
        ],
        element_sections: &[
            InspectorSection::Model,
            InspectorSection::Members,
            InspectorSection::Appearance,
            InspectorSection::Advanced,
        ],
        relationships: &["Association", "Generalization"],
        interaction: Interaction::Freeform,
    },
    Notation {
        id: NotationId::Flowchart,
        name: "Flowchart",
        delivery: Delivery::Planned("M4"),
        tools: &[
            Tool::Planned("Process"),
            Tool::Planned("Decision"),
            Tool::Planned("Start or end"),
            Tool::Planned("Input or output"),
            Tool::Planned("Flow"),
        ],
        element_sections: &[InspectorSection::Model, InspectorSection::Appearance],
        relationships: &["Control flow"],
        interaction: Interaction::Freeform,
    },
    Notation {
        id: NotationId::UseCase,
        name: "Use case diagram",
        delivery: Delivery::Planned("M4"),
        tools: &[
            Tool::Planned("Actor"),
            Tool::Planned("Use case"),
            Tool::Planned("System boundary"),
            Tool::Planned("Association"),
        ],
        element_sections: &[
            InspectorSection::Model,
            InspectorSection::Appearance,
            InspectorSection::Advanced,
        ],
        relationships: &["Association", "Include", "Extend", "Generalization"],
        interaction: Interaction::Freeform,
    },
    Notation {
        id: NotationId::EntityRelationship,
        name: "Entity-relationship diagram",
        delivery: Delivery::Planned("M5"),
        tools: &[Tool::Planned("Entity"), Tool::Planned("Relationship")],
        element_sections: &[
            InspectorSection::Model,
            InspectorSection::Members,
            InspectorSection::Appearance,
        ],
        relationships: &["One to one", "One to many", "Many to many"],
        interaction: Interaction::Freeform,
    },
    Notation {
        id: NotationId::Sequence,
        name: "Sequence diagram",
        delivery: Delivery::Planned("M5"),
        tools: &[
            Tool::Planned("Lifeline"),
            Tool::Planned("Message"),
            Tool::Planned("Reply"),
            Tool::Planned("Fragment"),
        ],
        element_sections: &[InspectorSection::Model, InspectorSection::Appearance],
        relationships: &["Synchronous message", "Asynchronous message", "Reply"],
        interaction: Interaction::Temporal(&SEQUENCE_REQUIREMENTS),
    },
];

/// The contract of `id`.
pub fn notation(id: NotationId) -> &'static Notation {
    match id {
        NotationId::Class => &NOTATIONS[0],
        NotationId::Flowchart => &NOTATIONS[1],
        NotationId::UseCase => &NOTATIONS[2],
        NotationId::EntityRelationship => &NOTATIONS[3],
        NotationId::Sequence => &NOTATIONS[4],
    }
}

/// Notations a user can create today: delivered ones only.
pub fn creatable() -> impl Iterator<Item = &'static Notation> {
    NOTATIONS
        .iter()
        .filter(|notation| notation.delivery == Delivery::Delivered)
}

/// The catalog commands in a notation's tool row, in order.
pub fn tool_commands(id: NotationId) -> impl Iterator<Item = CommandId> {
    notation(id).tools.iter().filter_map(|tool| match tool {
        Tool::Command(command) => Some(*command),
        Tool::Planned(_) => None,
    })
}
