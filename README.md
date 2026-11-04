# Diagrams

A planned offline desktop diagram editor written in Rust. The working product name is Diagram Studio.

The first release focuses on a dependable class-modeling workflow: classes and members, associations and inheritance, reusable model views, undo/redo, project persistence, recovery and SVG export. Broader diagram families follow after this foundation is verified.

**Status:** planning complete enough to start the prototype; application implementation has not started. The proposed GUI is egui through eframe, subject to the native text, accessibility, scaling and graphics checks in M0.

## Start here

- [Rust implementation roadmap](plan-rust-diagram-editor/RUST-ROADMAP.md)
- [Rust engineering rules](plan-rust-diagram-editor/RUST-ENGINEERING.md)
- [Product and architecture plan](plan-rust-diagram-editor/README.md)
- [39-task backlog](plan-rust-diagram-editor/rust-backlog.csv)
- [Visual Paradigm comparison](plan-rust-diagram-editor/comparison/README.md)
- [Supporting product research](research/visual-paradigm/README.md)

The roadmap has seven milestones. M0 proves the desktop foundation; M1 builds the model and storage; M2 completes class editing; M3 prepares v0.1. M4–M6 cover later diagram families and separately scoped capabilities. Task **R0-01** creates the future four-crate Cargo workspace under `app/`.

## Browse the planning dashboards

Clone this repository and open these files in a browser:

- `plan-rust-diagram-editor/ROADMAP.html`
- `plan-rust-diagram-editor/GUI-CONCEPT.html`
- `plan-rust-diagram-editor/comparison/COMPARISON.html`

They are local planning artifacts, not the Rust application. GitHub displays HTML source rather than running these dashboards.

To update the roadmap, edit `plan-rust-diagram-editor/roadmap.json`, then run:

```sh
python plan-rust-diagram-editor/build_rust_roadmap.py
```

The generator validates task dependencies and refreshes the HTML dashboard and CSV backlog. Completion requires implementation and verification evidence.

## Repository layout

```text
plan-rust-diagram-editor/   Product scope, architecture, roadmap and UI concepts
research/visual-paradigm/   Behavior observations, comparison evidence and fixtures
app/                       Future Rust workspace; not created yet
```

The editor will use independently written code and original assets. Visual Paradigm research describes observed behavior and does not imply affiliation or project-file compatibility. Machine-specific agent settings, temporary files and backup copies are excluded from version control.
