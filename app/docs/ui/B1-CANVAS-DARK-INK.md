# B1: fixes from the 2026-11-06 hand test

Slice plan: [SLICE-b1-canvas-dark-ink.md](../next-steps/SLICE-b1-canvas-dark-ink.md). The bugs came from rows 6, 15 and 16 of [progress-test-2026-11-06.md](progress-test-2026-11-06.md). No roadmap status changes; this is evidence for UI-07 and UI-06.

## What changed

| Bug | Fix |
|---|---|
| Relationship line and multiplicities almost invisible on the dark canvas (1.09:1) | New theme token `Palette::diagram_line` for whatever is painted straight onto the canvas: #223147 in light (unchanged) and #C9D4E3 in dark. Two new contrast pairs: "diagram line on canvas" and "selected diagram line on canvas" (accent), both at 4.5:1. Class boxes keep their document colours. |
| "1 classes shown" | `read_model::classes_shown`: "No classes shown", "1 class shown", "N classes shown". |
| Ctrl+Y did not redo | `CommandSpec::alternate_shortcut`. Redo answers Ctrl+Shift+Z and Ctrl+Y. A text field with focus keeps both keys, and no key combination belongs to two commands. |

## Tests

7 new tests (294 in total, all passing):
- `canvas_ink` (2): finds the painted connector by its ends next to its labels, selected and unselected, in both themes.
- `ui_audit` (1): the one-class diagram in dark at 1280×800 and 800×500.
- `commands` (2): Ctrl+Y redoes; Ctrl+Y inside a text field is left to the field.
- `inspector` (1) and `read_model` (1): the plural.

The existing contrast-pair tests now cover the two new pairs, and the shortcut-uniqueness test now counts alternates.

Each fix was undone on its own to prove its test catches it. All five undone versions fail:
- only the connector stroke back to the document ink;
- all canvas ink back to the document ink;
- no Ctrl+Y alternate;
- the alternate ignoring the text-field rule;
- the plural back to "classes".

Gates: fmt, clippy `-D warnings`, test and doc are clean; maintainability is at 0 FAIL and 7 WARN (no change); the roadmap check passes.

## Real window (Windows 11, release build)

| What | Result | Screenshot |
|---|---|---|
| Sample in dark, 1280×800: line, `1` and `*` readable | Pass | [01](b1-canvas-dark-ink/01-dark-1280x800.png) |
| Relationship selected in dark, 1280×800 and 800×500: accent line and labels readable | Pass | [02](b1-canvas-dark-ink/02-dark-selected-1280x800.png), [03](b1-canvas-dark-ink/03-dark-selected-800x500.png) |
| Light at 800×500 (selected) and 1440×900: looks as before | Pass | [04](b1-canvas-dark-ink/04-light-selected-800x500.png), [05](b1-canvas-dark-ink/05-light-1440x900.png) |
| "Order details" diagram: inspector says "1 class shown" | Pass | [06](b1-canvas-dark-ink/06-one-class-shown.png) |
| Rename, then Undo, then Ctrl+Y redoes ("Redid: Rename class."), then Ctrl+Z undoes ("Undid: Rename class.") | Pass | [07](b1-canvas-dark-ink/07-after-undo.png), [08](b1-canvas-dark-ink/08-ctrl-y-redo.png), [09](b1-canvas-dark-ink/09-ctrl-z-undo.png) |

Note on the test tool, not the app: partway through, keys sent with .NET `SendKeys` stopped delivering the Ctrl modifier, so Ctrl+A, Ctrl+Z and Ctrl+K all appeared to do nothing. The same keys sent as raw key events (`keybd_event`) worked, and so did the Undo button. The screenshots above use those working inputs.
