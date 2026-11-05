//! Renderer-neutral canvas boundary for Diagram Studio.
//!
//! Pure geometry for the editor: the world ↔ screen [`Viewport`] transform
//! and [`picking`] in visible stacking order. The core owns authoritative
//! document values; this crate borrows them and derives visual output. It
//! never owns a writable document or writes files. The shared scene and SVG
//! output arrive in M2.

#![deny(missing_docs)]

pub mod picking;
pub mod viewport;

pub use viewport::{MAX_ZOOM, MIN_ZOOM, ScreenSize, Viewport, ZOOM_STEP};
