# Agent A — New UI (look, layout, screens)

> **Merge note (2026-11-05):** only the model (`diagram-core`), the viewport and picking (`diagram-canvas`) and these documents were merged into `main`. The desktop work described here (contract, views, editor state, shortcuts) was **not** merged, because `main` already had its own reviewed version of those screens (PR #4). It stays on branch `ccr-9e509aa7-1fyp6k` for reference and will be ported onto main's design. See [the report, section 13](REPORT-2026-11-05.md#13-merging-into-main).

Date: 2026-11-05. Partner brief: [AGENT-B-FUNCTIONALITY.md](AGENT-B-FUNCTIONALITY.md). Both agents work at the same time. **The two briefs share sections 1–3 word for word.** If they ever differ, stop and ask the user. The ordered task list and the routing tests are in [TASKS-AND-TESTS.md](TASKS-AND-TESTS.md).

**Your job:** build every screen and control of the new design so it looks and behaves correctly *as a presentation*. You draw what you are given and report what the user did. You never decide what a button does to the project.

Read these before starting:
- [new-design/NEW-DESIGN-FUNCTIONAL-SPEC.md](../../app/docs/ui/new-design/NEW-DESIGN-FUNCTIONAL-SPEC.md) §2 (workspace), §6 (inspector) and §8 (focus).
- [FIGJAM-BOARD-REVIEW.md](../../app/docs/ui/new-design/FIGJAM-BOARD-REVIEW.md).
- [UI-RULES.md](../UI-RULES.md). This is mandatory.

---

## 1. Shared contract: the only seam between the agents

The UI and the functionality meet in **three types**. Agent B defines them in Phase 0 (section 3) and owns them. Agent A only reads them.

| Type (file, owner B) | Direction | What it is |
|---|---|---|
| `Presentation` (`src/presentation.rs`) | B → A | A read-only snapshot of everything the screen shows this frame. It has sub-views: `HeaderView`, `TabsView`, `ToolsView`, `NavigatorView`, `CanvasView`, `InspectorView`, `StatusView`, `Option<DialogView>`, `CommandSearchView`. Each command in it is a `CommandView { id, label, shortcut_text, availability, active }`. |
| `UiIntent` (`src/intent.rs`) | A → B | What the person did, for example `Command(CommandId)`, `Select(Target)`, `DraftChanged { field, text }`, `CommitDraft(field)`, `CancelDraft(field)`, `Pointer(CanvasPointer)`, `Search { scope, query }` and `FocusRegion(Region)`. Views **push** intents. They never mutate state. |
| `InputContext` (`src/intent.rs`) | A → B | Facts that only the UI layer knows each frame: `text_had_focus`, `ime_composing`, `focused_region` and `modal_open`. B uses it to decide shortcuts. |

Rules:
1. Every view function has the shape `fn show(ui: &mut egui::Ui, view: &XxxView, out: &mut Vec<UiIntent>)`. The only exceptions are pure local widget state, such as scroll position or the hover state of a drawer animation.
2. A needs a field, intent variant or command that doesn't exist yet. A writes it in **section 6 (Requests to Agent B)** of A's own brief and uses a local fixture meanwhile. A **never edits B's files**, and B never edits A's.
3. A changes to the contract is additive only: new fields and new enum variants. Renames or removals need both agents to agree, through a request entry.
4. Availability text such as "Not available yet: …" comes only from B. A displays it and never writes its own reason strings.

## 2. File ownership: one owner per file, no exceptions

| Area | Agent A (UI) owns | Agent B (functionality) owns |
|---|---|---|
| Desktop crate | `src/shell.rs` (layout composition only, after Phase 0), `src/ui/mod.rs`, `src/ui/theme.rs`, `components.rs`, `palette.rs`, `showcase.rs`, `workspace.rs`, `welcome.rs`; new `src/ui/header.rs`, `tabs.rs`, `toolbar.rs`, `navigator.rs`, `canvas_view.rs` (painting and hit regions only), `inspector.rs`, `status.rs`, `dialogs.rs`, `search_view.rs`, `fixtures.rs` | `src/lib.rs`, `src/main.rs`, `src/ui/commands.rs`; new `src/presentation.rs`, `src/intent.rs`, `src/app_state.rs`, `src/editor_state.rs`, `src/dispatch.rs`, `src/viewport.rs`, `src/drafts.rs` |
| Other crates | none | `diagram-core`, `diagram-canvas`, `diagram-storage` (all files) |
| Build | `assets/` (fonts and icons, with license notes) | `Cargo.toml` (all), `Cargo.lock` |
| Tests | `tests/theme.rs`, `typography.rs`, `showcase.rs`, `welcome.rs`, `workspace.rs`, `shell.rs`, `ui_audit.rs`; new `tests/views_*.rs`, `tests/ui_routing.rs`, `tests/common_ui/` | `tests/commands.rs`, `diagnostics.rs`; new `tests/routing.rs`, `editor_state.rs`, `presentation.rs`, `shortcuts.rs`, `contract.rs`, `journeys.rs`, `tests/common_state/`; all tests in the other crates |
| Docs | `app/docs/ui/*.md` and screenshots, except `new-design/` | new `app/docs/functionality/*.md` and `plan-rust-diagram-editor/PROGRESS-AND-NEXT-STEPS.md` |
| Tracker | `roadmap.json` entries for UI-02, UI-05, UI-07, UI-08, UI-09, UI-10, UI-12 and UI-18, plus R0-04 | `roadmap.json` entries for UI-06, UI-11, UI-13 to UI-17, and R1-xx/R2-xx when allowed |
| Read-only for both | `app/docs/ui/new-design/*`, `UI-RULES.md`, `UI-DESIGN-AND-INTEGRATION.md`, [TASKS-AND-TESTS.md](TASKS-AND-TESTS.md), sections 1–6 of the other agent's brief | same |

Only one place is shared: **`roadmap.json`**. Edit only your own task objects, rebase immediately before pushing, and run `python build_rust_roadmap.py`.

## 3. Order, branches and merging

| Step | Who | What | Blocks |
|---|---|---|---|
| **Phase 0** (first, small) | B | Move `ShellState`, `run()` and `command_context()` out of `shell.rs` into `app_state.rs` and `dispatch.rs`. Add `presentation.rs` and `intent.rs` with the types in section 1, built from today's sample state. Make `shell.rs` call `presentation::build(&state)` then `dispatch::apply(&mut state, intents)`. There is **no visible change**, and all existing tests and the audit must still pass. | A may not edit `shell.rs` until Phase 0 is merged. A may start on every *new* A file at once, using `fixtures.rs`. |
| Parallel work | A and B | Each works on its own branch: `agent-a/new-ui` and `agent-b/functionality`. Rebase on `main` daily. | — |
| Merge order | — | B's contract additions merge **before** A's views that use them. If A is ahead, A keeps the fixture, then swaps to the real field after B merges. | — |
| Integration | A | When B adds a working feature, A removes the matching fixture and adds the state to `ui_audit.rs`. | — |

**Decisions confirmed by the user on 2026-11-05:**
- **D1, yes:** command search keeps Ctrl+K and also gets Ctrl+Shift+P.
- **D2, yes:** the **Note** tool is removed from v0.1 (`AddNote` leaves the catalog and the tool row).
- **D3, yes:** **Pan** is renamed **Hand** (shortcut H, plus Space+drag for temporary panning).
- **Model gate relaxed, yes:** B starts the real model (B-12+, R1 then R2) now, in parallel with A's screens. R1-01 no longer waits for R0-05 or UI-12. If B's features are ready before A's new screens, **B tests them in the UI that exists at that time**: the existing shell already draws every catalog command, so enabling a command makes its current button work. B proves this with `journeys.rs` and the audit, never by editing A's files. A later moves the same commands onto the new screens.

Agent A draws whatever the catalog contains, so these decisions never block A.

---

## 4. What Agent A builds

Every item uses only `theme.rs` tokens and text roles, and `components.rs` controls. Every item works at 800×500, 1280×800 and 1440×900 in light and dark themes. Every item gets a headless test and an entry in `ui_audit.rs`. Every item has fixture states for empty, no-match, disabled-with-reason, error and loading wherever the spec lists them.

### A1 — Shell layout to the new wireframe (spec §2)
- Compose the shell as header, tabs, tool row, board, status, navigator and inspector, with the validation drawer under the board. Use the existing `workspace.rs` sizes. The row heights (48, 38, 40, 30) and panel widths (240 and 300) already match the spec.
- Keep the 800×500 drawer behavior from UI-03. Add **toolbar overflow** ("More tools ▾") when the tool row doesn't fit, so tools never clip.
- Add the region-focus outline that `InputContext.focused_region` reports, and draw the visible focus ring per region for F6 cycling. B decides the order and the key handling.

### A2 — Header (`header.rs`)
- The header shows: Project menu, project name, a dirty or save-state badge in words ("Unsaved", "Saving…", "Saved", "Save failed"), Undo and Redo, Search commands, Save and Export.
- Undo and Redo show B's pending action label, for example "Undo Rename class".
- The Project menu lists `CommandView`s grouped by `CommandGroup`, with shortcut text right-aligned.

### A3 — Diagram tabs (`tabs.rs`)
- Each tab has a title and a close button with an accessible name like "Close tab Class overview". There is a "+ Diagram" button.
- Overflowing tabs scroll and have a "More tabs ▾" list. The active tab is shown by more than color.

### A4 — Tool row (`toolbar.rs`)
- Labeled tool buttons come from `ToolsView`, with the active tool drawn as pressed (outline plus fill).
- The relationship-kind chooser ("Association ▾" or "Generalization ▾") sits next to the Relationship tool.
- Arrange is a menu button. The project-panel and inspector toggles sit at the row's ends.
- There is **one tool row only**. The bottom-dock variant B from the FigJam review is a fixture screen in `showcase.rs` for the comparison experiment, and it is never shown alongside the top row.

### A5 — Navigator (`navigator.rs`, UI-05)
- **Diagrams** and **Model** tabs, each with a labeled search field ("Find object…", Ctrl+F shown).
- Result rows show the name plus type and location, so duplicate names can be told apart. Long names are truncated with the full name kept for screen readers.
- The list has distinct empty, no-results and loading states.
- Row action menu: Reveal, Show in diagram, Duplicate model, Rename and Delete model everywhere. Destructive items are visually separated, and every item is a `CommandView`.

### A6 — Board drawing (`canvas_view.rs`, UI-07)
- Paint `CanvasView` items: class boxes with name, attribute and operation compartments; association and generalization edges with markers; selection outlines and handles; the marquee rectangle; placement preview; connect-source and connect-target highlights; valid and invalid target feedback **with text**; grid; and the empty-diagram prompt ("Add Class" with a hint).
- Publish `widget_info` for every node and edge, with a role and name. Report pointer input as `UiIntent::Pointer`, carrying world or screen position as the contract says. **Hit testing and transforms are B's** (`viewport.rs` and `diagram-canvas`). A gets screen rectangles in `CanvasView` and only draws them.
- Replace the hard-coded `CLASS_WIDTH`, `DOCUMENT_FILL` and `DOCUMENT_INK` in `shell.rs` with document-style values supplied in `CanvasView`. Document colors are never chrome theme tokens.

### A7 — Inspector (`inspector.rs`, UI-08)
- One layout per selection kind, per spec §6:
  - **None:** diagram settings.
  - **Class:** Model / Members / Appearance.
  - **Relationship:** ends, roles and multiplicity.
  - **Multiple selection:** count, common fields with "Mixed" values, and Align and Distribute.
  - **Mixed nodes and edges.**
- Every section has a persistent scope label: "Shared model — affects all views" or "This diagram view only".
- Member rows have Up, Down and Remove buttons, each with an accessible name such as "Move attribute id up". The expanded member editor has Apply and Cancel.
- Draft display: the field shows B's draft text, and an "Unsaved edit" marker or an error appears beside the field. Focus **stays in the field** when its error appears (UI rule 1.4).
- Numeric geometry fields for X, Y, Width and Height.

### A8 — Status bar (`status.rs`)
- Save state and path, tool hint or selection count, and an issues count button that opens the drawer.
- Grid and Snap toggles, zoom −, the zoom percent, zoom +, 100%, Fit, and the theme chooser.
- Every element is words and color, never color alone.

### A9 — Dialogs and drawers (`dialogs.rs`, UI-09)
- Unsaved-changes dialog: Save, Discard and Cancel, with Cancel as the default focus.
- Delete-model-everywhere impact review, listing the counts B supplies.
- Resolve edit: Return to field or Discard edit.
- Save failed: Retry and Save As.
- Open failed, Recovery (Restore or Ignore), and New diagram (type and name).
- Validation drawer: severity in words, message, location and **Go to object**, plus a stale or checking marker.
- Every modal traps Tab, closes on Escape, and returns focus to its opener.

### A10 — Command search and object search (`search_view.rs`)
- Two visibly separate surfaces, "Search commands…" and "Find object…", per spec §4.2 and research recommendation 2.
- Command results show the label, group and shortcut text, and unavailable commands show the reason **as visible text**.

### A11 — Typography and fonts (finish UI-02)
- Add reviewed Noto Sans and Noto Sans Arabic files under `assets/fonts/` with their OFL license text, and register them in `theme.rs`. If loading them needs a new crate, file a request to B, who owns `Cargo.toml`.
- Arabic and bidi correctness stays in R0-04 and UI-10. Claim only what the Windows checks prove.

### A12 — Keyboard, accessibility and review (UI-10 and UI-12)
- Extend `ui_audit.rs` to every new screen state and fixture.
- Add audit checks for:
  - visible focus inside each region;
  - no control hidden behind an open drawer;
  - tool-row overflow keeping every tool reachable.
- Take screenshots of each screen at the three sizes in both themes under `app/docs/ui/`, and write an evidence page per task.

## 5. Definition of done (Agent A)
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --locked` and `cargo doc --workspace --no-deps --locked` all pass from `app/`.
- No raw colors or font sizes appear outside `theme.rs`. No shortcut or reason text is written by A.
- Every A view takes only `&…View` plus `&mut Vec<UiIntent>`. A `grep` for `&mut ShellState` or `&mut AppState` in A's files finds nothing.
- Every bug fix has a test proven to fail without the fix.
- Task status is `done` only when its acceptance text and UI-RULES hold, and the evidence page plus screenshots exist.

## 6. Requests to Agent B (A appends here; B reads and answers in its own brief, section 6)

| # | Date | Need (field, intent or command) | Why | Status |
|---|---|---|---|---|
| 0 (from B) | 2026-11-05 | B asked A to stop hard-coding "Pan" and "Note" in `tests/shell.rs` and `tests/typography.rs` before D2/D3 land. | Catalog is the single source of truth. | Done in commit `A-req-B:`: labels and the disabled set now come from `commands::placed` and `commands::availability`; nothing asserts Hand is disabled. |
| 0b (from B) | 2026-11-05 | (a) With D2/D3 applied, `command_search_should_pass_the_audit` failed at 1440×900: rows clipped by the search list's scroll area kept their full rectangles in the accessibility tree. (b) `canvas_view` sent `Down` where egui recognized the drag (about 15 points past the press), so every Hand pan lost its first step. | Real defects in A's files. | Done in `A-req-B2:`. (a) Fixed by the alternative B named: the list renders only visible rows with `ScrollArea::show_rows` (already in commit ee9c92f, found by the same audit while testing Noto fonts), so no clipped row exists to report; the audit itself is unchanged and its self-test still proves a real off-window control fails. Clipping rects to scroll viewports in the audit was not done: AccessKit gives the audit no scroll-viewport bounds to clip against. (b) `Down` now uses `pointer.press_origin()`; regression test `board_drag_should_report_down_at_the_press_position` failed without the fix (Down at 720,510 instead of 700,500) and passes with it. |
| 1 | 2026-11-05 | `HeaderView`: save state as its own field (words plus a tone such as Unsaved / Saving… / Saved / Save failed) instead of inside `title`; Undo/Redo labels with the pending action ("Undo Rename class"). | §A2 save badge in words; history labels. | open |
| 2 | 2026-11-05 | `TabView`: a diagram identity; intents to activate a tab and to close one (`CloseTab(id)`, never deleting the diagram). | §A3 tab switching and "Close tab Class overview" buttons. The view draws tabs but cannot switch or close them yet. | open |
| 3 | 2026-11-05 | `ToolsView`: the relationship-kind chooser (current kind plus options) and the Arrange menu (Align ×6, Distribute ×2) as `CommandView`s. | §A4 kind chooser and Arrange menu button. | open |
| 4 | 2026-11-05 | `NavigatorView`: Diagrams/Model tab state and an intent to switch it; the object search field (query, shortcut text) and results with type and location; list states (loading, empty, no match, error); per-row actions as `CommandView`s with an intent that names the row's target (e.g. `CommandOn { command, target }`). | §A5. Today's rows have only a label. | open |
| 5 | 2026-11-05 | `CanvasView`: document style (fill/ink), edges with kind and route points, marquee, placement preview, connect source/target feedback with text, zoom; `Target` variants for edges, models and diagrams. | §A6. The view paints nodes and empty text only; it uses `theme::DOCUMENT_DEFAULT_*` until a document style arrives. | open |
| 6 | 2026-11-05 | `InspectorKind`: Relationship, Multiple (count, Mixed values, align/distribute), Mixed; member rows with Up/Down/Remove/Edit `CommandView`s; geometry fields; more `DraftField` variants; problem severity (error vs warning) on `FieldView`. | §A7. Only Nothing and Class exist. | open |
| 7 | 2026-11-05 | `StatusView`: Grid, Snap, Zoom −/+, 100%, Fit and Issues as `CommandView`s, the zoom percent text and the file path. | §A8. | open |
| 8 | 2026-11-05 | `DialogView` variants for Unsaved changes, Delete everywhere (impact counts), Resolve edit, Save failed, Open failed, Recovery, New diagram; a dialog-choice intent; a validation drawer view (severity, message, location, Go to object, freshness). | §A9 / A-8. No dialog types exist yet, so A-8 has not started. | open |
| 9 | 2026-11-05 | `CommandSearchView`: the result rows (label, group, shortcut text, availability) and the highlighted row, plus separate object-search results. | §A10 / A-9: the search modal still lives in `shell.rs` because its results are computed there. | open |
| 10 | 2026-11-05 | Update `journeys.rs` `escape_after_typing_a_valid_name_should_keep_it` (and any test assuming live commit) to the spec §6 draft policy, so A can set `ShellSupport::draft_events` and `resolve_dialog`. With `draft_events` on, it is the only failing test in the suite; A's own tests already commit with Enter. | J2–J4 need drafts on; A must not edit B's tests. | open |
| 11 | 2026-11-05 | Grid, Snap, zoom −/+, 100% and Fit as `CommandView`s in `StatusView` (today `grid`/`snap` are bools and the commands are menu-only). Also the pan offset in `CanvasView`, so the grid can follow panning. | §A8 status controls; the grid is drawn from the board origin. | open |

## 7. Task progress (Agent A only)

Task IDs come from [TASKS-AND-TESTS.md §3](TASKS-AND-TESTS.md#3-task-list). Add a row when a task starts. Set its state to `done` only when its listed tests pass and the joint checkpoint it feeds, if any, still holds.

| Task | State | Branch / commit | Notes |
|---|---|---|---|
| A-0 | done | `agent-a/new-ui` | `ui/fixtures.rs` (private contract copy until B-0, fixture `Presentation` per screen state), `tests/common_ui/` (harness at 3 sizes × 2 themes, intent recorder, find-by-name, shared audit), `ui_routing.rs` skeleton looping over every command. After merging B-0 the private copy was deleted; fixtures are now built by `presentation::build`. |
| A-1 | in progress | `agent-a/new-ui` | `ui/header.rs` on B's `HeaderView`: Project menu (submenus per group, shortcut text, disabled reasons as visible text), truncating title, header commands with group separators. `views_header.rs`. Remaining: save badge and history labels (request 1). |
| A-2 | in progress | `agent-a/new-ui` | `ui/tabs.rs`: tabs pressed + underlined when active, hidden-scroll strip with a "More tabs" menu that reveals and focuses a tab. `views_tabs.rs`. Remaining: switching and close buttons (request 2). |
| A-3 | in progress | `agent-a/new-ui` | `ui/toolbar.rs`: toggles at the ends, tools pressed when active, "More tools" overflow keeps every tool reachable. `views_toolbar.rs`. Remaining: kind chooser and Arrange menu (request 3). |
| A-4 | in progress | `agent-a/new-ui` | `ui/navigator.rs`: Diagrams and Model sections, truncated rows with full names, empty states, Select/Shift toggle. `views_navigator.rs`. Remaining: tabs, Find object, row actions, list states (request 4). |
| A-5 | in progress | `agent-a/new-ui` | `ui/canvas_view.rs`: nodes as accessible buttons at the presented rectangles, selection handles, empty text, Select on click, Pointer Down/Move/Up/Zoom with hit. Document colors moved to `theme::DOCUMENT_DEFAULT_*`. `views_canvas.rs`. Remaining: edges, marquee, previews, feedback (request 5). |
| A-6 | in progress | `agent-a/new-ui` | `ui/inspector.rs`: guidance, scope label, Name draft field (DraftChanged / CommitDraft / CancelDraft, focus kept when its problem appears). Fixed: CommitDraft was emitted twice per Enter (regression test in `ui_routing.rs`, failed before the fix). `views_inspector.rs`. Remaining: other selection kinds and members (request 6). |
| A-7 | in progress | `agent-a/new-ui` | `ui/status.rs`: save state and selection in words, theme chooser. `views_status.rs`. Remaining: Grid/Snap/zoom/Fit/Issues (request 7). |
| A-11 | in progress | `agent-a/new-ui` (ee9c92f) | Noto Sans and Noto Sans Arabic Regular (static, unhinted, OFL 1.1, licenses and SHA-256 in `assets/fonts/`), registered in `theme::fonts()` as the first fallbacks after egui's primary font; no new crate needed. Tests: registration order, Arabic and Latin Extended glyph coverage. Remaining: making Noto Sans the primary face (its ~3-point taller line pushes the welcome screen past 800×500, so the entry layout needs rework first). Arabic shaping/bidi is not claimed (R0-04). |
| A-13 | in progress | `agent-a/new-ui` | Swapped onto B's B-4..B-10 fields: ActivateTab, CanvasResized, TemporaryPan, grid, marquee, zoom/grid/snap words, tool hint, object search, Resolve edit; `ShellSupport { grid, object_search }` set. |
| A-12 | in progress | `agent-a/new-ui` | Audit extended (every view × 3 sizes × 2 themes, Project submenus, workspace states, object search, overflow reachability for tools and tabs). Evidence: `app/docs/ui/A-VIEWS-EVIDENCE.md`. Remaining: focus-ring-per-region check (needs region focus), drawer-cover check, scroll-aware audit, real-window screenshots (not possible here: libxkbcommon-x11 missing). |
| A-req-B2 | done | `agent-a/new-ui` | Command search rows virtualized (show_rows); board `Down` reported at the press origin, with a regression test. |
| A-8 | in progress | `agent-a/new-ui` | `ui/dialogs.rs` Resolve edit (default focus, Escape returns to field, Tab trapped) with `views_dialogs.rs`; drawn by the shell, `resolve_dialog` flag off until request 10. Other dialogs and the validation drawer blocked on request 8. |
| A-9 | in progress | `agent-a/new-ui` | `ui/search_view.rs` object search on `Presentation::object_search` (`views_search.rs`, audited in the shell at 1280×800/1440×900). Command search still in `shell.rs`, now virtualized with `show_rows`; moving it onto `CommandSearchView::results` remains. |
| A-10 | in progress | `agent-a/new-ui` | `shell.rs` composes only: header, tabs, tool row, navigator, board, inspector and status are drawn by the view modules; local drawing code and hard-coded document colors removed. The command search modal and drawer frame stay in `shell.rs` until requests 8–9. |
