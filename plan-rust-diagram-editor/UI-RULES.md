# UI rules — definition of done for every UI change

These rules apply to **every change to the application UI**, by a person or an agent. A UI task is not done, and its tracker status must not say `done`, until each rule below holds and its evidence is recorded. They come from the [UI design](UI-DESIGN-AND-INTEGRATION.md), the [engineering rules](RUST-ENGINEERING.md), and defects actually found while building UI-01 to UI-06 (each rule names the defect that motivated it).

## 1. Usability rules

1. **Every control has a visible text label and an accessible name.** No icon-only controls. Field labels are attached with `labelled_by` (use `components::text_field`).
2. **Hit targets are at least 32 points high**; buttons and fields are 34. Nothing overlaps another control. Everything fits inside the window at **800×500, 1280×800 and 1440×900**. *(Found: list rows wrapping and canvas names spilling at 800×500.)*
3. **A disabled control always says why**, in its hover text and, in lists and dialogs, as visible text. Fixture or sample data is labeled "design fixture", and nothing pretends to open, save or recover real files. *(Found: an unlabeled sample recovery notice.)*
4. **Errors and warnings are words, not colour alone:** "Error: …" and "Warning: …". A field error sits beside the field, and the field **keeps keyboard focus** when its error appears. *(Found: the name field lost focus when the error appeared.)*
5. **Everything works from the keyboard.** Tab reaches every enabled control; Enter activates. Escape closes the most recent thing first: **text field → dialog or window → drawer → selection**. Focus returns to the control that opened a dialog, drawer or window. *(Found: focus not restored after the search dialog; Escape swallowed by a focused button.)*
6. **Custom-drawn interactive areas publish accessibility info** (`widget_info` with a role and a name), or no screen reader or test can see them. Shapes on the canvas must also be reachable through a list (the project panel's Model list).
7. **Shortcuts come only from the command catalog** (`ui/commands.rs`), and keys a focused **text field** uses stay with the field. Decide with `text_edit_focused()` from the previous frame, never `egui_wants_keyboard_input()`.
8. **Long text stays on one line with an ellipsis** in lists, the status bar and canvas shapes; the full value stays in the accessibility tree and is editable in the inspector.
9. **Every list or surface has designed empty, no-match and error states**, not just the "full" state.
10. **Only design tokens and text roles:** no raw colours or font sizes outside `ui/theme.rs`. New colour pairs are added to the contrast tests: 4.5:1 for text, 3:1 for control boundaries and focus. Unpressed toggles keep a visible border. *(Found: the planned border colour at 1.2–2.3:1; borderless toggles.)*
11. **Applying styles must reach the current frame.** Anything installed in the egui context mid-frame must also be applied to the current `Ui`. *(Found: a start-up crash from a missing text style.)*

## 2. Testing rules

1. **Behaviour tests:** each new control or screen gets headless `egui_kittest` tests through the accessibility tree, covering presence by name, the main interaction, the keyboard path, and each state (enabled, disabled with reason, error, empty).
2. **Usability audit:** add every new screen state to `app/crates/diagram-desktop/tests/ui_audit.rs`. It checks rules 1.1, 1.2, 1.5 and 1.6 automatically, and must pass. egui_kittest's built-in check (unnamed widgets fail) stays on.
3. **Every bug fix has a regression test, and you prove it:** run the test with the fix removed (it must fail), then restored (it must pass).
4. **Real window:** before marking done, run the app, check the change at 1280×800 and 800×500 (both themes for visual changes), and save screenshots as evidence. Arabic/bidi, IME, screen readers and Windows scaling are recorded in R0-04 and UI-10; do not claim them from Linux runs.

## 3. Before every commit

Run from `app/`, and everything must pass:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
```

Also run the structure check from the repository root (0 FAIL required; see the [maintainability skill](../.claude/skills/diagram-studio-maintainability/SKILL.md)):

```sh
python .claude/skills/diagram-studio-maintainability/scripts/maintainability_check.py
```

If a task status changed, run `python build_rust_roadmap.py` in `plan-rust-diagram-editor/` **before committing**. It rejects a `done` task whose prerequisites are not done. Never commit a tracker the validator rejects.

## 4. Documentation for each UI task

- An evidence page in `app/docs/ui/` covering what was built, problems found and fixed, the verification table and screenshots.
- Updates to the shared progress page [app/docs/ui/README.md](../app/docs/ui/README.md), the task's `status` and `evidence` in [roadmap.json](roadmap.json), and [PROGRESS-AND-NEXT-STEPS.md](PROGRESS-AND-NEXT-STEPS.md).
- Status words are exact: `done` only when the task's own acceptance text **and** these rules hold; otherwise `in_progress` with the remaining items listed.
