# Execution plan: from ideas to code, tests and UI

Date: 2026-11-06. Turns [IDEAS-PLAN.md](IDEAS-PLAN.md) into ordered work. It follows the repository rules: [UI-RULES.md](../../../plan-rust-diagram-editor/UI-RULES.md) and the [maintainability skill](../../../.claude/skills/diagram-studio-maintainability/SKILL.md) (widgets queue a `UiIntent`; only the coordinator changes state; pure logic stays pure).

**All file names below are proposals.** Existing files are named exactly; new ones are marked *(new)*. Phases map to existing roadmap tasks in `plan-rust-diagram-editor/roadmap.json`; no task may be marked `done` until its acceptance text and the UI rules hold. Sizes are S (one session), M (a few), L (many); they are rough, not estimates of time.

## Ground rules for every phase

1. Do the work in `D:\REA visual\app`, on a normal branch. Nothing goes in `.claude` except skills and memory.
2. Order inside a phase: pure logic with unit tests, then the surface with headless tests and an audit scene, then a real-window check with screenshots saved under `app/docs/ui/`.
3. Before each commit, from `app/`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo doc --workspace --no-deps --locked`. From the repository root: the maintainability check, with 0 FAIL.
4. A bug fix gets a regression test that fails without the fix.
5. A change that only moves code goes in its own commit with no behaviour change.

## Phase 0: housekeeping and decisions (S)

| Work | Files | Tests |
|---|---|---|
| Make decisions D1 to D6 from the ideas plan and record them as ADRs | `plan-rust-diagram-editor/decisions/ADR-003-UI-DECISIONS.md` *(new)* | none (documentation) |
| One status source: regenerate summaries from `roadmap.json` | `plan-rust-diagram-editor/build_rust_roadmap.py`, `PROGRESS-AND-NEXT-STEPS.md` | `test_build_rust_roadmap.py`: fail if a summary count differs from `roadmap.json` |
| Shared test helpers (`harness`, `harness_at` are duplicated) | `app/crates/diagram-desktop/tests/common/mod.rs` *(new)* | existing tests keep passing |
| Split `shell.rs` (1,659 lines) before touching it: move the entry screens, tab row and canvas into their own `ui/` modules | `ui/canvas.rs` *(new)*, `ui/tabs.rs` *(new)*, `ui/entry.rs` *(new)*, `shell.rs` | all existing tests green before and after, one commit per move |

## Phase 1: bind the window to the real document (M, roadmap R2-03 and UI-13)

Goal: the project panel, canvas and inspector read `diagram-core`, not `PreviewClass`.

| Work | Files | Tests |
|---|---|---|
| A session wrapper that owns `diagram_core::history::Session` and applies commands | `diagram-desktop/src/session.rs` *(new)* | `tests/session.rs` *(new)*: executing a command changes the document; failure leaves it unchanged; dirty state follows history |
| Read model: pure functions that turn a `Document` into rows for the project panel, canvas and inspector | `diagram-desktop/src/ui/read_model.rs` *(new)* | unit tests next to the module: ordering, names, counts, relation labels |
| Typed targets in intents: replace `SelectClass(usize)` and friends with `ElementId`, `ViewId`, `RelationId`, `DiagramId` | `ui/intents.rs`, `shell.rs`, `ui/navigator.rs`, `ui/inspector.rs` | rewrite `tests/navigator.rs`, `tests/inspector.rs`; add a test that stale IDs after delete or undo are ignored with a message, never a panic |
| Keep fixtures only for the welcome screen "sample project", built through real commands | `ui/entry.rs`, `diagram-core/tests/common/mod.rs` (reuse) | `tests/welcome.rs`: sample project equals the document built by commands |
| Return to the welcome screen without restarting | `ui/commands.rs` (File > Close project), `ui/entry.rs` | `tests/commands.rs`: Close project asks first when dirty; Escape cancels |

Done when: the sample project renders from a `Document`, rename is a command, and the Undo button reflects the history.

## Phase 2: create and edit classes and members (M to L, R2-04 and UI-14)

| Work | Files | Tests |
|---|---|---|
| A tool state: Class tool enabled, click the canvas, a new class appears with its name in edit mode | `ui/commands.rs` (enable `Tool: Class`), `ui/canvas.rs`, `ui/intents.rs` (`CreateClassAt`) | `tests/workspace.rs`: tool enables, click creates one class, Escape cancels naming, one undo removes it |
| Duplicate-name rule from D2 shown as a dialog in words | `ui/dialogs.rs` *(new)*, `ui/components.rs` | `tests/dialogs.rs` *(new)*: dialog text, focus return, keyboard path |
| Member text parser: `name : String`, `+ total : Money`, `placeOrder(id : OrderId) : Order` | `diagram-core/src/member_text.rs` *(new)* | `diagram-core/tests/member_text.rs` *(new)*: valid forms, visibility prefixes, blank, malformed, Arabic names |
| An update-member command (core lacks one) and operation parameters | `diagram-core/src/command.rs` (`UpdateMember`), `diagram-core/src/model.rs` | extend `diagram-core/tests/commands.rs`: undo restores the old member; stale and missing IDs fail cleanly |
| Inspector member list: add, edit, delete, reorder, each a command | `ui/members.rs` *(new)*, `ui/inspector.rs` | `tests/inspector.rs`: add attribute enabled, Enter commits, Escape cancels, error stays beside the field with focus, mixed-selection state |
| Right-click menu on a class and on empty canvas (Add attribute, Add operation, Delete, Duplicate) | `ui/context_menu.rs` *(new)*, `ui/commands.rs` | `tests/context_menu.rs` *(new)*: opens by mouse and keyboard, disabled items give reasons |

## Phase 3: canvas interaction (L, R2-01, R2-02, R2-05 and UI-15)

| Work | Files | Tests |
|---|---|---|
| A shared scene (shapes, anchors, hit areas) that both drawing and export use | `diagram-canvas/src/scene.rs` *(new)*, `diagram-canvas/src/picking.rs` | `diagram-canvas/tests/scene.rs` *(new)*: scene from a fixture document; picking by point and by rectangle |
| A gesture state machine: idle, press, drag, draw-connector, cancel | `diagram-canvas/src/gesture.rs` *(new)* | `diagram-canvas/tests/gesture.rs` *(new)*: pure transitions; Escape cancels; one drag end yields one `MoveViews` |
| Drag to move with connectors following; one undo step per drag | `ui/canvas.rs`, `diagram-core` (`MoveViews`) | `tests/workspace.rs`: drag moves, connector endpoints follow, Undo restores |
| Association and generalization tools, with clear end labels from D1 | `ui/commands.rs`, `ui/notation.rs`, `ui/inspector.rs` | `tests/inspector.rs`: generalization shows "Parent" and "Child"; cycle and duplicate blocked with a written error |
| Undo and Redo enabled with labels from the history | `ui/status.rs`, `ui/commands.rs` | `tests/commands.rs`: menu shows "Undo: Move 2 classes" |

## Phase 4: reuse and delete (M, R2-06 and R2-07)

| Work | Files | Tests |
|---|---|---|
| New diagram, show an existing class in another diagram (explicit "Show in diagram"), and "Duplicate model" as a separate action | `ui/commands.rs`, `ui/navigator.rs`, `ui/dialogs.rs` | `tests/navigator.rs`: shared name changes appear in both diagrams; duplicate creates a new model |
| Remove from diagram versus delete from project, with the impact count from `delete_impact` | `ui/dialogs.rs`, `diagram-core/src/command.rs` (already has `RemoveViews`, `DeleteElement`) | `tests/dialogs.rs`: dialog states how many views and relations are affected; undo restores all |

## Phase 5: save, open and recovery (L, R1-05 to R1-07, UI-16, R3-03)

Start only after Phase 1, because Save needs a real `Document`.

| Work | Files | Tests |
|---|---|---|
| Schema v1 and DTOs, separate from the in-memory model | `diagram-storage/src/schema.rs` *(new)*, `diagram-storage/src/dto.rs` *(new)* | `diagram-storage/tests/schema.rs` *(new)*: version recorded; unknown newer version refused with a message |
| Transactional save: write a temporary file, check it, then replace | `diagram-storage/src/save.rs` *(new)* | `diagram-storage/tests/atomic_save.rs` *(new)*: injected failure leaves the old file intact |
| Load, with corruption and partial-file handling | `diagram-storage/src/load.rs` *(new)* | `diagram-storage/tests/load.rs` *(new)*: round trip equals the original document; truncated and tampered files fail with a message |
| A worker for save and load, so painting never blocks | `diagram-desktop/src/worker.rs` *(new)* | `tests/worker.rs` *(new)*: a late result for an old revision never marks newer edits as saved |
| Save, Save As, Open, recent projects from a real list, unsaved-changes prompt | `ui/commands.rs`, `ui/entry.rs`, `ui/recent.rs` *(new)* | `tests/commands.rs`, `tests/welcome.rs`: Ctrl+S, Cancel keeps data, missing file shows the existing "Error: File not found" pattern |
| Recovery of unsaved work after a crash | `diagram-storage/src/recovery.rs` *(new)* | `diagram-storage/tests/recovery.rs` *(new)*: recovered document equals last committed state |

## Phase 6: navigation and export (M, R3-01)

| Work | Files | Tests |
|---|---|---|
| Pan, zoom, zoom-to-fit, multi-select, simple alignment | `diagram-canvas/src/viewport.rs`, `ui/canvas.rs`, `ui/commands.rs` | `diagram-canvas/tests/transform.rs` (extend), `tests/workspace.rs` |
| SVG export from the shared scene | `diagram-canvas/src/svg.rs` *(new)* | `diagram-canvas/tests/svg.rs` *(new)*: golden files for a fixture project; text is escaped; Arabic names intact |

## Phase 7: quality gates (M, R0-04, UI-10, UI-12, R2-08, R3-04 to R3-07)

| Work | Tests and evidence |
|---|---|
| The class vertical-slice walkthrough: create project, two classes, members, association, generalization, second diagram, save, close, reopen, export SVG | `tests/journey.rs` *(new)*: the whole path by keyboard through the accessibility tree; a Windows run with saved screenshots |
| Text, IME, screen reader and DPI proof | Windows manual checks recorded in `app/docs/R0-04-VERIFICATION.md` *(new)* |
| Release profile and distribution check | `app/docs/R3-06-DISTRIBUTION.md` *(new)*; measured, not assumed, startup and memory |

## New and changed UI

Every change below also needs headless tests, an entry in `tests/ui_audit.rs`, and screenshots at 800×500, 1280×800 and 1440×900 in both themes.

| Screen or control | Change | Rule it must keep |
|---|---|---|
| Tool row | Select, Pan, Class, Association, Note and Arrange become real; each keeps a written reason while disabled | disabled controls say why |
| Canvas | Click to create a class and edit its name in place; drag to move; draw a connector with a live preview; Escape cancels the gesture | Escape closes text, dialog, drawer, selection in that order |
| Class on canvas | Shows attributes and operations; long names truncate with an ellipsis | long text stays on one line |
| Inspector | A member list with Add, Edit, Delete, Reorder; the Model and View sections stay labelled "shared by every diagram" versus "this view only" | field errors beside the field with focus kept |
| Relationship inspector | Names the parent and child end of a generalization in words | errors are words |
| Context menu | Right-click on a class or the canvas; the same commands as the toolbar | shortcuts only from the catalog |
| Dialogs | Duplicate name, remove versus delete, unsaved changes, save failure | focus returns to the opener |
| File menu | Save, Save As, Open, Close project, Export SVG | reasons when disabled |
| Welcome | Real recent projects and recovery; sample project built from real commands | fixtures labelled |
| Status bar | Saved, Unsaved or Failed to save, in words; Undo and Redo labels | not colour alone |

A rough wireframe for the main workspace after Phase 3:

```text
+------------------------------------------------------------------+
| Menu  Shop · unsaved      [Undo: Move class] [Redo] [Search] [Save]|
+---------+--------------------------------------------+-----------+
| Project | [Class diagram 1] [Close]  [New diagram]   | Selection |
| search  | [Select][Pan][Class][Association][Note]    | Name      |
| Diagrams|                                            | Attributes|
| Model   |   +----------+    places   +---------+    |  + Add    |
| Relations   | Customer |------------>| Order   |    | Operations|
|         |   +----------+ 1        0..*+---------+   |  + Add    |
+---------+--------------------------------------------+-----------+
| Unsaved changes                              Theme: Light Dark   |
+------------------------------------------------------------------+
```

## Order of work and what can run in parallel

Phase 0 first. Phase 1 blocks everything else. After Phase 1, Phase 2 (members, class creation) and the schema half of Phase 5 (R1-05, R1-06) can run in parallel because they touch different crates. Phase 3 needs Phase 2's creation path. Phases 4 and 6 follow Phase 3. Phase 7 comes last.

## What this plan does not claim

- It does not estimate calendar time.
- It does not add notation families, extra relation kinds or VP file compatibility.
- File names and splits are proposals; adjust them when the code shows a better place, and record the change in the task evidence.
