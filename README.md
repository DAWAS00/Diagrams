# Diagrams

A planned offline desktop diagram editor written in Rust. The working product name is Diagram Studio.

The first release focuses on a dependable class-modeling workflow: classes and members, associations and inheritance, reusable model views, undo/redo, project persistence, recovery and SVG export. Broader diagram families follow after this foundation is verified.

<!-- roadmap:summary:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->
**Status** (generated from [roadmap.json](plan-rust-diagram-editor/roadmap.json), last updated 2026-11-05): 57 tasks in 8 milestones, 46 of them through v0.1. 14 done, 2 in progress (R0-04, UI-02), 0 blocked, 41 not started. Complete milestones: none. Current gate: M0, 3 of 5 tasks done.
<!-- roadmap:summary:end -->

The native shell and bundled SQLite smoke test work; real diagram editing and project persistence are not implemented yet.

## Start here

- [Rust implementation roadmap](plan-rust-diagram-editor/RUST-ROADMAP.md)
- [Rust engineering rules](plan-rust-diagram-editor/RUST-ENGINEERING.md)
- [Product and architecture plan](plan-rust-diagram-editor/README.md)
- [Task backlog (CSV)](plan-rust-diagram-editor/rust-backlog.csv)
- [Visual Paradigm comparison](plan-rust-diagram-editor/comparison/README.md)
- [Supporting product research](research/visual-paradigm/README.md)
- [Build and run the workspace](app/README.md)
- [R0-01 verification results](app/docs/R0-01-VERIFICATION.md)
- [Project Rust skills](SKILLS.md)
- [Current Windows verification and remaining work](app/docs/WINDOWS-REVIEW-2026-11-05.md)

The roadmap has eight milestones. M0 proves the desktop foundation; UX delivers the designed native UI; M1 builds the model and storage; M2 completes class editing; M3 prepares v0.1. M4–M6 cover later diagram families and separately scoped capabilities. Task **R0-01** created the Cargo workspace under `app/`; **R0-02** and **R0-03** add the native shell prototype and bundled SQLite.

## UI-first design revision

Read the [new design documentation](app/docs/ui/new-design/README.md) for the [full functionality and shortcut specification](app/docs/ui/new-design/NEW-DESIGN-FUNCTIONAL-SPEC.md), FigJam findings, UI research and action map. Current implementation evidence is on the [UI progress page](app/docs/ui/README.md).

The [complete UI specification](plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md) defines the final v0.1 workspace, all screen states, technology and command contracts. Open the [interactive UI design](plan-rust-diagram-editor/UI-DESIGN.html) locally to compare themes and scenarios. Eighteen UI tasks: native presentation runs in parallel with M1, with real functional bindings in M2/M3. Progress on the UI work, with screenshots and what is next: [app/docs/ui/README.md](app/docs/ui/README.md). Every UI change follows the [UI rules](plan-rust-diagram-editor/UI-RULES.md), which include an automated usability audit.

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
app/                       Buildable four-crate Rust workspace and check script
```

The editor will use independently written code and original assets. Visual Paradigm research describes observed behavior and does not imply affiliation or project-file compatibility. Machine-specific agent settings, temporary files and backup copies are excluded from version control.
