# Rust diagram editor — implementation plan

Plan date: 2026-11-04. Working label: Diagram Studio. See [workspace instructions](../app/README.md) and [verification evidence](../app/docs/R0-01-VERIFICATION.md).

<!-- roadmap:summary:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->
**Status** (generated from [roadmap.json](roadmap.json), last updated 2026-11-05): 57 tasks in 8 milestones, 46 of them through v0.1. 14 done, 2 in progress (R0-04, UI-02), 0 blocked, 41 not started. Complete milestones: none. Current gate: M0, 3 of 5 tasks done.
<!-- roadmap:summary:end -->

Build a local Windows desktop modeling tool in Rust, designed to make the basic workflow clear and dependable before expanding diagram coverage.

**Research before UI implementation:** [UI research and recommendations](../app/docs/ui/new-design/UI-RESEARCH.md) audits the current prototype against documented editor patterns and accessibility guidance. The [function discoverability map](../app/docs/ui/new-design/UI-FUNCTION-MAP.csv) covers 28 v0.1 action groups and the report defines the user-validation protocol. Design proposals remain hypotheses until tested.

**Product-wide comparison:** read [Visual Paradigm versus our version](comparison/README.md) or the [searchable comparison](comparison/COMPARISON.html) for feature differences, all 101 notation candidates, deliberate omissions, and remaining validation gaps. These compare a shipped suite with an unbuilt Rust plan.

**Progress and next steps:** [what is done and what comes next](PROGRESS-AND-NEXT-STEPS.md).

**Rust execution roadmap:** open the [searchable milestone and task dashboard](ROADMAP.html), [detailed Rust roadmap](RUST-ROADMAP.md), and [Rust engineering rules](RUST-ENGINEERING.md). The [CSV backlog](rust-backlog.csv) lists every task with prerequisites and acceptance conditions. [roadmap.json](roadmap.json) is the only status source ([ADR-003](decisions/ADR-003-UI-DECISIONS.md), D6): edit it and run `python build_rust_roadmap.py` to regenerate the dashboard, the CSV and the status summaries in these documents; `python build_rust_roadmap.py --check` fails when a summary is stale.

**UI-first revision:** [complete UI specification](UI-DESIGN-AND-INTEGRATION.md), [interactive final design preview](UI-DESIGN.html), and 18 new UI tasks. UX design/native presentation runs in parallel with M1, and both gate M2; real feature binding remains in M2/M3. Existing completed tasks retain their evidence. UI progress: [app/docs/ui/README.md](../app/docs/ui/README.md). **Every UI change follows [UI-RULES.md](UI-RULES.md).**

**New design reference:** [functionality, Rust implementation and keyboard shortcuts](../app/docs/ui/new-design/NEW-DESIGN-FUNCTIONAL-SPEC.md) specifies every planned v0.1 surface, action, edit/cancel rule, model-versus-view behavior and failure state, plus the proposed Windows shortcut map and acceptance workflows informed by the FigJam review.

## Recommended decisions

| Area | Proposed choice |
|---|---|
| Language / target | Rust, Windows x64 MSVC first |
| GUI | egui through eframe; proceed only after the M0 usability/renderer gate |
| Organization | One desktop process, four small workspace crates |
| Model | Semantic objects and independent diagram views with stable typed IDs |
| Changes | Validated commands, atomic transactions and reversible patches |
| Project file | Versioned SQLite through rusqlite; independent format, proposed `.dgram` extension |
| First complete workflow | Class diagrams: classes, members, association, generalization, two views, save/reopen and SVG export |
| UI | Compact toolbar, project sidebar, canvas, contextual inspector; common tasks inline |
| Expansion | Flowcharts/use cases, ER, activity/state, sequence, then other families |

The GUI choice is a project recommendation, not a claim that one framework guarantees better usability. Validate the actual editor on this computer before committing to the framework. See the decision records.

## Read in this order

1. [Product scope](01-PRODUCT-SCOPE.md): first release and how we judge it.
2. [Architecture](02-ARCHITECTURE.md): crate boundaries, data, commands and persistence.
3. [GUI and workflows](03-GUI-AND-WORKFLOWS.md): simplified interaction design.
4. [Tooling and dependencies](04-TOOLING.md): installed prerequisites and proposed libraries.
5. [Rust execution roadmap](RUST-ROADMAP.md) and [engineering rules](RUST-ENGINEERING.md): detailed tasks, crate boundaries and development gates. [Milestone overview](05-ROADMAP.md) remains the shorter summary.
6. [Quality and validation](06-QUALITY.md): meaningful tests, performance targets and recovery.
7. [Decision records](decisions/ADR-001-GUI.md), [storage decision](decisions/ADR-002-STORAGE.md), [UI decisions D1 to D6](decisions/ADR-003-UI-DECISIONS.md).

Open [the interactive layout concept](GUI-CONCEPT.html) to review the proposed workspace. It is a local HTML planning artifact, not the implementation technology or a working Rust application. Its selection, tab and inspector interactions illustrate the proposed design only.

## Next development action

Current task status is in the generated summary at the top of this page. The order of work is in the [execution plan](../app/docs/next-steps/EXECUTION-PLAN.md): M1 needs only the R0-01 workspace, so it runs in parallel with the UX milestone and the remaining M0 checks; M2 needs M1, the UI-12 review and the R0-05 foundation gate, and then M2/M3 bind the UI to real domain and storage behavior. UI decisions D1 to D6 are recorded in [ADR-003](decisions/ADR-003-UI-DECISIONS.md).

## Assumptions and scope decisions

- Offline, single-user, Windows-first. Core code should remain portable; other platforms are later validation targets.
- Rust is required by the user. egui, SQLite, first-release notation and the layout are proposals chosen to make that requirement concrete.
- Work is independently implemented with original code/assets. The VP installation provides behavior evidence, not implementation code to reuse.
- Direct `.vpp` or VP XML compatibility is later work; our first version uses its own schema.
- No cloud/account/AI/code-generation requirement in the basic release.
- These files are a plan. No packages were installed and no build configuration was changed in this planning pass.

## Research basis

The previous study exercised Class, Sequence and ER editing, saved a fresh CE project, and exported selected diagrams to XML. Semantic identities, geometry, routing, activation and column/property distinctions inform this plan. Advanced behavior remains partly unverified; planning choices are labeled proposals rather than attributed to VP internals.

- [UI study](../research/visual-paradigm/UI-STUDY.md)
- [Diagram family inventory](../research/visual-paradigm/DIAGRAM-CATALOG.md)
- [Observed UI gallery](../research/visual-paradigm/UI-GALLERY.html)
- [Native research fixture](../research/visual-paradigm/ce-evidence/ui-research.vpp)

## Completion standard for the basic app

A user can create a project, place and name classes, add members, connect them, reuse one model in another diagram, move/style views independently, undo/redo, save, close, reopen and export SVG without losing meaning or geometry. Clear errors, recovery, keyboard access and usable scaling are part of that outcome.
