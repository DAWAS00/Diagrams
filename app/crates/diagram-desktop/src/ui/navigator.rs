//! Project panel (navigator) and diagram tabs (UI-05): search matching,
//! model-versus-view wording, and which tabs fit before the overflow menu.
//!
//! Diagrams here are prototype fixtures: a name and the classes it shows.
//! Real diagrams, views and sessions arrive with M1 and are bound in UI-13.

/// A diagram in the prototype project: a view of some of the model's classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    /// Diagram name, shown in the project panel and on its tab.
    pub name: String,
    /// Indices of the classes this diagram shows.
    pub classes: Vec<usize>,
}

impl Diagram {
    /// A diagram named `name` showing `classes`.
    pub fn new(name: &str, classes: &[usize]) -> Self {
        Self {
            name: name.to_owned(),
            classes: classes.to_vec(),
        }
    }
}

/// An open diagram tab and the selection it keeps while another tab is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTab {
    /// Index of the diagram shown in this tab.
    pub diagram: usize,
    /// Selected class in this tab, restored when the tab becomes active again.
    pub selected: Option<usize>,
}

/// True when `text` contains `query`, ignoring case; an empty query matches everything.
pub fn matches(query: &str, text: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty() || text.to_lowercase().contains(&query)
}

/// Indices of diagrams that show the class at `class`.
pub fn diagrams_showing(diagrams: &[Diagram], class: usize) -> Vec<usize> {
    diagrams
        .iter()
        .enumerate()
        .filter(|(_, diagram)| diagram.classes.contains(&class))
        .map(|(index, _)| index)
        .collect()
}

/// Model-versus-view wording for a class: which diagrams show it.
pub fn shown_in(diagrams: &[Diagram], class: usize) -> String {
    let names: Vec<&str> = diagrams_showing(diagrams, class)
        .into_iter()
        .map(|index| diagrams[index].name.as_str())
        .collect();
    match names.as_slice() {
        [] => String::from("Not shown in any diagram"),
        [only] => format!("Shown in {only}"),
        [first @ .., last] => format!("Shown in {} and {last}", first.join(", ")),
    }
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
