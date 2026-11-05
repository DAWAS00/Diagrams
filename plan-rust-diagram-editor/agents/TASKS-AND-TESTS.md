# Tasks and tests for Agent A (UI) and Agent B (functionality)

> **Merge note (2026-11-05):** only the model (`diagram-core`), the viewport and picking (`diagram-canvas`) and these documents were merged into `main`. The desktop work described here (contract, views, editor state, shortcuts) was **not** merged, because `main` already had its own reviewed version of those screens (PR #4). It stays on branch `ccr-9e509aa7-1fyp6k` for reference and will be ported onto main's design. See [the report, section 13](REPORT-2026-11-05.md#13-merging-into-main).

Date: 2026-11-05. Companion to [AGENT-A-UI.md](AGENT-A-UI.md) and [AGENT-B-FUNCTIONALITY.md](AGENT-B-FUNCTIONALITY.md). This file is **read-only for both agents**. Each agent records progress only in section 7 of its own brief, and records task status in its own `roadmap.json` entries. If this file is wrong, ask the user to change it.

## 1. The routing chain both agents must keep working

Every user action travels the same path. Each hop has exactly one owner and one test file.

```text
 [A] control drawn from Presentation ──(click / Enter / shortcut / search / pointer)──┐
                                                                                    v
 [A] view pushes UiIntent ─────────────> [B] commands::take_shortcut (keys only, uses InputContext)
                                                                                    v
                                         [B] dispatch::apply(&mut AppState, UiIntent)
                                                                                    v
                                         [B] AppState / EditorState / (later) Document
                                                                                    v
 [A] view draws it <────────────────── [B] presentation::build(&AppState) -> Presentation
```

| Hop | Owner | Proved by |
|---|---|---|
| H1: A control exists for every `CommandView` in its placement, with the same label, enabled state and reason | A | `tests/ui_routing.rs` |
| H2: Clicking or activating that control emits exactly the expected `UiIntent`; a disabled control emits nothing | A | `tests/ui_routing.rs` |
| H3: A key press with a given `InputContext` produces the right `CommandId`, or none | B | `tests/shortcuts.rs` |
| H4: `dispatch::apply` changes state correctly; rejected or disabled intents change nothing | B | `tests/routing.rs`, `tests/editor_state.rs` |
| H5: `presentation::build` reflects the new state (labels, availability, active tool, selection, drafts, focus region) | B | `tests/presentation.rs` |
| H6: The whole app, from real click to real screen, works for the full user journeys | B, with A reviewing | `tests/journeys.rs` |
| H7: The layout and accessibility audit still passes for every state reached in H6 | A | `tests/ui_audit.rs` |

**The single source of truth for routing is the catalog** (`CommandId::ALL` in `commands.rs`). The routing tests **loop over the catalog**; they don't keep their own list. So a new command is tested automatically on both sides, and no shared table needs editing. Non-command intents (pointer, select, draft, search, focus) are listed in section 4, and both agents' tests cover all of them.

## 2. Test files: ownership and what each must prove

All test files are under `app/crates/diagram-desktop/tests/` unless a different crate is named. Section 2 of each brief already assigns the files that exist today. **The files below are new, and their owner is the one listed here.**

| File | Owner | Must prove |
|---|---|---|
| `tests/common_ui/mod.rs` | A | Shared harness helpers for A: build a harness from a `Presentation` fixture at 800×500, 1280×800 or 1440×900 in either theme; an **intent recorder** (collects the `Vec<UiIntent>` a frame produced); and find-by-accessible-name helpers. |
| `tests/common_state/mod.rs` | B | Shared helpers for B: a sample `AppState`, `apply_all(&mut state, [intents])`, an `InputContext` builder (typing, IME composing, canvas focused, modal open), and a key-press event builder. |
| `tests/ui_routing.rs` | A | **H1 and H2**, looping over every `CommandId`: each placed command renders a control with an identical label and an accessible name; disabled controls show B's reason text visibly or in hover; click and Enter each emit `Command(id)` exactly once; disabled controls emit nothing. Also covers every non-command intent in section 4 (row click → `Select`, field typing → `DraftChanged`, Enter → `CommitDraft`, Escape → `CancelDraft`, canvas drag → `Pointer` sequence, search typing → `Search { scope }`). |
| `tests/views_*.rs` (one per view) | A | Each view in each fixture state (empty, no-match, loading, disabled, error, active, selected, Mixed) shows the expected text, keeps focus on an errored field, and fits at all three sizes. |
| `tests/shortcuts.rs` | B | **H3**: every catalog shortcut maps to its command; Ctrl+Shift+Z beats Ctrl+Z; V/H/C/R/G fire only with canvas focus, no text focus, no IME composition and no modal; text fields keep Ctrl+A, Ctrl+F, Ctrl+Z and Escape; each event is consumed once (Ctrl+S never also triggers S). |
| `tests/routing.rs` | B | **H4 equivalence**, looping over every `CommandId`: the button intent, menu intent, shortcut and command-search activation all leave **the same `AppState`**; a disabled command leaves state byte-for-byte unchanged (compare with `PartialEq` or a debug snapshot); availability in `commands::availability` equals availability in `Presentation`. |
| `tests/editor_state.rs` | B | Tools and gestures: switching tools cancels a preview; Escape peels one layer per press; Hand drag never moves a node. Viewport: zoom clamps to 10–400%, round-trip transform at several zooms, Fit never moves geometry. Selection: click, Shift-click, marquee, Ctrl+A. Drafts: commit valid, keep invalid, Resolve edit on selection change, cancel restores. F6 order skips hidden regions. Preferences never set dirty. |
| `tests/presentation.rs` | B | **H5**: after each intent in `routing.rs`, the snapshot fields A depends on are right (active tool, pressed state, selection count, inspector kind, draft text and error, focus region, search results per scope, status texts). Phase 0 check: `build(sample)` reproduces today's labels and availability exactly. |
| `tests/contract.rs` | B | The contract stays complete: every `CommandId` has a non-empty label, group and placement; every disabled reason starts with a reason word, not empty text; every intent variant in section 4 is handled by `dispatch::apply` (no `todo!`, no unhandled `_` arm that silently does nothing); **every row of `UI-FUNCTION-MAP.csv` names a catalog command**, enabled or disabled with a reason. |
| `tests/journeys.rs` | B (A reviews and may request steps) | **H6**: drives the real `ShellApp` with egui_kittest through the journeys in section 5, using accessible names only. It asserts what a person sees, never internal fields. |
| `diagram-canvas/tests/transform.rs`, `picking.rs` | B | Pure geometry: inverse transform within tolerance at 10%, 100% and 400%; hit tolerance in screen units; stacking order; NaN and inf rejected. |
| `diagram-core/tests/*` (B4 only) | B | Typed IDs, validation, cycle rejection, atomic commands, undo and redo restoring complete state. |
| `tests/ui_audit.rs` (existing) | A | **H7**: add every new screen state and every journey checkpoint state. |

### Rules for every test, from UI-RULES §2
1. Find controls by **accessible name**, never by position or internal ID. If a test can't find a control by name, that is a bug in the control.
2. A bug fix comes with a test that fails without the fix, and the commit message says how this was proved.
3. Neither agent deletes, ignores or weakens a test owned by the other agent. If the other agent's test fails because of your change, fix your change or file a request.
4. Both agents run the **whole** suite before every push, not just their own files.

## 3. Task list

The ID prefixes are **A-** (UI agent), **B-** (functionality agent) and **I-** (joint integration checkpoint). "Needs" lists blocking tasks; tasks without a "needs" entry can start immediately. The roadmap column shows which tracker task receives the evidence.

### Agent B (functionality)

| ID | Task | Needs | Roadmap | Tests that must pass |
|---|---|---|---|---|
| B-0 | Phase 0: move `ShellState`, `run()` and `command_context()` into `app_state.rs` and `dispatch.rs`; add `presentation.rs` and `intent.rs` (`Presentation`, `UiIntent`, `InputContext`); `shell.rs` calls `build` then `apply`. No visible change. | — | UI-06 | All existing tests and the audit, plus new `presentation.rs` and `contract.rs` |
| B-1 | Create `common_state` helpers and the `routing.rs` skeleton looping over the catalog | B-0 | UI-06 | `routing.rs` |
| B-2 | Apply decisions D1–D3 and add every missing command from AGENT-B §B1 with reasons and aliases | B-0 (D1–D3 confirmed: yes) | UI-06 | `contract.rs`, `routing.rs`, `commands.rs` |
| B-3 | Input dispatch priority and `InputContext` handling (IME, text, modal, region) | B-0 | UI-06 / UI-10 | `shortcuts.rs` |
| B-4 | Tool and gesture state machine: Select, Hand, Space+drag, plus placement and connect previews that cancel cleanly | B-0 | UI-07 | `editor_state.rs` |
| B-5 | Viewport (transform, zoom, Fit) and picking in `diagram-canvas`, with each tab keeping its own viewport | B-4 | UI-07 | `transform.rs`, `picking.rs`, `editor_state.rs` |
| B-6 | Selection: click, Shift-click, marquee, Ctrl+A, select from the navigator | B-5 | UI-07 / UI-05 | `editor_state.rs`, `presentation.rs` |
| B-7 | Draft policy for rename (commit, cancel, blur, Resolve edit) | B-0 | UI-08 | `editor_state.rs` |
| B-8 | Escape layering and F6 region order | B-3 | UI-10 | `editor_state.rs`, `shortcuts.rs` |
| B-9 | Split command search and object search; tab switching for fixture diagrams; preferences (theme, grid, snap) | B-2 | UI-05 / UI-06 | `presentation.rs`, `routing.rs` |
| B-10 | Notation contract (Class only) feeding the tool row and the Add diagram picker | B-2 | UI-11 | `contract.rs` |
| B-11 | Write `journeys.rs` journeys J1–J4 (section 5) | B-6, B-7, B-8, A-3 | UI-17 (pre-model part) | `journeys.rs`, `ui_audit.rs` |
| B-12+ | Model work: R1-01 … R1-08, then R2, then switch each disabled command on; journeys J5–J7 | B-0 (gate relaxed by the user on 2026-11-05; may run alongside B-1 to B-11) | R1/R2, UI-13 to UI-17 | `diagram-core/tests`, `routing.rs`, `journeys.rs` |

### Agent A (UI)

| ID | Task | Needs | Roadmap | Tests that must pass |
|---|---|---|---|---|
| A-0 | Create `fixtures.rs` (a `Presentation` value per screen state) and the `common_ui` helpers, including the intent recorder | contract types from B-0; until then, a local copy kept private to `fixtures.rs` | — | compiles; `ui_routing.rs` skeleton |
| A-1 | Header view (§A2) | A-0 | UI-09 | `views_header.rs`, `ui_routing.rs` |
| A-2 | Tabs view with overflow (§A3) | A-0 | UI-05 | `views_tabs.rs`, `ui_routing.rs` |
| A-3 | Tool row with kind chooser and overflow (§A4) | A-0 | UI-07 | `views_toolbar.rs`, `ui_routing.rs` |
| A-4 | Navigator: Diagrams and Model tabs, find, row actions (§A5) | A-0 | UI-05 | `views_navigator.rs`, `ui_routing.rs` |
| A-5 | Board drawing: all canvas states, accessible nodes and edges, pointer intents (§A6) | A-0 | UI-07 | `views_canvas.rs`, `ui_routing.rs` |
| A-6 | Inspector layouts for each selection kind, scope labels, drafts and errors, member rows (§A7) | A-0 | UI-08 | `views_inspector.rs`, `ui_routing.rs` |
| A-7 | Status bar (§A8) | A-0 | UI-09 | `views_status.rs`, `ui_routing.rs` |
| A-8 | Dialogs and the validation drawer, with focus trap and return (§A9) | A-0 | UI-09 | `views_dialogs.rs`, `ui_routing.rs` |
| A-9 | Separate command search and object search views (§A10) | A-0 | UI-06 (view part) | `views_search.rs`, `ui_routing.rs` |
| A-10 | Recompose `shell.rs` to the new wireframe, using views only (§A1) | **B-0 merged**, A-1 to A-9 | UI-03 / UI-12 | `shell.rs`, `workspace.rs`, `ui_audit.rs` |
| A-11 | Fonts and type (finish UI-02, §A11) | — (file a request to B if a crate is needed) | UI-02 | `typography.rs` |
| A-12 | Audit extensions (focus ring per region, nothing hidden under drawers, overflow reachable); screenshots and evidence pages | A-10 | UI-10 / UI-12 | `ui_audit.rs` |
| A-13 | Swap fixtures for real fields as B lands them (one row in AGENT-B §6 each) | the matching B task | — | `ui_routing.rs`, `journeys.rs` |

### Joint checkpoints (both agents green before moving on)

| ID | When | Pass condition |
|---|---|---|
| I-1 | After B-0 and A-0 | A's fixtures compile against B's real types. The app looks exactly as before (screenshot compared at 1280×800). The whole suite passes. |
| I-2 | After B-2 to B-4 and A-1, A-3, A-10 | `ui_routing.rs` and `routing.rs` both pass for **every** catalog command, so the routing chain H1–H5 is proved for every command. |
| I-3 | After B-11 | Journeys J1–J4 pass. The audit passes at every journey checkpoint. Real-window screenshots at 800×500 and 1280×800 in both themes are saved. Then UI-12 review can begin. |
| I-4 | After each model feature (B-12+) | The matching command switches from disabled to working **in whatever UI exists at that time** (the current shell if A's screens aren't merged yet); `routing.rs`, `presentation.rs`, `journeys.rs` and the audit all pass; A has removed the related fixture. |

## 4. Non-command intents: what each control must emit (H2 and H4)

| User action | Control (A) | `UiIntent` (A emits) | Result (B, tested in `routing.rs` or `editor_state.rs`) |
|---|---|---|---|
| Click a navigator row or canvas node | row or node | `Select(Target::View(id))` (Shift adds `toggle: true`) | selection updated; inspector kind changes |
| Click empty canvas | board | `Pointer(Down/Up at empty)` | selection cleared, unless a draft is unresolved (then Resolve edit) |
| Drag on canvas | board | `Pointer(Down, Move…, Up)` with screen position and modifiers | Select: marquee or move preview; Hand: pan only |
| Ctrl+wheel | board | `Pointer(Zoom { delta, anchor })` | zoom anchored and clamped |
| Type in a field | inspector or dialog field | `DraftChanged { field, text }` | draft text updated, validation message updated |
| Enter in a field | field | `CommitDraft(field)` | committed if valid; otherwise the error stays and focus stays |
| Escape in a field | field | `CancelDraft(field)` | original value restored; selection unchanged |
| Type in a search box | command or object search | `Search { scope, query }` | results for that scope only |
| Activate a search result | result row | `Command(id)` or `Select(target)` | same result as the direct control |
| F6 or click into a region | any region | `FocusRegion(region)` | region focus updated; hidden regions skipped |
| Open or close a drawer or dialog | toggle or close button | `Command(ToggleNavigator / ToggleInspector / …)` or `CloseDialog` | drawer or dialog state updated; focus return target recorded |

Any new intent variant must be added to this table by the user, and then to both `ui_routing.rs` (A) and `routing.rs` (B). `contract.rs` fails if `dispatch::apply` has an unhandled variant.

## 5. Journeys for `tests/journeys.rs`

These are driven by accessible names, at 1280×800 and again at 800×500.

| ID | Journey | Checks |
|---|---|---|
| J1 | Welcome → sample project → choose Hand → drag canvas → Fit all → zoom in, then press 100% | Nodes didn't move; the zoom text in the status bar updated; the project is not marked dirty |
| J2 | Select Customer from the navigator → rename to "Client" → Escape → rename again → Enter | The name was restored after Escape, then shows "Client" in the navigator, canvas and inspector; focus stayed in the field while it was in error |
| J3 | Keyboard only: F6 to the board → V, H, C → Escape until nothing is active → Ctrl+K "fit" → Enter → Ctrl+F "Order" → Enter | Each Escape peels one layer; letters never fire while a search field is focused; focus returns to the opener after the search closes |
| J4 | Narrow window: open the project-panel drawer → select a class → open the inspector drawer → Escape twice | The drawer closes, then the selection clears; focus returns to the toggle buttons; the audit passes at each step |
| J5 (B4) | Add Class → name it → add attribute `id: String` → Connect Customer to Order → set multiplicities → Undo three times → Redo | The history labels on Undo and Redo match; Undo restores state exactly |
| J6 (B4) | Save (forced failure) → Retry → Save As → close the project with unsaved changes → Cancel | "Save failed" persists until resolved; nothing says "Saved" falsely; Cancel keeps the project open |
| J7 (B4) | Show Customer in a second diagram → rename it → remove the view in diagram 2 → Delete model everywhere → review the impact → Undo | The rename shows in both diagrams; removing the view leaves the model; the impact counts are right; Undo restores everything |

## 6. Before every push (both agents)

From `app/`:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked          # the WHOLE suite, both agents' tests
cargo doc --workspace --no-deps --locked
```

If a `roadmap.json` status changed, also run `python build_rust_roadmap.py` in `plan-rust-diagram-editor/`. Rebase on `main` immediately before pushing. If the other agent's test now fails, don't push: fix your change or file a request.
