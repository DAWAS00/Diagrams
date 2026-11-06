# Slice B1: hand-test bugs (fix/canvas-dark-ink)

Base: `main` at `d202b5b` (S10 merged). Baseline: 287 tests pass; maintainability shows 0 FAIL and 7 WARN. Source: rows 6, 15 and 16 of [progress-test-2026-11-06.md](../ui/progress-test-2026-11-06.md). No roadmap status changes. The slice adds evidence for UI-07 (canvas states) and UI-06 (command catalog).

## Bugs and fixes

| # | Bug | Cause | Fix |
|---|---|---|---|
| 1 | In the dark theme, relationship lines and their multiplicities are almost invisible (contrast about 1.1:1). | `ui/canvas.rs` paints unselected connectors and their labels in `DOCUMENT_INK` (#223147), a document colour. They sit on `palette.canvas`, which follows the theme. Class boxes stay readable because they paint their own light fill. | Add a theme token `Palette::diagram_line`, used for anything painted directly on the canvas surface: connector lines and multiplicity text. Light keeps #223147, so the light theme looks the same; dark uses a light ink. Add a contrast pair "diagram line on canvas" at the text minimum (4.5:1), because the multiplicities are text. Selected connectors keep `accent`; because their multiplicities are text too, a new pair "selected diagram line on canvas" holds `accent` to 4.5:1 (the existing overlay pair only asks 3:1). Class boxes keep their document colours. When S12 builds SVG export, it uses its own document colours on its own background, so the export still never depends on the theme. |
| 2 | With one class shown, the inspector says "1 classes shown". | `format!("{} classes shown", n)` in `ui/inspector_view.rs`. | Add a pure function `read_model::classes_shown(count) -> String` that returns "No classes shown", "1 class shown" or "N classes shown". |
| 3 | Ctrl+Y does not redo. Windows users expect it. | `CommandSpec` has room for only one shortcut; Redo has Ctrl+Shift+Z. | Add `CommandSpec::alternate_shortcut: Option<KeyboardShortcut>` (None by default, set through a `const fn` builder), and set it for Redo only, to Ctrl+Y. `take_shortcut` considers both shortcuts. The text-field rule (`text_field_owns_shortcut`) applies to both, so Ctrl+Y inside a text field is left to the field. The uniqueness test covers primary and alternate shortcuts together. Menus and search keep showing the primary shortcut. The Windows shortcut table in UI-06 gains Ctrl+Y. |

## Tests (written first, each proven to fail without its fix)

- `tests/canvas_ink.rs` covers the sample project in both themes, with the relationship selected and not selected. The test collects the shapes painted in one frame (`harness.output().shapes`) and finds the connector as the line segment whose two ends sit next to a `1` label and a `*` label. That rules out class dividers, other strokes, and the `1` and `*` in the inspector fields. It checks that:
  - the labels have at least 4.5:1 contrast against `palette.canvas`;
  - the line has at least 3:1 and shares the labels' ink;
  - the unselected light-theme ink is still exactly #223147, so the light theme looks the same.
- `tests/theme.rs`: the existing contrast-pair test now includes "diagram line on canvas" and "selected diagram line on canvas" in both themes.
- `tests/ui_audit.rs`: a new scene, the one-class diagram in the dark theme at 1280×800 and 800×500, with the inspector showing "1 class shown".
- `tests/read_model.rs`: `classes_shown` for 0, 1 and 2. `tests/inspector.rs`: opening "Order details" (one class) shows "1 class shown".
- `tests/commands.rs`:
  - after Undo, Ctrl+Y redoes and the status says "Redid: Rename class.";
  - Ctrl+Y while a text field has focus leaves the history alone;
  - no two commands share a shortcut, counting alternates.

## Gates

fmt, clippy `-D warnings`, test and doc from `app/`; maintainability at 0 FAIL with WARN not above 7; roadmap `--check`. Real window, in both themes at 800×500 and 1280×800 (layout also at 1440×900): the connector readable when unselected and when selected, "1 class shown", and Ctrl+Y redo. Deliverables: an evidence page `app/docs/ui/B1-CANVAS-DARK-INK.md` (the fixes, test and proof results, the real-window table), screenshots in `app/docs/ui/b1-canvas-dark-ink/`, and a B1 section in `app/docs/ui/README.md`. The proofs undo only one fix each: only the connector stroke, all canvas ink, the Ctrl+Y alternate, the text-field rule for the alternate, and the plural.
