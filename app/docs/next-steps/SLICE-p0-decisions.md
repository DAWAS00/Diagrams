# Slice plan: `docs/p0-decisions`

Date: 2026-11-06. Phase 0, slice 1 of the [workflow plan](WORKFLOW-PLAN.md) section 7. It covers the first two rows of [EXECUTION-PLAN.md](EXECUTION-PLAN.md) Phase 0: record decisions D1 to D6, and make `roadmap.json` the only status source. It changes no Rust code and no UI.

## Baseline

| Item | Value |
|---|---|
| Base commit | `96b1fae` (`main` = `origin/main`) |
| Branch | `docs/p0-decisions`, main checkout `D:\REA visual`, no worktree |
| Python tests | 17 pass (`python -m unittest test_build_rust_roadmap` from `plan-rust-diagram-editor/`) |
| Maintainability check | 0 FAIL, 11 WARN |
| Roadmap | 57 tasks in 8 milestones: 14 done, 2 in progress (R0-04, UI-02), 0 blocked, 41 not started |
| Generator | `python build_rust_roadmap.py` reproduces the committed `ROADMAP.html` and `rust-backlog.csv` (no content diff after git's LF normalization) |

## Decisions (confirmed by the user on 2026-11-06)

The user accepted every recommendation in [IDEAS-PLAN.md](IDEAS-PLAN.md) section 5:

| ID | Decision | Code fact it rests on |
|---|---|---|
| D1 | Draw generalization from child to parent. The inspector shows "Parent: X, child: Y". | Core `Relation.source` is the child and `target` the parent (`diagram-core/src/model.rs:241`, `command.rs:92`). |
| D2 | When a typed class name already exists, say so in words and offer "Show the existing class here" or "Keep a new class". Never switch to the existing model without asking. | Core has no duplicate-name rule; the choice is a UI rule. |
| D3 | v0.1 ships association and generalization only. `RelationKind` stays extensible. Revisit after R2-08. | `RelationKind` (`diagram-core/src/notation.rs:41`) has 2 variants and is not `#[non_exhaustive]`. |
| D4 | A new member shows in every diagram that shows the class, with no prompt. | Members belong to the shared model element, not to a view. |
| D5 | Keep ADR-002 (one SQLite file per project). Define schema v1 (R1-05) before any UI save code. | ADR-002 is the storage proposal; R1-05 is not started. |
| D6 | `roadmap.json` is the only status source; summaries elsewhere are generated. | Five documents carry hand-copied counts that are now wrong (below). |

## Scope

### 1. ADR-003

New file `plan-rust-diagram-editor/decisions/ADR-003-UI-DECISIONS.md` in the ADR-001/ADR-002 format: title, `Status: accepted. Date: 2026-11-06.`, then Context, Alternatives (one table, a row per decision), Decision (D1 to D6 with their consequences for later phases), Consequences, Revisit trigger. ADR-002 itself is not edited; ADR-003 records that D5 confirms its direction and that ADR-002 moves to accepted when R1-05/R1-06 evidence lands.

Link ADR-003 from `IDEAS-PLAN.md` section 5, `EXECUTION-PLAN.md` Phase 0 and `plan-rust-diagram-editor/README.md`.

### 2. Generated status blocks (D6)

Extend `plan-rust-diagram-editor/build_rust_roadmap.py`. The HTML template, the CSV code and their outputs stay unchanged.

- **Markers.** A block is the text between `<!-- roadmap:NAME:begin ... -->` and `<!-- roadmap:NAME:end -->` on their own lines. The begin marker carries a note that the block is generated from `roadmap.json`.
- **Pure functions**, each taking the parsed roadmap and returning data or text:
  - `status_counts(data)`: validates first, then returns totals per status (default status applied), per-milestone counts in milestone order, the v0.1 task count (milestones up to and including the first whose `release` is `v0.1`; `None` when no milestone has that release), IDs in progress and IDs blocked, complete milestones (all tasks done, at least one task), and the current gate (first milestone not complete) with its done/total, or `None` when every task is done.
  - `render_summary(data, link)`: one paragraph: date `last_updated`, totals, v0.1 count (or "no milestone is marked v0.1"), per-status counts with in-progress and blocked IDs, complete milestones ("none" when empty), current gate ("none, every task is done" when complete).
  - **Escaping.** Every value from `roadmap.json` that enters Markdown goes through one `cell()` helper: CR/LF become spaces; the Markdown characters `` \ ` * _ [ ] ~ | `` are backslash-escaped, so values show literally (no links, images, emphasis or code); `&`, `<` and `>` become entities. (Added after code review round 1.) So a title can never produce a roadmap marker, an HTML tag or a broken table row. Link destinations are percent-encoded with `urllib.parse.quote` (keeping `/`).
  - Evidence may be a string or a list (the validator accepts both); `evidence_path()` normalizes it before taking the first entry.
  - `render_milestones(data)`: Markdown table `Milestone | Title | Done | In progress | Blocked | Not started | Total` plus a totals row.
  - `render_in_progress(data)`: table of in-progress and blocked tasks (`Task | Title | Milestone | Status`), or a sentence saying there are none.
  - `render_ui_tasks(data, link)`: table of `UI-` tasks in roadmap order: `Task | What it delivers (title) | Status`, where Status is `**Done**`, `In progress`, `Blocked` or `Not started`, followed by `— [evidence](...)` when the task has evidence. The link target is the path before the first `:` of the first evidence entry, made relative to the Markdown file.
  - `fill_blocks(text, blocks)`: replaces the content of every marked block with `blocks[name]`. It raises `ValueError` naming the problem on: a missing begin or end marker for a requested block, a duplicate marker, an end before its begin, an unknown block name in the file, or nested markers. Text outside the blocks is returned unchanged.
- **Target table** `GENERATED_BLOCKS`, paths relative to the repository root:

  | File | Blocks |
  |---|---|
  | `README.md` | summary |
  | `plan-rust-diagram-editor/README.md` | summary |
  | `plan-rust-diagram-editor/RUST-ROADMAP.md` | summary |
  | `plan-rust-diagram-editor/05-ROADMAP.md` | summary |
  | `plan-rust-diagram-editor/PROGRESS-AND-NEXT-STEPS.md` | summary, milestones, in-progress |
  | `app/docs/ui/README.md` | ui-tasks |

- **`main(argv=None, root=ROOT)`** runs in this order: load and validate `roadmap.json` → read every target and render all its blocks in memory, checking every marker → only if all of that succeeded, write the CSV and HTML exactly as today → write each changed Markdown file. A marker problem in any target is reported as `error: <file>: <problem>` on stderr with exit code 2, and nothing is written (no CSV, HTML or Markdown). Comparison uses `\n` newlines; a Markdown file is written only when its text changes (an unchanged file keeps its modification time), keeping the file's own line ending (CRLF working copies stay CRLF; git normalizes to LF). Each write goes to a temporary file in the same folder and then `os.replace`s the target, so an interrupted run never leaves a half-written document. Output paths come only from `GENERATED_BLOCKS` and must resolve inside the repository.
- **`--check`** renders every block in memory, writes nothing (no Markdown, HTML or CSV), prints each stale file and exits 1; it exits 0 when all blocks are current. It does not check the HTML and CSV, whose bytes depend on the platform newline. An unknown argument exits 2 with a usage line.
- **Entry point:** `raise SystemExit(main())`, so the exit code reaches the shell.

### 3. Remove stale hand-written status text

Each sentence below is replaced by a generated block or removed. Wording is otherwise kept.

| File:line (at `96b1fae`) | Stale text | Change |
|---|---|---|
| `README.md:7` | "R0-01, R0-02 and R0-03 are verified … M0 remains open" | summary block; keep "Real diagram editing and project persistence are not implemented yet." |
| `README.md:14` | link text "57-task backlog" | "Task backlog (CSV)" |
| `plan-rust-diagram-editor/README.md:3` | "R0-01/R0-02/R0-03 are verified …; R0-04 is in progress and R0-05 remains pending" | summary block below the plan date |
| `plan-rust-diagram-editor/README.md:13` | "tracks 57 tasks …; 46 lead through v0.1. Three tasks are complete; … 53 tasks are not started" | drop the counts; keep the links and the regenerate instruction |
| `RUST-ROADMAP.md:3` | status sentence | summary block |
| `RUST-ROADMAP.md:21` | "R0-01/R0-02/R0-03 are `done`; R0-04 is `in_progress`; 53 tasks remain `not_started`" | removed |
| `RUST-ROADMAP.md:87` | "R1-01 now requires UI-12" (contradicts the 2026-11-05 decision tested in `test_committed_ui_tasks_gate_the_right_milestones`) | "R1 runs in parallel with the UX milestone; R2-08 requires UI-12." |
| `RUST-ROADMAP.md:89` | "57 tasks across eight milestones; 46 fall through v0.1. New UI tasks are all not started." | removed |
| `RUST-ROADMAP.md:156` | "R0-02/R0-03 … stay in progress …; 36 tasks remain not started" | points to the summary block and `roadmap.json` |
| `05-ROADMAP.md:7` | "Status on 2026-11-05: R0-01/R0-02/R0-03 complete; …" | summary block |
| `05-ROADMAP.md:24` | "All new tasks are pending." | removed |
| `PROGRESS-AND-NEXT-STEPS.md:3,5` | "M0 remains incomplete: 3 of 5 tasks done"; "These are planned, not implemented" | summary block; the UI-first note keeps its links only |
| `PROGRESS-AND-NEXT-STEPS.md:9-15` | hand-copied M0 table | milestones and in-progress blocks |
| `PROGRESS-AND-NEXT-STEPS.md:68` | per-task UI status prose | link to the UI progress page; keep the "Next" order |
| `PROGRESS-AND-NEXT-STEPS.md:70-81` | section C lists R1-01 to R1-04 as future work "after UI-12" | section C lists the remaining M1 work R1-05 to R1-08, says R1 runs in parallel with UX, and links EXECUTION-PLAN Phase 5 |
| `PROGRESS-AND-NEXT-STEPS.md:84-85` | "M2 (13 tasks …)", "M3 (8 tasks …)" | counts dropped (the milestones block has them) |
| `PROGRESS-AND-NEXT-STEPS.md:89` | "then update the tables above" | "the status blocks regenerate; never edit them by hand" |
| `app/docs/ui/README.md:21-36` | hand-written UI status table | ui-tasks block |

**Ordering guidance that contradicts parallel R1/UX work.** The 2026-11-05 decision (tested in `test_committed_ui_tasks_gate_the_right_milestones`) is that M1 needs only M0's `R0-01`, runs in parallel with UX, and both gate M2 (`R2-08` needs `UI-12`, `UI-13` needs `R1-08`). These still say otherwise and are rewritten to that rule, pointing to the generated status and [EXECUTION-PLAN.md](EXECUTION-PLAN.md) for current tasks instead of naming them by hand:

| File:line | Stale text | Change |
|---|---|---|
| `plan-rust-diagram-editor/README.md:15` | "UX design/native presentation now precedes M1" | "UX design runs in parallel with M1; both gate M2" |
| `plan-rust-diagram-editor/README.md:49` | "Finish R0-04/R0-05 while starting UI-01/UI-02 … UI-12 review before M1" | next action points to the summary block and the execution plan's phase order |
| `RUST-ROADMAP.md:9-10` | Mermaid `M0 --> UX --> M1` | `M0 --> UX`, `M0 --> M1`, `UX --> M2`, `M1 --> M2` |
| `RUST-ROADMAP.md:85` | heading "UX — Complete the native UI design before M1" | "UX — Native UI design (in parallel with M1)" |
| `05-ROADMAP.md:24` | "complete UI-01–UI-12 before M1" | "UX runs in parallel with M1; UI-12 gates M2 through R2-08" |
| `PROGRESS-AND-NEXT-STEPS.md:66` | heading "UX — full native presentation before domain implementation" | "UX — native presentation (in parallel with M1)" |
| `app/docs/ui/README.md:17` | order of work "UX … → M1 → M2" | "M0 → UX and M1 in parallel → M2 → M3" |
| `README.md:28` | "Eighteen new UI tasks prioritize native presentation before M1" | "Eighteen UI tasks: native presentation runs in parallel with M1, with real functional bindings in M2/M3" |

Tests 11 and 12 below guard both lists.

### Security

No application runtime boundary changes. The generator reads only `roadmap.json` and the files in `GENERATED_BLOCKS` and writes only those plus the existing HTML and CSV. Roadmap text is escaped before it enters Markdown (see Escaping), so a task title cannot inject HTML, markers or table rows.

Left alone: dated reports (`HANDS-ON-VP-TEST-VS-RUST-BUILD.md`, `WINDOWS-REVIEW-2026-11-05.md`, `R0-02-R0-03-VERIFICATION.md`), the dated test-count log in `PROGRESS-AND-NEXT-STEPS.md` ("Windows passed 29 integration tests …", "Totals"), the agent logs under `agents/` (A-n and B-n are not roadmap tasks), and the untracked folder `app/docs/ui/windows-test-2026-11-05/`.

## Tests first (`plan-rust-diagram-editor/test_build_rust_roadmap.py`)

Written before the code; each must fail first (missing function or wrong output, recorded in the PR body).

1. `status_counts`: counts per status with the default status applied to tasks with no `status`; a `blocked` task is counted and listed; per-milestone rows sum to the totals; v0.1 count stops at the first `v0.1` milestone; complete milestones and current gate are right for an all-done milestone followed by a partial one; all tasks done gives no gate; no `v0.1` milestone gives `None`; an invalid roadmap raises.
2. `render_summary`: contains every count and the in-progress IDs; says "0 blocked" and "Complete milestones: none" when so; all done says "Current gate: none, every task is done"; no `v0.1` milestone says "no milestone is marked v0.1".
3. `render_milestones`: one row per milestone in order plus a totals row equal to the task count.
4. `render_in_progress`: lists in-progress and blocked tasks; prints the "none" sentence when there are none.
5. `render_ui_tasks`: only `UI-` tasks, roadmap order; evidence link is relative to the target file and stops before the first `:`; evidence given as a string and as a list both work; a task without evidence has no link; status words for all four statuses; a path with a space is percent-encoded.
6. **Escaping (adversarial):** a title containing `|`, a newline and the text `<!-- roadmap:summary:end -->` renders as one table row with no marker and no raw `<`; filling a block with it and running `fill_blocks` again on the result succeeds and gives the same text.
7. `fill_blocks`: replaces only the block content; text outside is unchanged; running it twice gives the same text (idempotent); an empty block is filled.
8. `fill_blocks` errors: missing begin, missing end, duplicate block, end before begin, unknown block name, nested markers, a marker that is not alone on its line.
9. **Committed files match `roadmap.json`:** for every file in `GENERATED_BLOCKS`, regenerating its blocks from the committed `roadmap.json` gives the committed text. This fails if any summary count in a committed file differs from `roadmap.json`.
10. **Adversarial:** changing one task's status in a copy of the roadmap changes every target file, so test 9 would catch a stale file.
11. **No stray status text:** outside the generated blocks, and after removing Markdown emphasis and code marks (`` ` ``, `*`), the target files contain none of these patterns (case-insensitive): `\d+ tasks? (remain|are) (not.started|complete)`, `tasks are complete`, `are all not started`, `all new tasks are pending`, `\d+ of \d+ tasks done`, `remains? incomplete`. A companion test proves the check (normalization plus pattern) flags each removed sentence verbatim, including ``53 tasks remain `not_started` `` and "**M0 remains incomplete: 3 of 5 tasks done.**". Adversarial: that exact RUST-ROADMAP sentence placed outside a block in a copy of a target makes the stale-text check fail, and placed inside a block it does not.
12. **No stale ordering:** the edited files contain none of: `before M1`, `precedes M1`, `starts after UI-12`, `R1-01 now requires UI-12`, `before domain implementation`, `UX --> M1`. A companion test proves each phrase from the old text matches.
13. Every file in `GENERATED_BLOCKS` exists and holds exactly the listed blocks, in order.
14. `--check` in process: on a temporary repository copy (roadmap, script, HTML, CSV and target files) with one stale block, `main(["--check"], root=tmp_plan)` returns 1, names the stale file and writes nothing (bytes and modification times of every file unchanged); on current files it returns 0.
15. **CLI exit code:** the same two cases run as a subprocess, `python -I <tmp>/plan-rust-diagram-editor/build_rust_roadmap.py --check` (the copied script finds its own `roadmap.json` through `__file__`), assert exit codes 1 and 0 and unchanged files; an unknown argument exits 2.
16. **All or nothing (adversarial):** in a temporary copy, make the first target stale and delete the end marker of the last target; normal regeneration exits 2, names the last file and the marker problem, and leaves every file (Markdown, HTML, CSV) byte-for-byte and modification-time unchanged.
17. **Line endings and second run:** in a temporary copy with one CRLF target and one LF target, both stale, normal regeneration fixes both, keeps each file's line ending and the text outside blocks; a second run changes no Markdown file and keeps every Markdown modification time. (`ROADMAP.html` and `rust-backlog.csv` keep today's behavior: they are rewritten on every normal run with identical content.)

## Gates

- From `plan-rust-diagram-editor/`: `python -m unittest test_build_rust_roadmap`, `python build_rust_roadmap.py` (no content diff in `ROADMAP.html` and `rust-backlog.csv`), `python build_rust_roadmap.py --check` exits 0.
- From `app/`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo doc --workspace --no-deps --locked` (no Rust change; run to prove nothing broke).
- From the repository root: `python .claude/skills/diagram-studio-maintainability/scripts/maintainability_check.py`: 0 FAIL, WARN not above 11.
- Real-window check: not applicable (no UI or Rust change).
- Codex: `/codex-review:plan` on this file and `/codex-review:code` on the full diff, at `xhigh`, read-only, up to 5 rounds each.

## Out of scope

Shared test helpers and the `shell.rs` split (slice 2, `refactor/p0-split-shell`); any Rust change; changing any task status in `roadmap.json`.

## Done when

ADR-003 is committed on `docs/p0-decisions`; the generated blocks are in place and the stale text is gone; validation, `--check` and the Python tests pass; Codex approved the plan and the code; all gates pass; the pull request to `main` is open after the user approves the push.
