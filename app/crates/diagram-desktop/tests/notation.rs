//! UI-11: notation capability and surface contract.

use diagram_desktop::shell::{self, ShellState};
use diagram_desktop::ui::commands::{self, CommandId, Placement};
use diagram_desktop::ui::notation::{
    self, Delivery, InspectorSection, Interaction, NOTATIONS, NotationId, SEQUENCE_REQUIREMENTS,
    Tool,
};
use eframe::egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

#[test]
fn only_delivered_notations_should_be_creatable() {
    let creatable: Vec<_> = notation::creatable().map(|n| n.id).collect();

    assert_eq!(creatable, [NotationId::Class]);
    for planned in NOTATIONS.iter().filter(|n| n.id != NotationId::Class) {
        assert!(
            matches!(planned.delivery, Delivery::Planned("M4" | "M5")),
            "{:?} must name its milestone",
            planned.id
        );
    }
}

#[test]
fn lookup_should_return_each_notation_s_own_entry() {
    for entry in &NOTATIONS {
        assert_eq!(notation::notation(entry.id).id, entry.id);
    }
}

#[test]
fn delivered_tools_should_be_real_catalog_commands_and_planned_tools_only_names() {
    for entry in &NOTATIONS {
        for tool in entry.tools {
            let consistent = matches!(
                (entry.delivery, tool),
                (Delivery::Delivered, Tool::Command(_)) | (Delivery::Planned(_), Tool::Planned(_))
            );
            assert!(
                consistent,
                "{:?} mixes delivered and planned tools",
                entry.id
            );
        }
    }
}

#[test]
fn class_tools_should_match_the_tool_row_placement() {
    let from_notation: Vec<CommandId> = notation::tool_commands(NotationId::Class).collect();
    let placed: Vec<CommandId> = commands::placed(Placement::ToolRow).map(|s| s.id).collect();

    assert_eq!(from_notation, placed);
}

#[test]
fn class_inspector_sections_should_include_members() {
    let class = notation::notation(NotationId::Class);

    assert_eq!(
        class.element_sections,
        [
            InspectorSection::Model,
            InspectorSection::Members,
            InspectorSection::Appearance,
            InspectorSection::Advanced
        ]
    );
    assert_eq!(class.relationships, ["Association", "Generalization"]);
    assert_eq!(class.interaction, Interaction::Freeform);
}

#[test]
fn sequence_should_keep_its_temporal_requirements_explicit() {
    let sequence = notation::notation(NotationId::Sequence);

    assert_eq!(
        sequence.interaction,
        Interaction::Temporal(&SEQUENCE_REQUIREMENTS)
    );
    assert!(SEQUENCE_REQUIREMENTS.len() >= 5);
    assert!(SEQUENCE_REQUIREMENTS[0].contains("ordered top to bottom"));
    assert!(
        NOTATIONS
            .iter()
            .filter(|n| n.id != NotationId::Sequence)
            .all(|n| n.interaction == Interaction::Freeform),
        "only sequence diagrams use the temporal model"
    );
}

#[test]
fn shell_should_show_the_active_notation_s_tools() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, ShellState::default());
    harness.run();

    for command in notation::tool_commands(NotationId::Class) {
        harness.get_by_label(command.spec().label);
    }
    for planned in ["Lifeline", "Decision", "Actor", "Entity"] {
        assert!(
            harness.query_by_label(planned).is_none(),
            "{planned} offered"
        );
    }
}

#[test]
fn new_project_form_should_offer_only_creatable_notations() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, ShellState::welcome());
    harness.run();
    harness.get_by_label("New project").click();
    harness.run();

    harness.get_by_label("Class diagram");
    for planned in NOTATIONS
        .iter()
        .filter(|n| n.delivery != Delivery::Delivered)
    {
        assert!(
            harness.query_by_label(planned.name).is_none(),
            "{} offered",
            planned.name
        );
    }
}
