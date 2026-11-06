# Slice S2: split `shell.rs` (refactor/p0-split-shell)

Base: `main` at `3e7cabd`. Baseline: 192 tests pass, maintainability 0 FAIL and 11 WARN.

## Scope

A pure structure change. No behaviour, text, size or colour changes.

1. Move `entry_screens` (welcome and new-project screens) to `ui/entry.rs`.
2. Move `TabAction`, `tab_width` and `tab_row` to `ui/tabs.rs`.
3. Move `canvas`, `paint_relation`, `paint_class` and the canvas constants (`CLASS_WIDTH`, `HEADER_HEIGHT`, `ATTRIBUTE_HEIGHT`, `TEXT_INSET`, `DOCUMENT_FILL`, `DOCUMENT_INK`) to `ui/canvas.rs`.
4. Add `tests/common/mod.rs` with `harness` (runs the first frame), `harness_at`, `harness_with` and `harness_unrun` (builds without running a frame, for `tests/shell.rs` and `tests/theme.rs`, which did that before), and delete the copies in the test files.
5. Add `BUILD-PROGRAM.md`, the slice list with roadmap task IDs and no status.

One commit per move. The moved function bodies are not edited.

## What must stay true

- No test changes its inputs, events, assertions or frame counts. The only edits to test files are mechanical: the helper definitions are deleted, imports are added, and calls are renamed one for one (`harness_at(w, h, state)` to `harness_with((w, h), state)` in `typography.rs`; `harness_at(size, state)` to `harness_with(size, state)` in `ui_audit.rs`; `harness()` to `harness_unrun()` in `shell.rs` and `theme.rs`; `welcome.rs` calls `harness_with(size, ShellState::welcome())`). All 192 tests pass before and after.
- Moved code reads state and queues intents, exactly as before. The two moved functions that touched private `ShellState` fields (`text_had_focus`, `pending_focus`) go through two new `pub(crate)` methods, `text_had_focus()` and `focus_next_frame()`. The fields stay private, and no new public field is added.
- Drawing helpers shared with `shell.rs` (`command_button`, `PreviewClass::size`) become `pub(crate)`; nothing becomes `pub` that was not already.
- `DOCUMENT_FILL`, `DOCUMENT_INK` and `TEXT_INSET` become private `const`s of `ui/canvas.rs` (only the canvas uses them). They stay document colours (the checker exempts the `DOCUMENT_` prefix), not editor theme tokens.
- `entry_screens` keeps its existing role: it applies `EntryAction`s itself and changes state directly. Tabs and canvas only queue intents. Turning the entry actions into intents belongs to S3 (session binding), not this slice.
- Maintainability: 0 FAIL, WARN count does not rise (expected 11 down to 9: `shell.rs` shrinks from 1,659 to about 1,250 lines, and the duplicated-helper warnings go).

## Tests and checks

- Behaviour tests: the existing 192, unchanged in intent. A move that needed a test body change would mean a behaviour change; stop and split.
- Gates: fmt, clippy `-D warnings`, test, doc from `app/`; the maintainability check from the root.
- Real window: launch `diagram-desktop`; on Welcome open New project (Escape cancels it), then create a project and open the sample project from a fresh start (the workspace has no way back to Welcome until S3). In the sample workspace click a class, use Ctrl+click, open and close a tab, and use the keyboard. Check 800×500, 1280×800 and 1440×900 in light and dark, and save screenshots under `app/docs/ui/p0-split-shell/`.
- Adversarial case for review: Escape on the New project form and ordinary tab Close must behave exactly as on `main`; existing tests in `tests/welcome.rs` and `tests/navigator.rs` cover both. Stale-target handling is not covered today and is added with typed IDs in S3.
