# Progress log and next steps

Updated 2026-11-04. **No application milestone is complete.** M0 (native prototype gate) is in progress: 1 of 5 tasks is done, 2 are implemented and waiting for Windows checks, 2 have not started. Task status lives in [roadmap.json](roadmap.json); this page explains what was built, how it was checked, and what comes next.

## Where we are

```text
M0  Native prototype gate   R0-01 done · R0-02 in progress · R0-03 in progress · R0-04 · R0-05
M1  Model, commands, storage          (blocked until the M0 gate, R0-05, passes)
M2  Complete class editor
M3  Reliability and v0.1 release
M4–M6  Later diagram families and proposals
```

| Task | Status | What is true today |
|---|---|---|
| R0-01 Cargo workspace | **Done** | Four crates, pinned Rust 1.99.1, lints, `ProjectName`. Evidence: [R0-01-VERIFICATION.md](../app/docs/R0-01-VERIFICATION.md) |
| R0-02 Native egui shell | **In progress** | Built and tested on Linux. Windows debug and release launch not yet recorded |
| R0-03 Bundled SQLite | **In progress** | Built and tested on Linux. Windows MSVC build not yet recorded |
| R0-04 Text, focus, accessibility | Not started | Needs a real Windows desktop |
| R0-05 Native foundation gate | Not started | Depends on R0-02, R0-03 and R0-04 |

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

Full results and the Windows table to fill in: [R0-02-R0-03-VERIFICATION.md](../app/docs/R0-02-R0-03-VERIFICATION.md).

### Totals
31 Rust tests (30 integration + 1 doc test) and 16 Python tests pass. `cargo fmt`, `clippy -D warnings` and `cargo doc` are clean.

## Findings that shape the next steps

- The headless test fonts have no Arabic glyphs, so **Arabic rendering is unverified** in the real window. This is R0-04's main risk.
- egui drops text focus the moment Escape is pressed, so shortcuts need previous-frame focus state. Expect this for every shortcut (F2, Delete, Ctrl+Z).
- On Linux the app needs `libxkbcommon-x11` and a Vulkan or GL driver at run time. Windows needs neither.
- The window follows the system theme; the design plan wants a light theme first.
- The canvas exposes classes to screen readers, but real screen-reader behaviour (NVDA, Narrator) is untested.

## Next steps, in order

### A. Finish M0 (the gate)
| # | Step | Who | Done when |
|---|---|---|---|
| 1 | Merge PR #2, run `app\scripts\check.ps1` and launch the debug and release builds on Windows; fill in the Windows table | **You** (needs Windows) | R0-02 and R0-03 can be set to `done` with evidence |
| 2 | Bundle an English + Arabic font, check its licence, keep its notice | Claude | Arabic and mixed text render in the shell on Windows, no empty boxes |
| 3 | Set the light theme and 14–16 px text as the default, keeping dark mode selectable | Claude | Theme follows the design plan; tests still pass |
| 4 | R0-04 test matrix: English, Arabic, mixed text; paste; IME; Tab/arrow focus; NVDA and Narrator; 100/150/200% scaling | **You** with Claude writing the checklist and fixes | A results table in `app/docs/`, failures fixed or listed as blockers |
| 5 | R0-05: update ADR-001 with real results and decide whether egui stays | Claude drafts, **you** decide | ADR-001 states the decision with evidence; M0 gate marked passed |

Steps 2 and 3 can be done now, before the Windows checks, because they do not depend on them.

### B. M1 — model, commands and storage (starts after R0-05)
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

### C. After M1
- **M2** (8 tasks): the full class-editing workflow on the real model — add, connect, edit members, multiple views, delete safely.
- **M3** (7 tasks): reliability, recovery, SVG export, usability checks and the Windows distribution test for **v0.1**.
- **M4–M6**: flowchart and use case, specialised editors, then separately scoped proposals.

## How to keep this page current
Update status and evidence in [roadmap.json](roadmap.json), run `python build_rust_roadmap.py`, then update the tables above. A task is `done` only with implementation and linked evidence.
