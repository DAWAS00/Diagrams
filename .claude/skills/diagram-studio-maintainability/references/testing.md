# Testing guide (with examples from this repo)

## The layers

| Layer | Where | Use for | Example |
|---|---|---|---|
| Pure unit test | `tests/<area>.rs` calling pure functions | Rules, tables, maths | `inspector::multiplicity_error("5..2")` returns an error |
| Headless UI test | `tests/<surface>.rs` with `egui_kittest::Harness` | Behaviour through the real shell | Click "Close Class diagram 1", then check that the classes are still in the project |
| Usability audit | `tests/ui_audit.rs` | Every screen state: names, sizes, overlaps, Tab reach | `inspector_states_should_pass_the_audit` |
| Real window | Run the app (Xvfb on Linux) | Visual and layout confirmation | Screenshots in `app/docs/ui/` |

## Writing a headless UI test

```rust
fn harness() -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, ShellState::default());
    harness.run();
    harness
}

#[test]
fn name_should_stay_a_draft_until_enter() {
    let mut harness = harness();
    harness.state_mut().select(0);          // fixture setup
    harness.run();
    harness.get_by_label("Name").focus();   // focus first: type_text needs it
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label("Name").type_text("Client");
    harness.run();

    assert_eq!(harness.state().classes[0].name, "Customer"); // not written yet
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().classes[0].name, "Client");
}
```

- **Arrange** with the fewest direct field writes. Prefer `state.queue(UiIntent::…)` or a click.
- **Act** through the accessibility tree: `click`, `click_modifiers(Modifiers::COMMAND)`, `key_press`, `type_text`.
- **Assert** on observable results: state reached through methods, nodes found by label or role, `accesskit_node().toggled()`, `.is_disabled()`, `.value()`.

## Pitfalls already hit here

| Pitfall | What to do |
|---|---|
| `type_text` does nothing | Call `focus()` and `run()` first, unless the field auto-focuses |
| `get_all_by_label` panics when nothing matches | Use `query_all_by_label` to check that something is absent |
| Two nodes share a label (a tab and a panel row, or the status bar and the inspector) | Pick by role (`Role::Tab`) or count with `get_all_by_label(..).count()` |
| Submenus do not open on click | `hover()` the submenu button, then `run()` |
| Panic: "No glyph for …" | The harness forbids missing glyphs. Use words instead of symbols in the UI; call `allow_missing_glyphs()` only for intentional multilingual test data |
| A drawing change makes the first frame panic | Styles installed mid-frame must also reach the current `Ui` (`ui.set_style(...)`) |
| Escape is swallowed | Shortcuts must use `text_edit_focused()`, never `egui_wants_keyboard_input()` |
| A custom clickable area is invisible to tests | Call `response.widget_info(...)` with a role and a name |
| Scrolled content reported as overlapping | Wrap it in `scrolling(...)` so it is a clipping `ScrollView` |

## Proving a regression test

```sh
# 1. Temporarily remove the fix (keep a copy of the file)
# 2. The new test must FAIL
cargo test -q -p diagram-desktop --test <file> <test_name> --locked
# 3. Restore the fix; the test must PASS
```

Record in the evidence page that you did this. A test that never failed proves nothing about the bug it names.

## Adding a screen to the usability audit

Add a `#[test]` in `tests/ui_audit.rs` that builds the state, then calls `audit_layout(...)` and `audit_tab_order(...)`, and finally `report(findings)`. Pass `floating: true` for dialogs, menus and windows, which cover other controls by design. Exclude canvas boxes under a drawer only on narrow windows, and say why in a comment.
