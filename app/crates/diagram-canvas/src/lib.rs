//! Renderer-neutral canvas boundary for Diagram Studio.
//!
//! Reserved for geometry, hit testing, interaction previews and the scene shared
//! by screen rendering and SVG export. The core owns authoritative document
//! values; this crate will borrow them and derive visual output.
//!
//! No canvas operations are implemented in R0-01. The desktop prototype is the
//! next step; the validated scene and interaction pipeline arrive in M2.

#![deny(missing_docs)]
