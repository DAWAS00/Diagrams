//! Typed identities for model and view records (R1-01).
//!
//! Each kind of record has its own ID type, so a [`ViewId`] can never be
//! passed where an [`ElementId`] is expected:
//!
//! ```compile_fail
//! use diagram_core::ids::{ElementId, ViewId};
//! fn rename(_: ElementId) {}
//! rename(ViewId::from_raw(1)); // a view is not a model element
//! ```
//!
//! ```compile_fail
//! use diagram_core::ids::{DiagramId, RelationId};
//! let relation: RelationId = DiagramId::from_raw(1);
//! ```
//!
//! IDs are allocated by the document from one counter, so they are unique
//! within a project and never reused after deletion. Vector positions are
//! never identity.

use std::fmt;

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            /// The ID with this raw value (for storage and tests).
            pub const fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            /// The raw value, for storage.
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, concat!($prefix, "{}"), self.0)
            }
        }
    };
}

id_type!(
    /// A model element, such as a class. Shared by every view of it.
    ElementId,
    "element-"
);
id_type!(
    /// An attribute or operation of a class.
    MemberId,
    "member-"
);
id_type!(
    /// A semantic relationship between two elements.
    RelationId,
    "relation-"
);
id_type!(
    /// A diagram.
    DiagramId,
    "diagram-"
);
id_type!(
    /// One appearance of an element (node view) or relation (edge view) in a diagram.
    ViewId,
    "view-"
);
