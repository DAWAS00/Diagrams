# Progress log and next steps

Updated 2026-11-05 after syncing GitHub commit `17c20ea` and running the local Windows checks. **M0 remains incomplete: 3 of 5 tasks done.**

**UI-first scope revision:** added 18 UI tasks, an eight-milestone/57-task tracker, a [full specification](UI-DESIGN-AND-INTEGRATION.md) and [interactive final design](UI-DESIGN.html). These are planned, not implemented. Existing Windows verification remains unchanged.

## Where we are

| Task | Current status | Evidence / remaining work |
|---|---|---|
| R0-01 workspace | Done | Four crates, pinned compiler and shared checks |
| R0-02 native shell | Done | Windows debug/release windows, selection, rename and focus smoke checks passed |
| R0-03 bundled SQLite | Done | MSVC compile/link and temporary-file roundtrip passed; no sqlite3.dll import |
| R0-04 text/accessibility | In progress | Initial English/mixed text and Escape checks passed; full bidi/IME/screen-reader/DPI matrix remains |
| R0-05 foundation review | Not started | Requires completed R0-04 evidence |

Read [the Windows review](../app/docs/WINDOWS-REVIEW-2026-11-05.md) for results, limitations and next work. Windows passed **29 integration tests + 1 doc test**, all **16 Python tests**, fmt/check/Clippy/docs/release build and five generators. The Linux-only display test is excluded on Windows. Local skill additions were preserved. The verification report and task tracker are included together in this update.

## What has been done

### 1. R0-01 — workspace foundation
- Cargo workspace with `diagram-core`, `diagram-canvas`, `diagram-storage` and `diagram-desktop`, and one-way dependencies (desktop → core/canvas/storage).
- Workspace lints: no unsafe code, no `unwrap`, `expect` or `panic` in production code, documented public APIs.
- `ProjectName`: validated, spelling-preserving name with Arabic/mixed-text support.

### 2. Review and test hardening (merged in PR #1)
- Reviewed every file. All Rust checks passed; all five generator scripts reproduced their committed outputs except one.
- **Bug fixed:** `catalog_diagrams.py` appended a duplicate 73-line appendix to `DIAGRAM-CATALOG.md` on every run. It now replaces it, so running it twice changes nothing.
- Tests added for every code path that existed: `ProjectName` (3 → 9 tests), the console bootstrap (0 → 5), and all 15 validation rules of the roadmap generator plus the committed `roadmap.json` (0 → 16, Python).

### 3. R0-02 — native egui shell (PR #2)
- eframe 0.37.0 on the wgpu renderer: toolbar, resizable sidebar and inspector, canvas, status bar.
- Two prototype classes. Select from the canvas or from the sidebar list; Escape and empty-canvas click deselect; Escape in the name field only leaves the field.
- Rename through a real text field; blank names show a written warning.
- Unfinished toolbar actions are disabled with a tooltip. Selection uses border and handles, not colour alone.
- Launch failures print the cause and a hint and exit with code 1; `--version` prints app and SQLite versions.
- 19 tests: 11 headless tests through the accessibility tree, 8 for the executable's diagnostics.
- A Linux release build ran through wgpu on a virtual display ([screenshot](../app/docs/r0-02-shell-linux.png)).

### 4. R0-03 — bundled SQLite (PR #2)
- rusqlite 0.41.1 with SQLite 3.54.1 compiled in; the release binary links no system SQLite.
- Smoke test: write Latin and Arabic text, close, reopen read-only, integrity check, confirm no journal file is left, delete the file.

Full results and the completed Windows table: [R0-02-R0-03-VERIFICATION.md](../app/docs/R0-02-R0-03-VERIFICATION.md).

### Totals
Linux recorded 31 Rust tests; Windows now passes 30 (29 integration + 1 doc test). All 16 Python tests pass. `cargo fmt`, `clippy -D warnings` and `cargo doc` are clean.

## Findings that shape the next steps

- Headless fonts lack Arabic glyphs. The initial Windows mixed-text sample rendered glyphs, but full Arabic/bidi editing and accessibility validation remains R0-04 work.
- egui drops text focus the moment Escape is pressed, so shortcuts need previous-frame focus state. Expect this for every shortcut (F2, Delete, Ctrl+Z).
- On Linux the app needs `libxkbcommon-x11` and a Vulkan or GL driver at run time. Windows needs neither.
- The window follows the system theme; the design plan wants a light theme first.
- The canvas exposes classes to screen readers, but real screen-reader behaviour (NVDA, Narrator) is untested.

## Next steps, in order

### A. Finish M0 (the gate)

1. R0-02/R0-03 Windows verification is complete; do not repeat the prerequisite work without a new change or failure.
2. Finish R0-04 with a dependable licensed font setup, bidi editing/selection, pure Arabic and mixed text, paste/IME, keyboard navigation, Narrator/NVDA and 100/150/200% DPI evidence.
3. Apply the planned light default and text sizing with a dark-theme choice, then review longer labels and smaller windows.
4. Complete R0-05: update ADR-001 using the full results and decide whether the GUI foundation passes.

### B. UX — full native presentation before domain implementation

UI-01, UI-02 and UI-06 are **in progress**: tokens, controls, text roles, the command catalog and Ctrl+K search are built and tested; their remaining items are listed in the [UI progress page](../app/docs/ui/README.md). UI-03 (responsive workspace with docked panels and drawers) is **done**. Next: UI-04 entry screens; complete UI-04–UI-11 surfaces, states, accessible focus paths and notation capability definitions. UI-12 requires these and R0-05. Keep unimplemented application actions disabled; presentation fixtures are clearly labeled.

### C. M1 — model, commands and storage (starts after UI-12)
All headless: no window needed, so it can be tested fully in this environment.
1. **R1-01** Typed IDs and document records (classes, members, relations, diagrams, views); invalid geometry is rejected.
2. **R1-02** Validation: references, ownership, associations, inheritance cycles.
3. **R1-03** Reversible command transactions; a failed command leaves the document untouched.
4. **R1-04** Undo/redo with saved-state tracking.
5. **R1-05** Persistence DTOs and SQLite schema v1; refuse malformed or newer files without rewriting them.
6. **R1-06** Transactional save and Save As with fault injection.
7. **R1-07** Bounded worker protocol so an old save result cannot mark newer edits saved.
8. **R1-08** Headless round-trip fixtures.

R1-01 → R1-02 → R1-03 → R1-04 is one chain; R1-05/R1-06 can run in parallel with R1-03/R1-04.

### D. After M1
- **M2** (13 tasks, including five UI bindings): the full class-editing workflow on the real model — add, connect, edit members, multiple views, delete safely.
- **M3** (8 tasks, including final UI review): reliability, recovery, SVG export, usability checks and the Windows distribution test for **v0.1**.
- **M4–M6**: flowchart and use case, specialised editors, then separately scoped proposals.

## How to keep this page current
Update status and evidence in [roadmap.json](roadmap.json), run `python build_rust_roadmap.py`, then update the tables above. A task is `done` only with implementation and linked evidence.
