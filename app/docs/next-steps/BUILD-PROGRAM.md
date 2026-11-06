# Build program: slices S2 to S13

Date: 2026-11-06. The ordered list of slices that build [EXECUTION-PLAN.md](EXECUTION-PLAN.md), each with the roadmap tasks it serves. How each slice is run is in [WORKFLOW-PLAN.md](WORKFLOW-PLAN.md) section 3. Decisions D1 to D6 are in [ADR-003](../../../plan-rust-diagram-editor/decisions/ADR-003-UI-DECISIONS.md).

**This file holds no status.** [roadmap.json](../../../plan-rust-diagram-editor/roadmap.json) is the only status source (D6). A task changes status there only when its acceptance text and the UI rules truly hold, and the summaries are regenerated with `build_rust_roadmap.py`. A slice *contributes* to a task; it does not close it. A task closes only when its acceptance text holds **and** every task in its `depends_on` list is already `done`, so closing follows roadmap dependency order, not slice order. Several prerequisites (UI-02, UI-07, UI-09, UI-10, UI-12, R0-04, R0-05) are outside what these slices finish; tasks that depend on them stay open until they are closed by their own reviews.

Each slice has its own branch off up-to-date `main` and its own pull request. S1 was `docs/p0-decisions` (ADR-003 and the tracker rule).

| Slice | Branch | What it delivers | Contributes to |
|---|---|---|---|
| S2 | `refactor/p0-split-shell` | Entry screens, tab row and canvas move out of `shell.rs` into `ui/entry.rs`, `ui/tabs.rs` and `ui/canvas.rs`, one commit per move, no behaviour change. `tests/common/mod.rs` with `harness`, `harness_at`, `harness_with` and `harness_unrun`. This file. | Groundwork for UI-13; no task contributed |
| S3 | `feat/p1-session-binding` | All of Phase 1: `session.rs`, `ui/read_model.rs`, typed-ID intents that ignore stale IDs with a message, the sample project built through commands, File > Close project with a way back to Welcome. | R2-03, UI-13 |
| S10 | `feat/p5-storage-core` | In `diagram-storage`, no UI: schema v1, DTOs, save (temporary file, check, replace), load, recovery. Early on purpose, once S3 is merged. | R1-05, R1-06, R3-03 (storage half) |
| S4 | `feat/p2-member-core` | In `diagram-core`: `member_text.rs` parser, an `UpdateMember` command and operation parameters. | R2-04 (core half) |
| S5 | `feat/p2-class-tool` | The Class tool creates a class and starts naming it. The duplicate-name dialog in `ui/dialogs.rs` (D2). | R2-04, UI-14, UI-09 (duplicate-name dialog only) |
| S6 | `feat/p2-inspector-members` | `ui/members.rs` (add, edit, delete, reorder) and `ui/context_menu.rs`. | R2-04, UI-14 |
| S7 | `feat/p3-scene-gesture` | In `diagram-canvas`: `scene.rs`, picking and `gesture.rs`. Pure code with tests. | R2-01, R2-02 |
| S8 | `feat/p3-drag-relations` | Drag to move with connectors following and one undo per drag. Association and generalization tools with Parent and Child labels (D1). Undo and Redo labels. | R2-05, R2-07, UI-15 |
| S9 | `feat/p4-reuse-delete` | New diagram, Show in diagram, Duplicate model, Remove from diagram versus Delete from project with the impact count. | R2-06, UI-09 (remove versus delete dialog) |
| S11 | `feat/p5-save-open-ui` | `worker.rs` (a late result never marks newer edits as saved). Save, Save As, Open, `ui/recent.rs`, the unsaved-changes prompt, recovery in the UI. Headless foundation fixtures that go through public APIs. | R1-07, R1-08, UI-16, UI-09 (save, open and recovery states), UI-13 (real Open), R3-03 |
| S12 | `feat/p6-viewport-svg` | Pan, zoom, zoom-to-fit, multi-select, alignment. `svg.rs` export from the shared scene, with golden files. | R3-01, R3-02 |
| S13 | `feat/p7-journey` | `tests/journey.rs` (the whole path by keyboard). `app/docs/R0-04-VERIFICATION.md` (text, IME, NVDA, DPI). `app/docs/R3-05-PROFILE.md` (frame time, save, load and idle measurements on a representative project). `app/docs/R3-06-DISTRIBUTION.md` (measured startup and memory, runtime requirements; a clean-machine run is recorded only if one is available, otherwise listed as a gap). | R2-08, UI-17, R0-04, R3-05, R3-06 |

## Gates no slice closes

R0-05, UI-10, UI-12, R3-04, UI-18 and R3-07 are review gates. The slices produce their evidence (tests, screenshots, the hand-test report), but each gate closes only through its own review and only if its dependencies are `done`. This program does not claim any of them.

After S13, the finished app is built from `main` and tested by hand; the result goes to `app/docs/ui/acceptance-2026-11-06.md` with screenshots in `app/docs/ui/acceptance-2026-11-06/`. A failed row becomes a bug slice that starts with a regression test.
