# Slice S3: bind the window to the real document (feat/p1-session-binding)

Base: `main` at `ffbabcc`. Baseline: 192 tests pass, maintainability 0 FAIL and 9 WARN. Roadmap tasks served: R2-03 and UI-13 (see [BUILD-PROGRAM.md](BUILD-PROGRAM.md)); neither can close yet because both depend on R1-08.

## Scope (Phase 1 of the execution plan)

1. `diagram-desktop/src/session.rs`: `ProjectSession` owns a `diagram_core::history::Session`, executes commands, undoes and redoes, and says whether closing would lose work. The sample project is built by executing core commands (with an empty undo history), not from fixtures.
2. `ui/read_model.rs`: pure functions from a `Document` to rows with typed IDs for the project panel, canvas and inspector.
3. Typed IDs everywhere a record is named: `UiIntent` carries `ElementId`, `RelationId` and `DiagramId`; selection and tabs hold IDs. An ID that no longer exists is ignored with a sentence in the status bar and never selects or changes anything.
4. Rename is a command (`RenameElement`); relation role and multiplicity edits are `SetEnds`. Undo and Redo are enabled from the history and say what they did.
5. File > Close project returns to Welcome. With unsaved changes it asks first in a modal dialog (`ui/dialogs.rs`): the safe button has focus, Escape cancels, focus returns to the opener.
6. Welcome's New project and Sample actions become intents, so the coordinator is the only place that opens a project.
7. Removed: `PreviewClass`, `PreviewRelation`, the prototype `Diagram`, and the relationship name field (the model has role and multiplicity on each end, no relation name). Public `ShellState` fields drop from 19 to 10.

## Decisions made here (not covered by ADR-003)

- A new project nobody has edited has nothing to lose, so closing it does not ask. An edited new project, or a project opened from a file with unsaved changes, asks.
- The core writes the multiplicity `0..*` as `*` (same meaning); the UI shows the core's text. A blank multiplicity is allowed and means none, as in the core command.
- Relationship rows read "Customer – Order (association)" because the model has no relationship name.

## Review round 1 changes (Codex plan review, maintainability and UI-rules reviewers)

- **Requests for an old project.** Record IDs restart in every project, so an old project's ID can name a record in the next one. The coordinator counts projects opened or closed (`epoch`); once it changes mid-batch, the rest of the batch is dropped with a sentence. Open and Create are also refused while a project is open.
- **Drafts and history.** A draft committed in the same frame as Undo or Redo is dropped, so it cannot undo the undo. Undo and Redo clear all drafts; an ordinary edit clears only drafts whose record is gone.
- **Escape.** The dialog takes Escape before a drawer or the showcase.
- **Focus return.** From the menu it goes to the Menu button; from the command search to where focus was before the search opened. Asking to close while the dialog is open does nothing.
- **Same value is not an edit.** Names are stored without surrounding spaces; committing the same name, or a multiplicity that means the same (`0..*` and `*`), changes nothing and leaves no history entry.
- **Other.** Closing a background tab keeps the active tab's selection; the sample's construction now fails explicitly (`Result`) and its test compares it with an independently built document; a self-association gets distinct accessible names; a generalization says roles and multiplicities do not apply; refusals read `Error: ... Nothing was changed.` with the full text on hover.
- **Known limit.** The core restores its ID counter when a creation is undone, so an ID of a removed class can be given to the next class. Nothing creates classes in this slice; S5 must handle it (a stale request after create, undo, create must not hit the new class).
- **Close policy.** A new project that was edited and then fully undone still asks before closing, because the check is "an edit was made" (the revision only grows).

## Tests (new or rewritten)

- `tests/session.rs` (6): command changes document and history; a refused command changes nothing and is not an edit; undo and redo restore the exact document; the close prompt follows edits; the sample equals the document built by commands, is valid and has no undo history.
- `tests/read_model.rs` (9): ordering, names, counts, labels, per-diagram canvas model, member lines, empty model for a missing diagram.
- `tests/commands.rs`: Undo and Redo follow history and report it; Close project closes at once when clean; asks first when edited (focus on Cancel, Escape cancels and returns focus, confirm closes); shortcuts do nothing behind a dialog; catalog and contract tests include Close project.
- `tests/navigator.rs`: stale class, relationship, diagram and tab IDs are ignored with a message and change nothing; a stale rename changes nothing.
- `tests/inspector.rs`: rename is one undoable command that restores the exact document; association fields write `SetEnds`.
- `tests/ui_audit.rs`: the close-project dialog at 1280×800 and 800×500.
- Every old test keeps its behaviour check; index-based setup became ID-based through `tests/common`.

## Gates

fmt, clippy `-D warnings`, test and doc from `app/`; maintainability 0 FAIL and WARN not above 9 (now 7); roadmap `--check` (no status changes). `maintainability_check.py` learns the new coordinator helpers and the dialog state field; both are applied only by the coordinator or drawn like the showcase.

## Real-window check

Launch `diagram-desktop`: Welcome, sample project, select a class, rename it and press Enter, Undo and Redo from the header and Ctrl+Z, Close project from the menu (with and without edits), Escape on the dialog, at 800×500, 1280×800 and 1440×900, light and dark. Screenshots go under `app/docs/ui/p1-session-binding/`.

## Adversarial cases for review

- A rename queued before the sample opened, with the same IDs as the new sample's Customer: dropped, history empty (`requests_queued_for_an_old_project_should_not_touch_the_next_one`).
- A draft typed after a committed rename, then Undo clicked: the draft must not replace the restored name and Redo stays available.
- Undo of a creation: the selection is pruned and a later request for that class is refused.
- Close project from the menu and from the command search (focus return), and with a drawer open (Escape order).

Each fix above has a test that was shown to fail with the fix disabled (cross-project requests, draft versus Undo, Escape order, background tab close, focus return from the search).
