# Agent B — Functionality (what every button does)

Date: 2026-11-05. Partner brief: [AGENT-A-UI.md](AGENT-A-UI.md). Both agents work at the same time. **The two briefs share sections 1–3 word for word.** If they ever differ, stop and ask the user.

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
| Tests | `tests/theme.rs`, `typography.rs`, `showcase.rs`, `welcome.rs`, `workspace.rs`, `shell.rs`, `ui_audit.rs`; new `tests/views_*.rs` | `tests/commands.rs`, `diagnostics.rs`; new `tests/dispatch.rs`, `editor_state.rs`, `presentation.rs`, `shortcuts.rs`; all tests in the other crates |
| Docs | `app/docs/ui/*.md` and screenshots, except `new-design/` | new `app/docs/functionality/*.md` and `plan-rust-diagram-editor/PROGRESS-AND-NEXT-STEPS.md` |
| Tracker | `roadmap.json` entries for UI-02, UI-05, UI-07, UI-08, UI-09, UI-10, UI-12 and UI-18, plus R0-04 | `roadmap.json` entries for UI-06, UI-11, UI-13 to UI-17, and R1-xx/R2-xx when allowed |
| Read-only for both | `app/docs/ui/new-design/*`, `UI-RULES.md`, `UI-DESIGN-AND-INTEGRATION.md`, the other agent's brief | same |

Only one place is shared: **`roadmap.json`**. Edit only your own task objects, rebase immediately before pushing, and run `python build_rust_roadmap.py`.

## 3. Order, branches and merging

| Step | Who | What | Blocks |
|---|---|---|---|
| **Phase 0** (first, small) | B | Move `ShellState`, `run()` and `command_context()` out of `shell.rs` into `app_state.rs` and `dispatch.rs`. Add `presentation.rs` and `intent.rs` with the types in section 1, built from today's sample state. Make `shell.rs` call `presentation::build(&state)` then `dispatch::apply(&mut state, intents)`. There is **no visible change**, and all existing tests and the audit must still pass. | A may not edit `shell.rs` until Phase 0 is merged. A may start on every *new* A file at once, using `fixtures.rs`. |
| Parallel work | A and B | Each works on its own branch: `agent-a/new-ui` and `agent-b/functionality`. Rebase on `main` daily. | — |
| Merge order | — | B's contract additions merge **before** A's views that use them. If A is ahead, A keeps the fixture, then swaps to the real field after B merges. | — |
| Integration | A | When B adds a working feature, A removes the matching fixture and adds the state to `ui_audit.rs`. | — |

**Decisions the user must confirm before B changes the catalog:**
- **D1:** Command search shortcut. Recommended: keep Ctrl+K and add Ctrl+Shift+P as a second binding.
- **D2:** Remove the **Note** tool from the tool row for v0.1. Recommended: yes, because the spec says unsupported tools must be absent.
- **D3:** Rename **Pan** to **Hand** (shortcut H, with Space+drag for temporary panning). Recommended: yes.

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
- Apply decisions D1–D3 once the user confirms them.
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

### B4 — Real model and binding (needs the user's approval; see the gate note below)
These follow the roadmap order, R1 then R2 then UI-13 to UI-17. They cover:
- `diagram-core`: typed IDs; model, view and viewport separation; validation including generalization cycles and multiplicity grammar; atomic reversible commands with labels; history; and saved-state identity.
- `diagram-storage`: the `.dgram` schema, atomic save, and recovery.
- The worker protocol.
- Shared scene and SVG.

For each, switch the matching catalog entry from Disabled to the real behavior. Fill the `Presentation` fields that A already draws from fixtures: action labels on Undo and Redo, save state, validation issues, and impact counts.

**Gate note:** in `roadmap.json`, R1-01 depends on `R0-05` and `UI-12`. B may **not** start B4 while those are open, unless the user explicitly relaxes the gate. If the user does relax it, B updates `depends_on` and the validator in the same commit, and writes down why.

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
| Phase 0 | `Presentation`, `UiIntent`, `InputContext`, `dispatch::apply` | — | not started |
