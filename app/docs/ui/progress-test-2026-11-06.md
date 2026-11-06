# Progress test, 2026-11-06

What was tested: everything built so far. That is `main` (S1, S2 and S3 merged) plus the S10 storage core on branch `feat/p5-storage-core` (commits `cba403a` and `5be8720`). The release build of `diagram-desktop` was run on Windows 11 and driven by mouse and keyboard. Screenshots are in [progress-test-2026-11-06/](progress-test-2026-11-06/).

This is a progress check, not the final acceptance test. That one comes after S13 (`acceptance-2026-11-06.md`).

## Automated checks

| Check | Result |
|---|---|
| `cargo fmt --check` | Pass |
| `cargo clippy --workspace --all-targets -D warnings` | Pass |
| `cargo test --workspace --locked` | **287 passed, 0 failed** |
| `cargo doc --workspace --no-deps` | Pass, no warnings |
| Maintainability check | 0 FAIL, 7 WARN (same as the baseline) |
| `build_rust_roadmap.py --check` | Pass, no status change |

Tests per area:

| Area | Tests |
|---|---|
| Core (`commands`, `model`, `project_name`) | 32 |
| Canvas (`picking`, `transform`) | 12 |
| Desktop window, headless (`commands`, `diagnostics`, `inspector`, `navigator`, `notation`, `read_model`, `session`, `shell`, `showcase`, `theme`, `typography`, `ui_audit`, `welcome`, `workspace`) | 184 |
| Storage, new in S10 (`atomic_save` 15, `load` 23, `recovery` 9, `schema` 7, `sqlite_smoke` 2) | 56 |
| Doc tests | 3 |

S10 has no window yet; it is used by the window in S11. The storage tests prove:
- the round trip is exact (Arabic names, ID counter, several diagrams);
- a failed save at each step (create, write, flush, verify, replace) leaves the old file byte for byte;
- newer-format, foreign, write-ahead-log and damaged files are refused untouched;
- a leftover journal next to the target stops the save;
- recovery snapshots stay apart per project.

## Hand test in the real window

| # | What was done | Expected | Result | Screenshot |
|---|---|---|---|---|
| 1 | Start the app at 1280×800 | Welcome with New project and sample; Open project disabled (no files yet) | Pass | 01 |
| 2 | Continue with the sample project | Two classes and a relationship read from the real document; project panel lists diagrams, model and relationships | Pass | 02 |
| 3 | Click Customer on the canvas | Selection handles; inspector shows name, attributes, "Shown in Class diagram 1"; status bar "Selected: Customer" | Pass | 03 |
| 4 | Rename to Client in the inspector, Enter | Canvas, model list and relationship label change together; Undo turns on | Pass | 04 |
| 5 | Escape, Ctrl+Z | Name back to Customer; status "Undid: Rename class." | Pass | 05 |
| 6 | Ctrl+Y | Redo (the usual Windows shortcut) | **Fail**: nothing happens. Only Ctrl+Shift+Z is mapped | — |
| 7 | Ctrl+Shift+Z | Name is Client again; status "Redid: Rename class." | Pass | 06 |
| 8 | Select the relationship in the panel | Line drawn thicker in the accent colour; inspector shows both roles and multiplicities | Pass | 07 |
| 9 | Type `5..2` as a multiplicity, Enter | Error in words below the field, red border, canvas unchanged | Pass | 08 |
| 10 | Type `0..5`, Enter | Canvas label becomes `0..5` | Pass | 09 |
| 11 | Menu > File > Close project with changes | Dialog explains changes will be lost; Cancel has focus | Pass | 10, 11, 12 |
| 12 | Escape in the dialog | Dialog closes, nothing lost, focus back on Menu | Pass | 13 |
| 13 | Close project > Discard changes and close | Back to Welcome | Pass | 14 |
| 14 | New project "Library", Enter; then Close project | Empty project with one diagram and empty-state text; closing an unchanged project asks nothing | Pass | 15, 16, 17 |
| 15 | Sample in dark theme at 800×500 and 1440×900 | Everything fits; relationship readable | **Fail**: the relationship line and its `1` / `*` labels are the same colour as the dark canvas and almost invisible. Already noted at S2 as "known", but no slice had owned it until now | 18, 19, 20 (zoomed) |
| 16 | Ctrl+K, type "close"; Escape; open "Order details" | Search finds File: Close project; Escape closes it; a second tab opens | Pass, but the inspector says **"1 classes shown"** (wrong plural) | 21, 22 |

Fits at 800×500, 1280×800 and 1440×900: yes, nothing overlapped or was cut off. Light and dark both checked.

## Findings and where they go

All three go to bug slice **B1** (`fix/canvas-dark-ink`), recorded in [BUILD-PROGRAM.md](../next-steps/BUILD-PROGRAM.md). Each fix starts with a test that fails without it.

1. **Relationship ink on the dark canvas.** `ui/canvas.rs` draws connectors and multiplicities in `DOCUMENT_INK` (#223147) on `palette.canvas`, which is dark in the dark theme. The class boxes stay readable because they paint their own light fill. Fix: give connectors an ink that contrasts with the canvas they are drawn on (or paint the diagram on its own document page), with a contrast test in both themes.
2. **Plural.** `ui/inspector_view.rs:116` writes `"{n} classes shown"`. It should say "1 class shown".
3. **Ctrl+Y.** `ui/commands.rs` maps Redo to Ctrl+Shift+Z only. Windows users expect Ctrl+Y as well; the catalog needs a second shortcut and the shortcut table in the UI rules needs that line.

Not bugs, but not built yet (disabled with a reason): Class, Association, Note, Arrange, Pan, New diagram, Save, Export, Open, Add attribute and Add operation. The Undo and Redo buttons do not yet show what they will undo; that is S8.
