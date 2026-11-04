# Rust diagram editor — implementation plan

Plan date: 2026-11-04. Status updated 2026-11-05. Working label: Diagram Studio. R0-01/R0-02/R0-03 are verified on Windows; R0-04 is in progress and R0-05 remains pending. See [workspace instructions](../app/README.md) and [verification evidence](../app/docs/R0-01-VERIFICATION.md).

Build a local Windows desktop modeling tool in Rust, designed to make the basic workflow clear and dependable before expanding diagram coverage.

**Product-wide comparison:** read [Visual Paradigm versus our version](comparison/README.md) or the [searchable comparison](comparison/COMPARISON.html) for feature differences, all 101 notation candidates, deliberate omissions, and remaining validation gaps. These compare a shipped suite with an unbuilt Rust plan.

**Progress and next steps:** [what is done and what comes next](PROGRESS-AND-NEXT-STEPS.md).

**Rust execution roadmap:** open the [searchable milestone and task dashboard](ROADMAP.html), [detailed Rust roadmap](RUST-ROADMAP.md), and [Rust engineering rules](RUST-ENGINEERING.md). The [CSV backlog](rust-backlog.csv) tracks 57 tasks with prerequisites and acceptance conditions; 46 lead through v0.1. Three tasks are complete; R0-04 is in progress and 53 tasks are not started, including 18 UI tasks. The complete UI plan is [07-UI-PLAN.md](07-UI-PLAN.md). Edit [roadmap.json](roadmap.json) and run `python build_rust_roadmap.py` to regenerate the dashboard and CSV.

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
7. [Decision records](decisions/ADR-001-GUI.md), [storage decision](decisions/ADR-002-STORAGE.md).

Open [the interactive layout concept](GUI-CONCEPT.html) to review the proposed workspace. It is a local HTML planning artifact, not the implementation technology or a working Rust application. Its selection, tab and inspector interactions illustrate the proposed design only.

## Next development action

Record the Windows results for **R0-02** (native shell) and **R0-03** (bundled SQLite), then run R0-04. Continue M0 to prove text editing, canvas picking, DPI scaling, keyboard focus, SQLite and graphics startup on this computer. Then implement the class-diagram vertical slice. Do not start by reproducing every Visual Paradigm menu or all 101 installed API constants.

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
