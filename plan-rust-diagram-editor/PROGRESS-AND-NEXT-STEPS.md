# Progress log and next steps

Log updated 2026-11-05 after syncing GitHub commit `17c20ea` and running the local Windows checks. The status below is generated from [roadmap.json](roadmap.json), the only status source ([ADR-003](decisions/ADR-003-UI-DECISIONS.md), D6).

<!-- roadmap:summary:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->
**Status** (generated from [roadmap.json](roadmap.json), last updated 2026-11-05): 57 tasks in 8 milestones, 46 of them through v0.1. 14 done, 2 in progress (R0-04, UI-02), 0 blocked, 41 not started. Complete milestones: none. Current gate: M0, 3 of 5 tasks done.
<!-- roadmap:summary:end -->

**UI-first scope revision:** added 18 UI tasks, a [full specification](UI-DESIGN-AND-INTEGRATION.md) and [interactive final design](UI-DESIGN.html). UI progress and evidence: [app/docs/ui/README.md](../app/docs/ui/README.md).

## Where we are

Tasks per milestone:

<!-- roadmap:milestones:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->
| Milestone | Title | Done | In progress | Blocked | Not started | Total |
|---|---|---|---|---|---|---|
| M0 | Native prototype and Rust baseline | 3 | 1 | 0 | 1 | 5 |
| UX | Complete UI design and native presentation | 7 | 1 | 0 | 4 | 12 |
| M1 | Model, commands, history and storage | 4 | 0 | 0 | 4 | 8 |
| M2 | Complete class-modeling workflow | 0 | 0 | 0 | 13 | 13 |
| M3 | Reliability, usability and packaging | 0 | 0 | 0 | 8 | 8 |
| M4 | Flowchart and Use Case adapters | 0 | 0 | 0 | 3 | 3 |
| M5 | Specialized modeling families | 0 | 0 | 0 | 5 | 5 |
| M6 | Broader suite proposals | 0 | 0 | 0 | 3 | 3 |
| **All** | | 14 | 2 | 0 | 41 | 57 |
<!-- roadmap:milestones:end -->

Tasks in progress or blocked:

<!-- roadmap:in-progress:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->
| Task | Title | Milestone | Status |
|---|---|---|---|
| R0-04 | Validate GUI text, focus and accessibility | M0 | In progress |
| UI-02 | Specify and prove typography and font assets | UX | In progress |
<!-- roadmap:in-progress:end -->

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

### A. Finish M0 (the gate before M2)

Sections A, B and C run in parallel: M1 needs only the R0-01 workspace, and M2 needs all three.

1. R0-02/R0-03 Windows verification is complete; do not repeat the prerequisite work without a new change or failure.
2. Finish R0-04 with a dependable licensed font setup, bidi editing/selection, pure Arabic and mixed text, paste/IME, keyboard navigation, Narrator/NVDA and 100/150/200% DPI evidence.
3. Apply the planned light default and text sizing with a dark-theme choice, then review longer labels and smaller windows.
4. Complete R0-05: update ADR-001 using the full results and decide whether the GUI foundation passes.

### B. UX — native presentation (in parallel with M1)

Per-task UI status and evidence are generated on the [UI progress page](../app/docs/ui/README.md). Next: UI-07 canvas states and UI-09 dialogs/status/validation, then UI-02 fonts and UI-10 accessibility checks; complete UI-07–UI-10 surfaces, states, accessible focus paths and notation capability definitions. UI-12 requires these and R0-05. Keep unimplemented application actions disabled; presentation fixtures are clearly labeled.

### C. M1 — model, commands and storage (in parallel with UX)
All headless: no window needed, so it can be tested fully in this environment. M1 needs only R0-01 and runs in parallel with the UX milestone; both gate M2. R1-01 to R1-04 (typed IDs, validation, reversible commands, undo/redo) form one chain; their status is in the tables above. The remaining storage work, in order:
*2026-11-06:* R1-05 and R1-06 are built and tested in slice S10 (`feat/p5-storage-core`, not yet merged). The window does not use them until S11. Their status stays open until S11 holds, as stated in [BUILD-PROGRAM.md](../app/docs/next-steps/BUILD-PROGRAM.md#where-the-build-stands-2026-11-06).

1. **R1-05** Persistence DTOs and SQLite schema v1; refuse malformed or newer files without rewriting them. Per [ADR-003](decisions/ADR-003-UI-DECISIONS.md) D5, this comes before any UI save code.
2. **R1-06** Transactional save and Save As with fault injection.
3. **R1-07** Bounded worker protocol so an old save result cannot mark newer edits saved.
4. **R1-08** Headless round-trip fixtures.

The [execution plan](../app/docs/next-steps/EXECUTION-PLAN.md) Phase 5 breaks this work into files and tests.

### D. After M1
- **M2** (including five UI bindings): the full class-editing workflow on the real model — add, connect, edit members, multiple views, delete safely.
- **M3** (including final UI review): reliability, recovery, SVG export, usability checks and the Windows distribution test for **v0.1**.
- **M4–M6**: flowchart and use case, specialised editors, then separately scoped proposals.

## How to keep this page current
Update status and evidence in [roadmap.json](roadmap.json) and run `python build_rust_roadmap.py`: it regenerates the status summary and tables above, which are never edited by hand (`python build_rust_roadmap.py --check` fails when they are stale). A task is `done` only with implementation and linked evidence.
