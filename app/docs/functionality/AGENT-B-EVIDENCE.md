# Agent B evidence: contract, commands, editor state and core model

Date: 2026-11-05. Branch `agent-b/functionality`. Plan: [AGENT-B-FUNCTIONALITY.md](../../../plan-rust-diagram-editor/agents/AGENT-B-FUNCTIONALITY.md), tasks and tests: [TASKS-AND-TESTS.md](../../../plan-rust-diagram-editor/agents/TASKS-AND-TESTS.md).

## What was built

| Task | What exists now | Where |
|---|---|---|
| B-0 | `Presentation`, `UiIntent`, `InputContext`, `AppState` and `dispatch::apply`. The shell draws only from the presentation and applies intents. | `src/presentation.rs`, `src/intent.rs`, `src/app_state.rs`, `src/dispatch.rs` |
| B-1 | Shared test helpers and catalog-driven routing tests. | `tests/common_state/`, `tests/routing.rs` |
| B-2 | Catalog of 63 commands from spec §8: D1 (Ctrl+Shift+P), D2 (Note removed), D3 (Hand). Each command has a group, placement, shortcut, scope, aliases and a reason when disabled. | `src/ui/commands.rs` |
| B-3 | Shortcut dispatch priority: modal → IME or text field → region → app command. Each key event is consumed once. | `commands::take_shortcut_in` |
| B-4 to B-9 | Tools and gestures, viewport, selection, drafts, Escape layering, F6 region order, preferences, tabs, and separate command and object searches. | `src/editor_state.rs`, `src/viewport.rs`, `src/drafts.rs` |
| B-5 | Checked world ↔ screen transform and picking. | `diagram-canvas` |
| B-10 | Notation contract: Class is the only creatable notation, and the tool row is built from it. | `diagram-core/src/notation.rs` |
| R1-01 | Typed IDs (`ElementId`, `MemberId`, `RelationId`, `DiagramId`, `ViewId`). Model records are separate from views. Geometry is finite and checked, and order is explicit. | `diagram-core/src/ids.rs`, `model.rs`, `geometry.rs` |
| R1-02 | Typed `DomainError`s, relation checks (self-generalization, duplicates, cycles, deferred self-association), multiplicity grammar, and a whole-document check that separates errors from warnings. | `diagram-core/src/validate.rs` |
| R1-03 | `Command` enum with labels, validated into a `ChangeSet` of before/after record patches. Application is all-or-nothing with rollback, the inverse is exact, and delete-everywhere reports its impact counts. | `diagram-core/src/command.rs` |
| R1-04 | `Session` with undo and redo, `SessionId`, `Revision`, `StateId` and saved-state tracking across undo, redo and branching. Stale save results are ignored. History is bounded to 500 transactions. | `diagram-core/src/history.rs` |

## How today's shell reaches the features

The views come from Agent A. `ShellSupport` records what the running shell actually draws and sends. A command whose effect only a new screen could show (grid, snap, Find object, F6 regions, F2) stays disabled with a reason until the shell sets the flag. Nothing is enabled without a visible effect.

The current inspector sends no Enter or blur events. A valid name therefore commits as it is typed, and an invalid one is held until it is fixed or discarded with Escape.

## Verification

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | pass |
| `cargo test --workspace --locked` | 264 passed, 0 failed, both agents' tests included |
| `cargo doc --workspace --no-deps --locked` | pass |
| H3 `tests/shortcuts.rs` | every catalog shortcut, priority, typing, IME and modal cases |
| H4 `tests/routing.rs` | control, shortcut and search routes give the same state; disabled commands change nothing |
| H5 `tests/presentation.rs` | Phase 0 equivalence and state reflection |
| H6 `tests/journeys.rs` | J1 (Hand, Fit, zoom, 100%), Ctrl+Shift+P, Select all, rename with Escape, blank-name hold, all in the real shell |
| Core | `diagram-core/tests/model.rs`, `commands.rs`, `compile_fail` ID doc tests; `diagram-canvas/tests/transform.rs`, `picking.rs` |

## Problems found and limits

- **Real window not checked:** the container lacks `libxkbcommon-x11`, so the app panics at start under Xvfb. No screenshots are claimed, and UI-06 stays `in_progress` for that reason.
- **A view lost the first pan step:** A's canvas view sent `Down` where egui recognized the drag instead of at the press. A fixed it in `A-req-B2` (c182128) with a regression test, and J1 now checks the exact distance.
- **Rows clipped by a scroll area were audited as visible:** removing Note moved one hidden search row onto the window edge. A fixed it by rendering only visible rows (ee9c92f).
- **History budget:** the history is bounded by transaction count only. The 64 MiB byte budget is not measured.
- **Not bound yet:** the core `Session` is not yet connected to the desktop editor (UI-13 to UI-15). The desktop still edits the sample classes, so Undo, Save and Add class stay disabled with reasons.
