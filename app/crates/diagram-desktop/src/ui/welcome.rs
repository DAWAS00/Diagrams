//! Welcome and project entry screens (UI-04).
//!
//! Recent projects and the recovery notice are labeled sample entries: this
//! prototype has no project storage, so Open and Review never touch real
//! files and say why. The new-project name is validated by the core
//! [`ProjectName`] rule.

use diagram_core::ProjectName;
use eframe::egui::{self, Id, Response};

use super::components;

/// Explains every file action that needs project storage.
pub const NEEDS_STORAGE: &str =
    "Not available yet: opening project files arrives with project storage.";
/// Explains why recovery cannot be reviewed yet.
pub const NEEDS_RECOVERY: &str = "Not available yet: recovery arrives with project storage.";
/// The diagram types that can be created today.
pub const CREATABLE_TYPES: [&str; 1] = ["Class diagram"];

/// Which entry screen is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryScreen {
    /// New, open, recent projects and recovery.
    Welcome,
    /// The new-project form.
    NewProject,
}

/// Whether a recent project's file can still be found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecentStatus {
    /// The file exists.
    Available,
    /// The file is no longer at its path.
    Missing,
}

/// One entry in the recent-projects list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentProject {
    /// Project name.
    pub name: String,
    /// Where the project file was saved.
    pub path: String,
    /// Whether the file can be found.
    pub status: RecentStatus,
}

/// Labeled sample entries shown until project storage exists.
pub fn sample_recent_projects() -> Vec<RecentProject> {
    let entry = |name: &str, path: &str, status| RecentProject {
        name: name.to_owned(),
        path: path.to_owned(),
        status,
    };
    vec![
        entry(
            "Customer model",
            r"D:\Projects\customer-model.dgram",
            RecentStatus::Available,
        ),
        entry(
            "Order workflow",
            r"D:\Projects\order-workflow.dgram",
            RecentStatus::Available,
        ),
        entry(
            "Old inventory",
            r"E:\Archive\inventory.dgram",
            RecentStatus::Missing,
        ),
    ]
}

/// Indices of recent projects whose name or path contains `query`, ignoring case.
pub fn matching_recent(recent: &[RecentProject], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    recent
        .iter()
        .enumerate()
        .filter(|(_, project)| {
            query.is_empty()
                || project.name.to_lowercase().contains(&query)
                || project.path.to_lowercase().contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

/// The error to show beside the project-name field, if any.
pub fn name_error(name: &str) -> Option<String> {
    ProjectName::new(name.to_owned()).err().map(|error| {
        let message = error.to_string();
        let mut letters = message.chars();
        match letters.next() {
            Some(first) => format!("{}{}.", first.to_uppercase(), letters.as_str()),
            None => message,
        }
    })
}

/// State of the entry screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryState {
    /// The screen being shown.
    pub screen: EntryScreen,
    /// Recent projects (sample entries).
    pub recent: Vec<RecentProject>,
    /// Text in the recent-projects search field.
    pub search: String,
    /// Whether the recovery notice is shown.
    pub recovery_visible: bool,
    /// Name typed into the new-project form.
    pub new_name: String,
    /// The name field has been edited or Create was tried, so errors show.
    pub name_touched: bool,
    /// Give the name field focus on the next frame.
    pub focus_name: bool,
    /// The "New project" button, which gets focus back after Cancel.
    pub new_button: Option<Id>,
}

impl Default for EntryState {
    fn default() -> Self {
        Self {
            screen: EntryScreen::Welcome,
            recent: sample_recent_projects(),
            search: String::new(),
            recovery_visible: true,
            new_name: String::new(),
            name_touched: false,
            focus_name: false,
            new_button: None,
        }
    }
}

/// What the user asked for on an entry screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryAction {
    /// Show the new-project form.
    StartNewProject,
    /// Leave the form and return to the welcome screen.
    CancelNewProject,
    /// Create a project with this validated name and open its empty diagram.
    CreateProject(String),
    /// Open the built-in sample project.
    OpenSample,
    /// Remove a recent entry (by index) from the list.
    RemoveRecent(usize),
    /// Hide the recovery notice.
    DismissRecovery,
}

/// Draws the welcome screen and returns what the user chose, if anything.
pub fn welcome(ui: &mut egui::Ui, state: &mut EntryState) -> Option<EntryAction> {
    let mut action = None;
    components::page_title(ui, crate::APPLICATION_NAME);
    ui.label("Create a class diagram project, or continue with the sample project.");
    ui.add_space(super::theme::SPACING);
    ui.horizontal(|ui| {
        let new = components::action_button(ui, "New project", None, None);
        state.new_button = Some(new.id);
        if new.clicked() {
            action = Some(EntryAction::StartNewProject);
        }
        let _ = components::action_button(ui, "Open project…", Some(NEEDS_STORAGE), None);
        if components::action_button(ui, "Continue with the sample project", None, None).clicked() {
            action = Some(EntryAction::OpenSample);
        }
    });

    if state.recovery_visible {
        ui.add_space(super::theme::SPACING);
        components::panel_title(ui, "Recovered changes");
        components::field_label(
            ui,
            "Sample notice (design fixture): nothing was actually recovered.",
        );
        components::warning_text(
            ui,
            "Unsaved changes to Customer model were recovered from 4 November 2026, 18:20.",
        );
        ui.horizontal(|ui| {
            let _ = components::action_button(ui, "Review recovery", Some(NEEDS_RECOVERY), None);
            if components::action_button(ui, "Dismiss recovery", None, None).clicked() {
                action = Some(EntryAction::DismissRecovery);
            }
        });
    }

    ui.add_space(super::theme::SPACING);
    components::panel_title(ui, "Recent projects");
    components::field_label(
        ui,
        "Sample entries (design fixture): this prototype cannot open project files.",
    );
    components::text_field(ui, "Search recent projects", &mut state.search);
    let matches = matching_recent(&state.recent, &state.search);
    if state.recent.is_empty() {
        ui.label("No recent projects yet.");
    } else if matches.is_empty() {
        ui.label(format!(
            "No recent projects match \"{}\".",
            state.search.trim()
        ));
    }
    for index in matches {
        let project = &state.recent[index];
        ui.add_space(super::theme::GAP);
        let label = format!("{} — {}", project.name, project.path);
        match project.status {
            RecentStatus::Available => {
                let _ = components::action_button(ui, &label, Some(NEEDS_STORAGE), None);
            }
            RecentStatus::Missing => {
                components::truncated_text(ui, &label);
                // The error and its action share a line (wrapping when
                // narrow), so the list still fits at 800×500.
                ui.horizontal_wrapped(|ui| {
                    components::error_text(ui, "File not found; it may have moved.");
                    let remove = format!("Remove {} from the list", project.name);
                    if components::action_button(ui, &remove, None, None).clicked() {
                        action = Some(EntryAction::RemoveRecent(index));
                    }
                });
            }
        }
    }
    action
}

/// Draws the new-project form and returns what the user chose, if anything.
pub fn new_project(ui: &mut egui::Ui, state: &mut EntryState) -> Option<EntryAction> {
    let mut action = None;
    components::page_title(ui, "New project");

    let error = name_error(&state.new_name);
    let field: Response = match (&error, state.name_touched) {
        (Some(message), true) => {
            components::text_field_with_error(ui, "Project name", &mut state.new_name, message)
        }
        _ => components::text_field(ui, "Project name", &mut state.new_name),
    };
    if state.focus_name {
        field.request_focus();
        state.focus_name = false;
    }
    if field.changed() {
        state.name_touched = true;
    }
    let submitted = field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

    ui.add_space(super::theme::SPACING);
    components::field_label(ui, "Diagram type");
    for (index, kind) in CREATABLE_TYPES.into_iter().enumerate() {
        let _ = components::choice(ui, index == 0, kind);
    }
    components::field_label(
        ui,
        "Other diagram types appear here once they can be created.",
    );

    ui.add_space(super::theme::SPACING);
    components::field_label(ui, "Location");
    ui.label("Chosen when you first save the project.");

    ui.add_space(super::theme::SPACING);
    ui.horizontal(|ui| {
        let reason = error.as_ref().map(|_| "Enter a project name first.");
        if components::action_button(ui, "Create project", reason, Some("Enter")).clicked() {
            action = Some(EntryAction::CreateProject(state.new_name.clone()));
        }
        if components::action_button(ui, "Cancel", None, Some("Escape")).clicked() {
            action = Some(EntryAction::CancelNewProject);
        }
    });

    if submitted {
        state.name_touched = true;
        if error.is_none() {
            action = Some(EntryAction::CreateProject(state.new_name.clone()));
        }
    }
    action
}
