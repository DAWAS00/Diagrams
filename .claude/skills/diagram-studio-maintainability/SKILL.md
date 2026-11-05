---
name: diagram-studio-maintainability
description: How to keep the Diagram Studio Rust/egui code maintainable and well structured. It covers where new code belongs, the frame-by-frame program flow (widgets queue a UiIntent, one coordinator applies it), the testing practice (pure unit tests, headless egui_kittest tests through the accessibility tree, the usability audit, regression tests proven to fail), a review checklist, and a script that measures the structural rules. Use this skill whenever you add, change, refactor, review or test code under app/ in this repository, whether a UI surface, command, inspector field, notation, theme token, core rule or storage code. Use it too when someone asks about code structure, maintainability, "where should this go", program flow, technical debt, splitting a big file, test quality or reviewing a pull request, even if they never say "maintainable".
---

# Diagram Studio maintainability

The app grows one UI task at a time, and different people and agents work on it. These rules keep each change small, predictable and easy to test. Each rule says **why** it exists, so you can apply its intent to cases it does not name. The UI-specific usability rules are in `plan-rust-diagram-editor/UI-RULES.md`; this skill covers structure, flow and testing, and complements them.

## 1. How a frame flows

Understand this before changing anything. Every frame, eframe calls `shell::show(ui, state)`:

```text
shell::show
 1. Install theme styles if they changed, and apply them to *this* frame's Ui too
 2. Return pending focus (only once no modal layer remains)
 3. Entry screens?  -> entry_screens(): welcome / new project -> EntryAction -> applied there; return
 4. Decide the layout (Docked or Narrow) from the window width
 5. Escape priority, using the previous frame's text focus:
      text field -> showcase -> drawer -> catalog shortcuts      => queue UiIntent
 6. Draw the surfaces: header, status, project panel, inspector, tab row,
    tool row, canvas, drawer, showcase, command search
      * they READ state
      * text fields edit DRAFTS (state.drafts)
      * every other change is QUEUED as a UiIntent
 7. apply_intents(): the ONLY place editor state changes
      Run{command} -> run() checks availability first
      Select / Reveal / OpenDiagram / Tabs / Rename / SetRelationField ...
      (each validated again before touching a record)
 8. Remember whether a text field had focus (for step 5 next frame)
```

Why it matters: drawing and changing state never interleave, so a frame always draws one consistent state. Every user action has exactly one code path you can test, and you can test it without a window, either by queueing an intent or by clicking through the accessibility tree. If you find yourself writing `state.something = …` inside a drawing function, stop: add an intent instead.

## 2. Where new code goes

| You are adding… | Put it in | Test it with |
|---|---|---|
| A rule, calculation or data table (validation, matching, layout maths, which tabs fit) | A **pure function** in `ui/<area>.rs` that does not take `egui::Ui` | A direct unit test |
| Drawing a surface or part of one | A function `fn x(ui, state)` in `ui/<surface>.rs` that reads state and queues intents. **Not `shell.rs`**, which is over budget; see section 6 | Headless kittest tests and a usability-audit scene |
| A way to change editor state | A `UiIntent` variant, a coordinator arm in `apply_intents`, and a validated method on `ShellState` | A test that queues the intent and one that drives it through the UI |
| A user command (button, menu, shortcut) | A catalog entry in `ui/commands.rs` with availability, reason, placement and an `ActionContract` naming its test | The catalog and contract tests in `tests/commands.rs` |
| Colours, sizes, spacing | `ui/theme.rs` tokens and `TextRole`, plus a contrast pair | `tests/theme.rs` |
| What a diagram type offers | The `NOTATIONS` table in `ui/notation.rs` | `tests/notation.rs` |
| A domain rule (names, IDs, model invariants) | `diagram-core`, which has **no egui or SQL** | A core unit test |
| Persistence | `diagram-storage`, run by a worker later, never during painting | A storage integration test with a temporary file |

Dependency direction: `desktop → core / canvas / storage`; `canvas → core`; `storage → core`. Core never depends on the GUI. This keeps the model testable without a window and lets M1 replace the prototype fixtures without touching the UI.

Read `references/architecture.md` for the module map, the responsibilities of each file, and a safe playbook for splitting `shell.rs`.

## 3. Structure rules (and why)

1. **Widgets never change records.** Queue an intent; text fields edit drafts that commit through an intent. *Why:* one place to validate, one place to test, no half-applied edits.
2. **Pure logic stays pure.** Anything that does not need a `Ui` is a plain function over plain data. *Why:* most bugs are in rules, and pure rules are cheap to test exhaustively.
3. **One surface per module, files under about 500 lines; split before 1,000.** *Why:* a reviewer should hold a whole module in their head. When you touch a surface that still lives in `shell.rs`, move it out first, in its own commit, with all tests green before and after.
4. **Add read methods, not public fields.** `ShellState` already has too many public fields. New state should be private, with a method that answers the question callers have, for example `selected_classes()` or `active_notation()`. *Why:* public fields let tests and widgets bypass the coordinator, and every field becomes a compatibility promise.
5. **One source of truth per concept.** Commands come from the catalog, sizes and colours from the theme, diagram capabilities from the notation table. *Why:* duplicated lists drift apart; you saw this with the hard-coded diagram-type list.
6. **Prototype data is labeled and confined.** `PreviewClass`, `Diagram` and `PreviewRelation` are fixtures until M1. Keep them in the desktop crate, never in core, and label any sample data shown to users. *Why:* M1 swaps them for the real model in one place.
7. **Make each change in one commit that shows a clear intent:** structure-only moves separate from behaviour changes. *Why:* reviews and `git bisect` stay meaningful.

## 4. Testing practice

Use the cheapest test that can catch the bug, and go up the layers only when needed:

```text
  real window (screenshots, Linux here; Windows for scaling, IME, screen readers)
  usability audit      tests/ui_audit.rs: every screen state, every control
  headless UI tests    egui_kittest through the accessibility tree
  pure unit tests      rules, tables, maths: fast and exhaustive
```

- **Name tests for behaviour:** `subject_should_result_when_condition`, for example `escape_should_cancel_the_draft_and_restore_the_name`.
- **Find controls by name and role, never by coordinates.** Derive geometry from `workspace::HEADER_HEIGHT`-style constants or from another node's bounds. *Why:* layout changes should not break behaviour tests.
- **Set up state through public behaviour** where possible (`state.queue(UiIntent::…)`, clicks, keys). Set fields directly only to build a fixture that has no UI path yet.
- **Every bug fix gets a regression test, and you prove it:** remove the fix and watch the test fail, then restore it and watch it pass. A test that has never failed has not been shown to protect anything.
- **Every new screen state goes into the usability audit.** The audit's self-test proves it can fail; keep it that way.
- **Shared test helpers belong in `tests/common/mod.rs`,** not copied into each file. This is a current debt: `harness`/`harness_at` exist in several files.

`references/testing.md` has examples from this repo and the egui_kittest pitfalls already hit: `type_text` needs focus, `get_all_by_label` panics when nothing matches, submenus open on hover, missing glyphs panic in the harness, and Ctrl+click needs `click_modifiers`.

## 5. Workflow for any change

1. Read the task's acceptance text in `plan-rust-diagram-editor/roadmap.json`, plus the plan sections it cites.
2. Decide where each piece goes (section 2). If a surface you touch is in `shell.rs`, move it out first.
3. Write the pure parts with unit tests, then the surface with headless tests and an audit scene.
4. Run the structure check and fix every FAIL. Do not let a WARN grow:
   ```sh
   python .claude/skills/diagram-studio-maintainability/scripts/maintainability_check.py
   ```
5. Run the gates from `app/`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo doc --workspace --no-deps --locked`.
6. If a task status changed, run `python build_rust_roadmap.py` in `plan-rust-diagram-editor/` **before** committing. It rejects `done` tasks with unfinished prerequisites.
7. Check the real window at 1280×800 and 800×500, and update the evidence page and `app/docs/ui/README.md`.

## 6. Review checklist

Copy this into a review and answer each point:

- [ ] Each new piece of code is where section 2 says; no new drawing code in `shell.rs`
- [ ] No widget changes state directly; new state changes are intents with validation in the coordinator
- [ ] No new public fields on `ShellState`; new questions are answered by methods
- [ ] Pure logic is separate from drawing and has direct tests
- [ ] Commands, sizes, colours and notation capabilities come from their single source
- [ ] Tests find controls by name or role; no new coordinate literals
- [ ] Each bug fix has a regression test that was shown to fail without the fix
- [ ] New screen states are in the usability audit
- [ ] `maintainability_check.py` shows 0 FAIL, and no WARN count went up
- [ ] fmt, clippy, test and doc pass; the tracker validator passes if a status changed
- [ ] The commit message says what and why; behaviour changes and pure moves are separate commits

## 7. Current debts (measured 2026-11-05)

The check reports 0 FAIL and 10 WARN. Pay these down in this order, each as its own commit with tests green:

1. **Split `shell.rs` (1,659 lines; the status bar already moved to `ui/status.rs`) by surface:** header, status, panels, tab row, tool row, canvas, drawer, search, showcase window and entry screens each get their own `ui/` module. Keep `shell.rs` as state plus the coordinator. Follow the playbook in `references/architecture.md`.
2. **Create `tests/common/mod.rs`** with `harness`, `harness_at`, `bounds` and friends, and delete the copies.
3. **Encapsulate `ShellState`** (19 public fields): add methods, make fields private one at a time, and route test setup through intents or fixture constructors.
4. **Replace the coordinate literals** in `tests/navigator.rs` and `tests/workspace.rs` with values derived from constants or node bounds.
5. **Add a direct test for `navigator::diagrams_showing`.**
6. **Watch `ui/commands.rs` (585 lines):** if it grows, move `contract()` into `ui/contracts.rs`.
