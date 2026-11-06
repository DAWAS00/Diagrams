//! Project panel (navigator) and diagram tabs (UI-05): search matching and
//! which tabs fit before the overflow menu.
//!
//! The rows themselves, and the model-versus-view wording, come from
//! [`super::read_model`], which reads the real document (UI-13).

use diagram_core::ids::{DiagramId, ElementId};

/// An open diagram tab and the selection it keeps while another tab is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTab {
    /// The diagram shown in this tab.
    pub diagram: DiagramId,
    /// Selected class in this tab, restored when the tab becomes active again.
    pub selected: Option<ElementId>,
}

/// True when `text` contains `query`, ignoring case; an empty query matches everything.
pub fn matches(query: &str, text: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty() || text.to_lowercase().contains(&query)
}

/// Splits tabs (by `widths`) into those shown in `available` width and those
/// moved to the overflow menu. The active tab is always shown; the others
/// keep their order.
pub fn tabs_that_fit(
    widths: &[f32],
    available: f32,
    active: Option<usize>,
) -> (Vec<usize>, Vec<usize>) {
    let mut remaining = available
        - active
            .and_then(|index| widths.get(index))
            .copied()
            .unwrap_or(0.0);
    let mut shown = Vec::new();
    let mut overflow = Vec::new();
    for (index, width) in widths.iter().copied().enumerate() {
        if Some(index) == active {
            shown.push(index);
        } else if width <= remaining {
            remaining -= width;
            shown.push(index);
        } else {
            overflow.push(index);
            // Keep later tabs out too, so the visible order never skips.
            remaining = f32::NEG_INFINITY;
        }
    }
    (shown, overflow)
}
