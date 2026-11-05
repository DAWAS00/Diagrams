# Architecture map

Read this when deciding where code goes, or before moving code between modules.

## Crates

| Crate | Owns | Must not depend on |
|---|---|---|
| `diagram-core` | Domain values and rules (`ProjectName` today; IDs, model and commands in M1) | egui, SQL, OS dialogs |
| `diagram-canvas` | Renderer-neutral geometry and scenes (M2) | egui widgets, storage |
| `diagram-storage` | SQLite persistence and recovery (`sqlite_version` today) | egui, pointer input |
| `diagram-desktop` | The egui application: shell, UI modules, diagnostics | — (it is the top) |

## `diagram-desktop` modules

| Module | Responsibility | Pure or draws |
|---|---|---|
| `shell.rs` | `ShellState` (editor state and prototype fixtures), the coordinator `apply_intents` and `run`, and currently the drawing of most surfaces (**debt: split it**) | both (too much) |
| `ui/intents.rs` | `UiIntent`: every requested state change | pure data |
| `ui/commands.rs` | Command catalog, placements, availability with reasons, `take_shortcut`, `ActionContract` | pure (plus input) |
| `ui/theme.rs` | `Palette` tokens, `TextRole`, egui style, contrast pairs | pure |
| `ui/components.rs` | Reusable widgets: labeled field, row, action button, error and warning text | draws |
| `ui/workspace.rs` | Row heights, panel sizes, `layout_mode`, `Drawer`, `PanelPreferences` | pure |
| `ui/navigator.rs` | Diagram fixtures, `OpenTab`, search matching, "shown in" wording, which tabs fit | pure |
| `ui/inspector.rs` | `PreviewRelation`, field rules (multiplicity, class name), `DraftKey`, `DraftOutcome` | pure |
| `ui/notation.rs` | `NOTATIONS` table: tools, sections, relationships, interaction per notation | pure |
| `ui/palette.rs` | Command and object search results and highlight logic | pure |
| `ui/welcome.rs` | Entry state, sample recent list, name rule, welcome and new-project screens | both (small) |
| `ui/showcase.rs` | Component showcase fixture | draws |
| `ui/status.rs` | Status bar (first surface moved out of `shell.rs`) | draws |
| `lib.rs`, `main.rs` | Native options, `--version`, launch diagnostics, panic hook | — |

## State and flow rules

- **`ShellState` is the only editor state.** Widgets get it by reference and only **read** it, apart from text drafts.
- **Changes travel as `UiIntent`.** Widgets call `state.queue(intent)`. `apply_intents` runs once per frame after drawing, in queue order. `Run { command, origin }` goes through `run()`, which checks `commands::availability` first; `origin` gets focus back when what it opened closes.
- **Entry screens** use the same idea with `EntryAction`, which `entry_screens` applies.
- **Validation happens twice:** the widget shows the error on the draft, and the coordinator re-checks before writing. A bug in one place therefore cannot corrupt a record.
- **Focus rules:** shortcuts use the previous frame's `text_edit_focused()`. Focus returned after a modal waits until `top_modal_layer()` is empty.
- **Scroll regions** use `scrolling(ui, id, label, …)`, which exposes a named, clipping `ScrollView` node. Use it for any new scrolled content.

## Playbook: moving a surface out of `shell.rs`

Do this when you next touch a surface, one surface per commit:

1. Run the full test suite and the maintainability check. Everything must be green before you start.
2. Create `ui/<surface>.rs`. Move the drawing function or functions **unchanged**, make them `pub(crate)`, and import them in `shell.rs`. Move the helpers only that surface uses; leave shared helpers in `components.rs`.
3. If the function wrote state directly, that is a bug: change it to queue an intent, in a **separate, later commit** with its own test.
4. Build, run fmt, clippy and the tests. **No test may change** in a pure move. If one has to, the move changed behaviour, so stop and split the commit.
5. Run the maintainability check: `shell.rs` lines should go down, and no new WARN may appear.
6. Commit with a message like `refactor(ui): move <surface> out of shell.rs (no behaviour change)`.

Suggested order (most self-contained first): status bar (done: `ui/status.rs`) → showcase window → command search → drawer → tab row → tool row → header and menu → project panel → inspector → canvas → entry screens.
