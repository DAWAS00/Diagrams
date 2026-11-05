# Agent B — Functionality (what every button does)

> **Merge note (2026-11-05):** only the model (`diagram-core`), the viewport and picking (`diagram-canvas`) and these documents were merged into `main`. The desktop work described here (contract, views, editor state, shortcuts) was **not** merged, because `main` already had its own reviewed version of those screens (PR #4). It stays on branch `ccr-9e509aa7-1fyp6k` for reference and will be ported onto main's design. See [the report, section 13](REPORT-2026-11-05.md#13-merging-into-main).

Date: 2026-11-05. Partner brief: [AGENT-A-UI.md](AGENT-A-UI.md). Both agents work at the same time. **The two briefs share sections 1–3 word for word.** If they ever differ, stop and ask the user. The ordered task list and the routing tests are in [TASKS-AND-TESTS.md](TASKS-AND-TESTS.md).

**Your job:** make every command, button, shortcut and gesture do the right thing, or explain truthfully why it can't yet. You own state, rules and the contract types. You never draw or lay out controls.

Read these before starting:
- [new-design/NEW-DESIGN-FUNCTIONAL-SPEC.md](../../app/docs/ui/new-design/NEW-DESIGN-FUNCTIONAL-SPEC.md) §3–§9.
- [UI-FUNCTION-MAP.csv](../../app/docs/ui/new-design/UI-FUNCTION-MAP.csv).
- [RUST-ENGINEERING.md](../RUST-ENGINEERING.md) and [02-ARCHITECTURE.md](../02-ARCHITECTURE.md).
- [UI-RULES.md](../UI-RULES.md), which covers shortcuts and Escape.

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

## 4. What Agent B builds

Every behavior is reached through the one catalog in `ui/commands.rs`. Menu, button, shortcut and search all produce the same `UiIntent::Command(id)`, and they all go through `dispatch::apply`. A command that can't run returns `Availability::Disabled("…reason…")`, and B writes that reason text. **Nothing may pretend to work:** no fake "Saved", no fake undo.

### B0 — Phase 0 contract (section 3)
Do this first, in one small pull request with no visible change:
- Move the code out of `shell.rs` as section 3 says.
- Add `Presentation`, `UiIntent` and `InputContext`.
- Add `tests/presentation.rs`, which checks that `build(&sample_state)` reproduces today's labels, availability and selection.
- Tell A when it is merged, through the status column in section 6.

### B1 — Command catalog to the new design (finish UI-06)
- Apply the confirmed decisions D1–D3: add the Ctrl+Shift+P binding, remove `AddNote`, rename `PanTool` to Hand.
- Add the missing commands from spec §8 and the function map, each with a group, placement, shortcut, `text_field_owns_shortcut`, search aliases and an availability reason:
  - Open, Save As, Close project
  - Add diagram, Rename diagram, Close tab, Next tab, Previous tab, Delete diagram
  - Find object
  - Hand, Relationship kind, Align ×6, Distribute ×2
  - Remove view, Delete model everywhere, Duplicate model, Show in diagram, Reveal
  - Add attribute, Add operation, Member up, Member down, Remove member
  - Zoom in, Zoom out, Actual size, Fit all, Fit selection
  - Grid, Snap, Preferences, Help and shortcuts, Next region, Previous region, Context menu
- Search aliases include terms such as "attribute", "reuse", "fit" and "issues", and each one maps to one canonical label.
- Shortcut dispatch priority (spec §8 "Input dispatch"): modal → IME or text field → gesture → focused region → app command.
  - Each event is consumed once.
  - Single-letter keys (V, H, C, R and G) fire only when the canvas has focus, with no text focus and no IME composition. Read these facts from `InputContext`.
  - Text fields keep their own Ctrl+A, Ctrl+F and Ctrl+Z.
- Tests in `tests/commands.rs` and `tests/shortcuts.rs` cover:
  - every command is reachable from search;
  - menu, button and shortcut all produce the same intent;
  - a more specific shortcut wins (Ctrl+Shift+Z over Ctrl+Z);
  - a letter key never fires while typing or during IME composition.

### B2 — Editor state that can work now, without the project model
These change only **editor state** (`editor_state.rs`, `viewport.rs` and `drafts.rs`), not the document, so they're allowed before R1. They run on the sample classes.

| Behavior | Rule (spec section) |
|---|---|
| Tools: Select, Hand, temporary Space+drag | Explicit `Tool` and `Gesture` enums: Idle, Pan, Marquee, Placement, MovePreview, ResizePreview, ConnectSource, ConnectTarget, InlineEdit (§4). Switching tools cancels a geometry preview. |
| Viewport | `screen = origin + pan + world * zoom`, with a checked inverse. Zoom is clamped to 10–400%, anchored at the cursor or the center. Covers Ctrl+wheel, Ctrl+plus, Ctrl+minus and Ctrl+0, Fit all and Fit selection (Fit selection is disabled when nothing is selected). Panning never changes the document. Tabs keep their own viewport. |
| Picking | Screen-sized hit tolerance and stacking order. Lives in `diagram-canvas` as pure functions, plus tests at several zoom levels. |
| Selection | Click, Shift-click to toggle, marquee for fully enclosed items, Ctrl+A on the canvas, and selection from the navigator list. |
| Escape layering | One layer per press: modal or draft → gesture or tool → selection (UI rule 1.5). |
| Drafts | The rename field edits a draft. Enter commits a valid draft. Escape restores the original value. A valid draft commits on blur. An invalid draft stays and triggers "Resolve edit" when selection, tab or tool changes. Commit goes through one function that M1 will later turn into a real command. |
| Region focus | F6 and Shift+F6 order: header → navigator → board → inspector → drawer → status, skipping hidden regions. Publish the current region in `Presentation`. |
| Preferences | Theme, grid and snap stored outside the document. They never mark the project dirty. |
| Search split | The command search and the object search (Ctrl+F) produce separate result lists in `Presentation`. |
| Tabs | Switch, next/previous and close-tab for **fixture** diagrams. Closing a tab never deletes anything. |

Everything that needs the real model stays **disabled with a reason**: Add class, Connect, members, Save, Export, Undo, Delete and so on.

### B3 — Notation contract (UI-11)
- One `Notation` description lists the display name, tools, inspector sections, relationship kinds and capabilities. v0.1 has Class only.
- The tool row and the "Add diagram" picker are built from it, so unsupported families never appear.

### B4 — Real model and binding (approved to start now)
These follow the roadmap order, R1 then R2 then UI-13 to UI-17. They cover:
- `diagram-core`: typed IDs; model, view and viewport separation; validation including generalization cycles and multiplicity grammar; atomic reversible commands with labels; history; and saved-state identity.
- `diagram-storage`: the `.dgram` schema, atomic save, and recovery.
- The worker protocol.
- Shared scene and SVG.

For each, switch the matching catalog entry from Disabled to the real behavior. Fill the `Presentation` fields that A already draws from fixtures: action labels on Undo and Redo, save state, validation issues, and impact counts.

**Gate note (relaxed by the user, 2026-11-05):** in `roadmap.json`, R1-01 now depends only on R0-01, so B may start B4 immediately. Keep `diagram-core` and `diagram-storage` free of egui. Until A's new screens exist, bind each finished feature to the current shell controls through the catalog and `Presentation`, and add its journey (J5–J7) to `journeys.rs` against the UI that exists then.

### Button → behavior table (A draws it; B makes it work)

| Surface | Control | B's behavior | Works in B2 (now) or B4 (model) |
|---|---|---|---|
| Header | Undo / Redo | History, with labels | B4 (disabled with reason until then) |
| Header | Search commands | Open command search; Ctrl+K (plus Ctrl+Shift+P if D1 is confirmed) | B2 |
| Header | Save / Export | Worker save and SVG export, with real results | B4 |
| Header | Project menu | New, Open, Save As, Close, Preferences, Help | Menu works in B2; file actions in B4 |
| Tabs | Tab, Close tab, + Diagram | Switch and close in B2; Add diagram in B4 | B2 / B4 |
| Tool row | Select, Hand | Tool state | B2 |
| Tool row | Class, Relationship and kind | Placement and connect gestures | Gesture preview in B2 (cancels, creates nothing); commit in B4 |
| Tool row | Arrange → Align/Distribute | Rules (at least 2 or 3 views) with reasons | B4 |
| Panel toggles | Project panel, Inspector | Dock or drawer, focus return | B2 |
| Navigator | Find object, Diagrams/Model tabs, Reveal | Search index and reveal-without-move | B2 on sample data |
| Navigator | Show in diagram, Duplicate, Delete everywhere, Rename diagram | Domain commands and impact counts | B4 |
| Inspector | Name | Draft, commit and cancel | B2 (sample), B4 (real command) |
| Inspector | Members, Appearance, Relation ends | Domain commands | B4 |
| Status | Grid, Snap, Zoom −/+, 100%, Fit, Theme | Preferences and viewport | B2 |
| Status | Issues | Open drawer, Go to object | B4 |
| Dialogs | Save/Discard/Cancel, Retry/Save As, Resolve edit, Restore/Ignore | Close and recovery flow | Resolve edit in B2; the others in B4 |

## 5. Definition of done (Agent B)
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --locked` and `cargo doc --workspace --no-deps --locked` all pass from `app/`.
- No egui painting or layout code exists in B's files, except `commands.rs` keyboard types and `InputContext`. No document mutation happens inside a view.
- Rejected commands leave the state unchanged. Cancelled gestures add no history. Every behavior has a headless test through `dispatch::apply`. Every bug fix has a test proven to fail without the fix.
- `UI-FUNCTION-MAP.csv`: B adds a test that every row's action exists in the catalog, either enabled or disabled with a reason.
- Status is `done` only when the acceptance text holds. Write an evidence page in `app/docs/functionality/`.

## 6. Answers to Agent A's requests (B appends here)

Read [AGENT-A-UI.md §6](AGENT-A-UI.md#6-requests-to-agent-b-a-appends-here-b-reads-and-answers-in-its-own-brief-section-6). For each request, add a row here with the merged commit, then A swaps its fixture.

| A's # | Answer: field, intent or command added | Merged in | Status |
|---|---|---|---|
| (contract) | Additive since Phase 0, for A's new screens: `UiIntent::{ActivateTab, TemporaryPan, CanvasResized, ResolveEdit}`, `Surface::ObjectSearch`, `ResolveChoice`; `Presentation::{object_search, diagram_types}`; `CanvasView::{zoom_percent, grid, marquee, tool}`; `StatusView::{zoom_text, grid, snap, tool_hint}`; `InspectorView::selection_count`; `CommandSearchView::results`; `DialogView::ResolveEdit`. **A's new shell must set `state.support = ShellSupport::NEW_SCREENS`** (or the flags it really supports) and send `CanvasResized`, `CommitDraft` on Enter/blur and `CancelDraft` on Escape; until then the matching commands stay disabled with a reason. | `B-4 to B-10` commit | ready |
| Phase 0 | `Presentation` (`src/presentation.rs`), `UiIntent` and `InputContext` (`src/intent.rs`), `AppState` (`src/app_state.rs`, `ShellState` kept as an alias), `dispatch::apply(&mut AppState, &InputContext, intents)` (`src/dispatch.rs`). `shell.rs` now builds the presentation, draws from it and applies intents. A may edit `shell.rs` from this commit on. | `agent-b/functionality`, commit 6c31494 | done (not yet merged to `main`) |

## 7. Task progress (Agent B only)

Task IDs come from [TASKS-AND-TESTS.md §3](TASKS-AND-TESTS.md#3-task-list). Add a row when a task starts. Set its state to `done` only when its listed tests pass and the joint checkpoint it feeds, if any, still holds.

| Task | State | Branch / commit | Notes |
|---|---|---|---|
| B-0 | done | `agent-b/functionality`, 6c31494 | Contract types added; no visible change; all existing tests and the audit pass; new `tests/presentation.rs` (9) and `tests/contract.rs` (5). |
| B-1 | done | `agent-b/functionality`, d27e498 | `tests/common_state/` helpers; `tests/routing.rs` loops over `CommandId::ALL` (control, shortcut and search routes give the same state; disabled commands change nothing; presentation availability equals the catalog). |
| B-2 | done | d27e498, D2/D3 commit | D1 (Ctrl+Shift+P) and 48 added commands with reasons and aliases; every `UI-FUNCTION-MAP.csv` row maps to a command. D2 (Note removed) and D3 (Pan renamed Hand, H, enabled) landed after merging A's `A-req-B` (5672f10) and `A-req-B2` (c182128). |
| B-3 | done | d27e498 | `commands::take_shortcut_in(&mut InputState, &InputContext)` with spec §8 priority (modal → IME/text → region → app); canvas-only keys need board focus; each event consumed once; the existing `take_shortcut` stays for today's shell. `tests/shortcuts.rs` (11). |
| B-4 | done | `B-4 to B-10` commit | `editor_state.rs`: `Tool` (Select, Hand) and an explicit `Gesture` enum (Idle, Pan, Marquee, MovePreview, ResizePreview, Placement, ConnectSource, ConnectTarget, InlineEdit); tool switches cancel previews; Space+drag via `UiIntent::TemporaryPan`; Hand drags never move a node. Hand is enabled since the D2/D3 commit (J1 journey in the real shell). |
| B-5 | done | same | `diagram-canvas`: checked `Viewport` (`screen = origin + pan + world * zoom`, 10–400%, anchored zoom, fit) and `picking` (screen-sized tolerance, stacking order, marquee); `diagram-core::geometry` finite Point/Size/Rect. Zoom in/out, Actual size, Fit all, Fit selection and Reveal work in today's shell (nodes are drawn through the viewport). Each tab keeps its own viewport. |
| B-6 | done | same | Click, Shift-click toggle, marquee (fully enclosed; Shift adds), click empty clears, Select all (menu/search; Ctrl+A on the canvas once region focus is known). `AppState::selection` added; `selected` stays the primary. |
| B-7 | done | same | `drafts.rs`: draft, commit, cancel, blur-commit, Resolve edit (`DialogView::ResolveEdit`, `UiIntent::ResolveEdit`). One commit path `drafts::commit_class_name`. Today's shell sends no Enter/blur, so with `ShellSupport::draft_events == false` a valid name commits as typed and an invalid one is held (Escape restores). |
| B-8 | done | same | `CommandId::Cancel` owns Escape: Resolve question → draft → gesture → tool → selection, one per press. F6/Shift+F6 cycle visible regions (hidden ones skipped); enabled when the shell sets `ShellSupport::region_focus`. |
| B-9 | done | same | Command search results (commands only) and object search (`ObjectSearchView`, empty and no-match texts) are separate; tabs (switch, next/previous, close-tab-never-deletes) on `AppState::with_second_diagram()`; grid/snap preferences never touch the document. Grid, snap and Find object are enabled when the shell sets `ShellSupport`. |
| B-10 | done | same | `diagram-core::notation`: Class only is creatable; planned notations record their needs (sequence needs its own interaction strategy); `Presentation::diagram_types`; `commands::tool_command` maps notation tools to tool-row commands. The tool row is built from the Class notation since the D2/D3 commit. |
| B-12 (R1-01) | done | R1 commit | Typed IDs (`compile_fail` doc tests), model/view records, finite geometry, explicit order. |
| B-12 (R1-02) | done | R1 commit | `DomainError`, cycle/self/duplicate generalization checks, multiplicity grammar, document check with severities. |
| B-12 (R1-03) | done | R1 commit | `Command` → `ChangeSet` patches; all-or-nothing apply with rollback; exact inverse; delete-everywhere impact. |
| B-12 (R1-04) | done | R1 commit | `Session`: undo/redo labels, SessionId/Revision/StateId, saved state across undo/redo/branching, 500-transaction bound. |
| B-11 | in_progress | — | J1 is in `journeys.rs` against today's shell. J2–J4 need A's inspector drafts (`ShellSupport::draft_events`), region focus and drawer flows on the new shell. |
| R1-05/R1-06, UI-13 to UI-17 | not started | — | Storage schema and save are next; then `Session` binding switches Undo, Add class, Connect and members on. |
