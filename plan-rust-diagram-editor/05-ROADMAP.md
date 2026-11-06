# 05 — Implementation roadmap

Ordered deliverables with observable exit conditions. These are milestones, not calendar estimates. Size the later work after M0/M1 reveal measured UI and storage costs.

For the detailed Rust execution sequence, use [RUST-ROADMAP.md](RUST-ROADMAP.md), the [interactive dashboard](ROADMAP.html) and [Rust engineering rules](RUST-ENGINEERING.md). The `R0-01` through `R6-03` IDs in [roadmap.json](roadmap.json) are the detailed task tracker; the `BAS` IDs below remain high-level acceptance references.

<!-- roadmap:summary:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->
**Status** (generated from [roadmap.json](roadmap.json), last updated 2026-11-05): 57 tasks in 8 milestones, 46 of them through v0.1. 14 done, 2 in progress (R0-04, UI-02), 0 blocked, 41 not started. Complete milestones: none. Current gate: M0, 3 of 5 tasks done.
<!-- roadmap:summary:end -->

See [Windows verification](../app/docs/WINDOWS-REVIEW-2026-11-05.md).

## M0 — Prove the desktop foundation

Tasks:

- Create the future `app/` Cargo workspace, pin the tested Rust version and resolve eframe.
- Open one native window with a top bar, collapsible sidebar, canvas and inspector.
- Draw/select a class-shaped node; edit a name using a real egui text widget.
- Exercise keyboard focus, pointer capture loss, DPI changes, English/Arabic/mixed text and IME/paste.
- Verify accessible naming/focus for standard controls and a keyboard object-list alternative to canvas selection.
- Build/link bundled rusqlite, write/read a temporary fixture, and report adapter/startup errors clearly.

Exit gate: native debug/release build launches repeatedly on this computer; tested text/DPI/keyboard behavior is acceptable; no renderer startup failures are concealed. Record limitations. If the framework cannot meet essential text/accessibility needs with bounded work, revisit ADR-001 before writing the editor around it.

## UX — UI design and native presentation

UX runs in parallel with M1, with design exploration starting alongside the remaining M0 checks; UI-12 gates M2 through R2-08. Use [the complete specification](UI-DESIGN-AND-INTEGRATION.md) and [interactive design](UI-DESIGN.html). UI-13–UI-17 connect the presentation to real behavior in M2; UI-18 is part of M3 release review.

## M1 — Trustworthy model and persistence

- Typed IDs, class/member/relation model, diagrams and separate views.
- Validation and inverse patches; compound commands, history budget, revision tracking.
- SQLite schema v1; foreign keys, load limits, transactional save, Save As publication and clear errors.
- Separate recovery snapshots and migration policy.

Exit gate: headless reference fixtures survive save/load; rename/duplicate/reuse/delete/undo invariants hold; malformed files and failed writes leave the previous project intact. Basic migrations and unsupported-newer-schema behavior are tested.

## M2 — First usable class diagram

- Palette placement and inline naming; attributes/operations in inspector.
- Node measurement, compartments, selection, resizing, pan/zoom/fit and snapping.
- Association/generalization, endpoint roles/multiplicity and route preview.
- Keyboard movement, cancellation and compound undo.
- Project tree, diagram tabs and model reuse in a second diagram.

Exit gate: the complete Customer/Order/PreferredCustomer scenario in 01-PRODUCT-SCOPE.md works through the real UI and the engine tests.

## M3 — Reliability and ease of use

- Multi-selection, alignment, grouping, styles, identity-aware duplicate/reuse.
- Save/recovery status, close prompts and bounded background worker behavior.
- Keyboard navigation and accessible object-list flow.
- SVG export from the shared scene; clipping/text/arrowhead checks.
- Representative-user evaluation and measured performance on declared fixtures.

Exit gate: all v0.1 acceptance checks pass, no known data-loss path remains, and usability results/remaining limitations are recorded. This is the first basic release.

## M4 — Validate the shared notation framework

Add Flowchart and Use Case adapters. Each brings typed shapes/relations, creation tools, inspector fields and export fixtures. Reuse the canvas and command pipeline rather than branching the entire application.

Exit gate: a new notation is added without duplicating project management, save/load, selection or history. Test invalid-connection feedback and mixed model/view identity behavior.

## M5 — Specialized modeling

ER: typed columns/keys/cardinality, entity compartments and relationship properties. Activity/state: containment, lanes/regions and transitions. Requirements/SysML follow their own constrained schemas. Sequence is a dedicated milestone covering lifelines, ordered messages, activation spans, fragment membership and specialized movement/numbering rules.

Exit gate per family: one complete meaningful workflow, persistence/undo fixtures, rendering checks and clearly scoped standards support. A family is not complete because its palette appears.

## M6 — Broader capabilities after scope agreement

Consider remaining families from the 17-family research catalog, project XML interoperability, images/PDF, richer auto-layout, reports, code/database tooling, optional services and collaboration. Each needs a separate proposal and fixtures. Native VP file compatibility is not implied by choosing SQLite for our file format.

## First implementation checklist

| ID | Deliverable | Depends on | Proof |
|---|---|---|---|
| BAS-001 | Cargo workspace and reproducible native build | None | Debug/release startup log |
| BAS-002 | Text/DPI/focus/accessibility prototype | BAS-001 | Recorded native matrix and ADR gate result |
| BAS-003 | Typed document model and commands | BAS-001 | Invariant and inverse-patch tests |
| BAS-004 | SQLite v1, safe save/load | BAS-003 | Roundtrip plus failed-write evidence |
| BAS-005 | Scene, geometry and pointer-state machine | BAS-002/003 | Transform/hit-test/cancel fixtures |
| BAS-006 | Class/member editor | BAS-005 | Two edited classes |
| BAS-007 | Relations, anchors and route updates | BAS-006 | Association/generalization workflow |
| BAS-008 | Multi-view navigation and inspector | BAS-006/007 | Shared rename, independent geometry |
| BAS-009 | Recovery, errors and close behavior | BAS-004/008 | Interrupted-save/recovery matrix |
| BAS-010 | SVG export | BAS-005/008 | Render/structure fixture |
| BAS-011 | Usability and performance pass | BAS-009/010 | Measured acceptance report |

Resolve usability or persistence failures before adding the next notation. Architectural expansion follows evidence from the first working vertical slice.
