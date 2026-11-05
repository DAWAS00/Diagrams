//! Design fixtures: one [`Presentation`] per screen state, used by the view
//! tests, the usability audit and evidence screenshots (task A-0).
//!
//! Every fixture is built by Agent B's [`presentation::build`] from an
//! [`AppState`], so labels, shortcut text, availability and reasons always
//! come from the command catalog; the UI never writes them. States the
//! sample project cannot reach yet (many tabs) adjust the built
//! presentation and are labeled "design fixture". Nothing here opens, saves
//! or changes a project.

use crate::app_state::AppState;
use crate::presentation::{self, Presentation, TabView};
use crate::ui::workspace::{Drawer, LayoutMode};

/// A class name longer than any box or row can show.
pub const LONG_NAME: &str =
    "PreferredCustomerAccountWithLoyaltyProgrammeAndRegionalBillingSettings";

/// The sample project with nothing selected.
pub fn sample_state() -> AppState {
    AppState::default()
}

/// The sample project with the class at `index` selected.
pub fn selected_state(index: usize) -> AppState {
    let mut state = sample_state();
    state.select(index);
    state
}

/// The sample workspace, docked, nothing selected.
pub fn sample() -> Presentation {
    presentation::build(&sample_state())
}

/// Customer selected.
pub fn class_selected() -> Presentation {
    presentation::build(&selected_state(0))
}

/// Customer selected with a blank name: the Name field shows its problem.
pub fn blank_name() -> Presentation {
    let mut state = selected_state(0);
    state.classes[0].name = String::new();
    presentation::build(&state)
}

/// Customer selected and renamed to [`LONG_NAME`].
pub fn long_name() -> Presentation {
    let mut state = selected_state(0);
    state.classes[0].name = LONG_NAME.to_owned();
    presentation::build(&state)
}

/// A new, empty project.
pub fn empty_diagram() -> Presentation {
    let mut state = sample_state();
    state.classes.clear();
    presentation::build(&state)
}

/// Both docked panels hidden by the user.
pub fn panels_hidden() -> Presentation {
    let mut state = sample_state();
    state.panels.navigator_visible = false;
    state.panels.inspector_visible = false;
    presentation::build(&state)
}

/// Narrow window with `drawer` open and Order selected.
pub fn narrow(drawer: Option<Drawer>) -> Presentation {
    let mut state = selected_state(1);
    state.layout = LayoutMode::Narrow;
    state.drawer = drawer;
    presentation::build(&state)
}

/// More tabs than fit at 800 wide. The sample project has one diagram, so
/// this design fixture adds tabs to the built presentation.
pub fn many_tabs() -> Presentation {
    let mut view = sample();
    let titles = [
        "Class overview",
        "Domain details",
        "Billing and invoices",
        "Customer accounts",
        "Shipping and returns",
        "Reporting model",
        "Archive",
    ];
    view.tabs.tabs = titles
        .iter()
        .enumerate()
        .map(|(index, title)| TabView {
            label: format!("{title} (design fixture)"),
            active: index == 2,
        })
        .collect();
    view
}

/// The sample project with the object search open and `query` typed,
/// reached through B's dispatcher exactly as Ctrl+F and typing would.
pub fn object_search(query: &str) -> Presentation {
    use crate::intent::{InputContext, SearchScope, UiIntent};
    use crate::ui::commands::CommandId;
    let mut state = sample_state();
    state.support.object_search = true;
    crate::dispatch::apply(
        &mut state,
        &InputContext::default(),
        [
            UiIntent::Command(CommandId::FindObject),
            UiIntent::Search {
                scope: SearchScope::Objects,
                query: query.to_owned(),
            },
        ],
    );
    presentation::build(&state)
}

/// The Resolve edit dialog for a blank class name.
pub fn resolve_edit() -> crate::presentation::DialogView {
    crate::presentation::DialogView::ResolveEdit {
        message: format!(
            "The Name field has an edit that cannot be kept: {}",
            crate::app_state::BLANK_NAME_WARNING
        ),
    }
}

/// Every named workspace state, for the view tests and the audit.
pub fn states() -> Vec<(&'static str, Presentation)> {
    vec![
        ("sample", sample()),
        ("class selected", class_selected()),
        ("blank name", blank_name()),
        ("long name", long_name()),
        ("empty diagram", empty_diagram()),
        ("panels hidden", panels_hidden()),
        ("many tabs", many_tabs()),
    ]
}
