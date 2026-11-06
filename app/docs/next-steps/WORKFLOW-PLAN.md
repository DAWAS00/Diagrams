# Workflow plan: how Claude and Codex build Diagram Studio

Date: 2026-11-06. This plan was agreed between Claude and Codex. Codex wrote its proposal with `codex-cli 0.167.0` at reasoning effort `xhigh`, in read-only mode. This file covers *how* the work is done. [EXECUTION-PLAN.md](EXECUTION-PLAN.md) covers *what* is built, and [IDEAS-PLAN.md](IDEAS-PLAN.md) covers *why*.

## 1. Roles

| Who | Does | Never does |
|---|---|---|
| **Claude** | Writes all code, tests, docs and tracker updates. Runs the checks. Launches the app and tests it by hand. Commits, pushes and opens pull requests once the gates below pass. | Pushes before every gate passes. Marks a task `done` without evidence. |
| **Codex** (`xhigh`, read-only) | Reviews each slice plan with `/codex-review:plan` and each finished diff with `/codex-review:code`, for up to 5 rounds each. Reviews the tests and names missing cases taken from the acceptance text. | Edits files. Approves work it has not seen in full. |
| **You** | Settle decisions D1 to D6. Approve each push or pull request. Take part in the final hands-on test if you want to. | — |

**Codex reviews the tests but does not write them** (both agreed). This keeps a single author for the code and keeps Codex strictly read-only. To catch blind spots the two share, Codex derives test cases from the acceptance text, and Claude must add them.

**Running Codex at `xhigh`.** The review plugin leaves out `-m`, so Codex uses its own defaults. Every Codex call adds `-c model_reasoning_effort="xhigh"`. A one-time alternative is to set `model_reasoning_effort = "xhigh"` in `~/.codex/config.toml`.

## 2. Branches and pull requests

- **One branch and one pull request per small, complete slice**, grouped under its phase: for example `feat/p1-session-binding` or `refactor/p0-split-shell`. A small phase can be one slice. Every slice leaves the app usable.
- **Work in the main checkout `D:\REA visual`**, as `CLAUDE.md` requires, on a normal branch. Do not use worktrees. This session is still pinned to the `.claude` worktree, so the build starts in **a new session opened in `D:\REA visual`**.
- Commit structural moves (file splits) separately from behaviour changes, one surface per move.
- Order: Phase 0, then Phase 1 (binding, which every later phase needs). After that, Phase 2 and 3, with the storage schema (Phase 5a) alongside. Then Phase 4, 5, 6, and Phase 7 last.

## 3. The loop for every slice

```
plan ──► Codex plan review ──► baseline ──► failing tests ──► code ──► local gates
                                                                          │
   push + PR ◄── seal ◄── real-window check ◄── Codex code review (APPROVED)
```

1. **Plan the slice.** Write the scope, the roadmap acceptance text, the affected modules, the test cases and the manual checks. Run `/codex-review:plan` and resolve its findings before coding.
2. **Baseline.** Record the base commit, the current test count and the current maintainability WARN count (11 today).
3. **Tests first.** Write behaviour tests that fail first. Write pure logic in core or canvas before wiring the UI.
   - Every bug fix gets a regression test, shown to fail without the fix and pass with it. Record the command and the real failure; a failure to compile does not count.
4. **Write the code** following the structure rules in section 4.
5. **Local gates**, all of which must pass:
   - From `app/`:
     ```sh
     cargo fmt --all -- --check
     cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
     cargo test --workspace --locked
     cargo doc --workspace --no-deps --locked
     ```
   - From the repository root:
     ```sh
     python .claude/skills/diagram-studio-maintainability/scripts/maintainability_check.py
     ```
     This must show **0 FAIL**, and the WARN count must not rise.
   - If a task status changed, also run `python build_rust_roadmap.py` from `plan-rust-diagram-editor/`.
6. **Codex code review.** Freeze edits. Run `/codex-review:code` on the full candidate: the diff, the new files, the tests and the gate output, tagged with the base commit and a diff hash. Claude fixes the findings and reruns the affected gates until **`VERDICT: APPROVED`**.
   - If Codex has not approved after 5 rounds, the slice is **blocked**, not approved. It comes to you with Codex's open findings.
7. **Real-window check.** Build and launch `diagram-desktop`, then exercise the slice with the mouse and the keyboard. For UI changes, save screenshots at **800×500, 1280×800 and 1440×900**, in light and dark theme, under `app/docs/ui/<slice>/`. Update `app/docs/ui/README.md`.
8. **Seal and deliver.** Any edit after approval means a fresh review, and the gates run again. Then commit, check that the commit matches the approved diff hash, push and open the pull request. **Nothing is pushed before all of steps 1 to 7 pass.**

## 4. Structure rules Codex checks in every review

- **Crate direction:**
  - desktop → core, canvas and storage;
  - canvas and storage → core;
  - core holds no GUI or SQL code;
  - canvas and storage never depend on widgets.
- **Frame flow:**
  - Widgets read state and edit drafts, then queue a `UiIntent`.
  - Only the coordinator applies changes, through `diagram-core` commands, so Undo works.
  - Results from the save worker come back through the coordinator too.
- **Identity:** use typed `ElementId`, `ViewId`, `RelationId` and `DiagramId`, never list indexes. Test stale targets after delete and undo.
- **No I/O while painting.** Pure rules stay apart from drawing. New state stays private behind methods, to work down the debt of 19 public fields in `ShellState`.
- **Size:**
  - Give each surface its own module, aiming for under about 500 lines and splitting before 1,000.
  - `shell.rs` (1,659 lines) is split first, on the surfaces each slice touches.
- **Single sources:**
  - Shortcuts live in `ui/commands.rs`, tokens in `ui/theme.rs`, and notation rules in core.
  - Every new screen state gets accessibility-tree tests and an entry in `tests/ui_audit.rs`.

## 5. Final hands-on test of everything shipped

After the last slice of a phase, and again at the end, Claude builds the approved commit and launches the Windows app. The commit, build profile and screen size are recorded in `app/docs/ui/acceptance-<date>.md`. Each new feature gets a row:

| Feature | Steps | Expected | Actual | Mouse | Keyboard | Undo/Redo | Errors and focus | Sizes and themes | Evidence |
|---|---|---|---|---|---|---|---|---|---|
| e.g. Create class | Class tool, click canvas, type name, Enter | Class appears with the name being edited | … | ✓/✗ | ✓/✗ | ✓/✗ | ✓/✗ | ✓/✗ | screenshot link |

The end-to-end journey is **create → edit members → relate → reuse in a second diagram → save → close → reopen → export SVG**. Where the Phase 7 gate applies, Arabic text, IME, NVDA and Windows scaling are checked as well. If a row fails, a bug slice is opened, starting with a regression test.

## 6. Risks of a two-agent workflow

| Risk | Guard |
|---|---|
| Codex rubber-stamps | Its findings must cite acceptance text, and it must propose at least one adversarial case per slice. |
| Approval of a stale diff | Freeze the candidate and record its hash; any later edit restarts the review. |
| Flaky UI tests | Find widgets by accessibility name, use the shared `tests/common` harness, finish workers deterministically and take geometry from nodes. |
| Green tests, broken app | The real-window check (step 7) is mandatory; headless tests or nice screenshots never replace working behaviour. |
| Both agents share a blind spot | Ask you about decisions D1 to D6. Run the hands-on test, ideally with you watching. |

## 7. First slices to run

1. `docs/p0-decisions`: record D1 to D6 as ADR-003, and make `roadmap.json` the only status source.
2. `refactor/p0-split-shell`: split `shell.rs` by surface and add `tests/common/mod.rs`, with no behaviour change.
3. `feat/p1-session-binding`: add `session.rs` and `ui/read_model.rs`, switch intents to typed IDs, and add Close project with a return to Welcome.

Each slice goes through the loop in section 3 before the next one starts.
