# Agent A — new view modules and shell composition (evidence)

Date: 2026-11-05. Branch `agent-a/new-ui`. Tasks A-0 to A-11 and A-13 from [TASKS-AND-TESTS.md](../../../plan-rust-diagram-editor/agents/TASKS-AND-TESTS.md). Roadmap tasks fed: UI-02, UI-05, UI-07, UI-08 and UI-09. All of them are **in progress**; none is marked done.

## What was built

The shell (`src/shell.rs`) now only composes the layout. Every region is drawn by a view module from Agent B's `Presentation`, and each view pushes `UiIntent`s. A view never takes `&mut AppState`.

| Module | Draws | Pushes |
|---|---|---|
| `ui/header.rs` | Project menu (one submenu per command group, shortcut text, "Unavailable" plus the catalog reason on hover for disabled items), project title that shortens with an ellipsis, header commands with group separators | `Command` |
| `ui/tabs.rs` | Tabs; the active one is pressed, bold, underlined and reported as selected. When tabs do not fit, a window of whole tabs plus a "More tabs" menu that reveals and focuses a tab | `ActivateTab`, `Command` |
| `ui/toolbar.rs` | Project panel and Inspector toggles at the ends, tools pressed when active, and "More tools" when the tools do not fit | `Command` (+ the toggle that opened a drawer, for focus return) |
| `ui/navigator.rs` | Diagrams and Model rows (32 high, ellipsis, full name as the accessible name), empty states | `Select` (Shift toggles) |
| `ui/search_view.rs` | Object search ("Find object"), with kind plus location per result, no-match text, and Close search | `Search { Objects }`, `Select`, `Close(ObjectSearch)` |
| `ui/canvas_view.rs` | Class boxes as accessible buttons at the presented rectangles, selection handles, the 16-unit grid, the marquee, empty text; document colors from `theme::DOCUMENT_DEFAULT_*`, not chrome tokens | `Select`, `Pointer` Down/Move/Up/Zoom with hit, `CanvasResized`, `TemporaryPan` |
| `ui/inspector.rs` | Guidance, "Scope: Shared model — affects all views", a Name draft field with its problem beside it, attributes | `DraftChanged`, `CommitDraft` (Enter or blur), `CancelDraft` (Escape) |
| `ui/status.rs` | Save state, "Zoom 100% · Grid on · Snap off", selection, tool hint, theme chooser | `SetTheme` |
| `ui/dialogs.rs` | Resolve edit: default focus on "Return to field"; Escape and clicking outside return to the field; Tab stays inside | `ResolveEdit` |
| `ui/fixtures.rs` | One `Presentation` per screen state, built by `presentation::build` (B's code). Labels, shortcuts and reasons therefore all come from the catalog | — |

Fonts (A-11): Noto Sans Regular and Noto Sans Arabic Regular are in `app/crates/diagram-desktop/assets/fonts/`, with their OFL texts, source paths and SHA-256 sums. They are registered as the first fallbacks after egui's primary font.

## Problems found and fixed

Each fix has a test that failed before the fix and passes after it.

| Problem | Found by | Fix | Regression test |
|---|---|---|---|
| Enter in the Name field emitted `CommitDraft` twice (`lost_focus` reported on two runs) | `ui_routing` | Commit or cancel once per focus session (local widget state) | `name_field_should_emit_draft_changed_commit_and_cancel` (2 commits before the fix) |
| Tabs hidden by a scroll strip kept unclipped rectangles in the accessibility tree, outside the window and under "New diagram" | `views_tabs` audit (14 findings) | Draw only whole tabs; "More tabs" moves the window | `tab_row_should_pass_the_audit_at_every_size_and_theme` |
| With B's full catalog, the File submenu ran past the bottom of 800×500 | `ui_audit` | One-line menu rows, "Unavailable" plus the reason on hover | `open_project_menu_should_pass_the_audit` |
| Command search rows scrolled out of the list kept off-window rectangles (also reported by Agent B for D2/D3) | `ui_audit` | `ScrollArea::show_rows` lays out only visible rows | `command_search_should_pass_the_audit` |
| A board drag reported `Down` past the press point, so a Hand pan lost its first step (reported by Agent B) | B's journey work | `Down` uses `pointer.press_origin()` | `board_drag_should_report_down_at_the_press_position` (Down 720,510 instead of 700,500 before the fix) |
| Noto Sans as primary face pushed the welcome screen 16 points past 800×500 | `ui_audit` | Not fixed: Noto stays a fallback; the missing-file error now shares a line with its Remove button | `welcome_screen_should_pass_the_audit` |

## Verification

| Check | Result |
|---|---|
| `cargo fmt --check`, `clippy -D warnings`, `test`, `doc` (whole workspace) | Pass at every commit; 281 tests after merging B's final head |
| Every view at 800×500, 1280×800 and 1440×900 in light and dark themes: names, 32-point hit height, inside the window, no overlap, Tab reachability | Pass (`views_*.rs`, `audit_view_everywhere`) |
| H1/H2 for every placed command and every catalog command (through the Project menu) | Pass (`ui_routing.rs`) |
| Non-command intents: select, Shift toggle, draft, commit, cancel, pointer, zoom, resize, Space pan, tab, theme, object search, dialog | Pass (`ui_routing.rs`, `views_*.rs`) |
| `grep '&mut ShellState\|&mut AppState' src/ui` | No matches |
| Real window and screenshots | **Not done.** The run-time library `libxkbcommon-x11` is missing in this container, and the coordinator reported that the app panics under Xvfb. I did not run the real window, and no new screenshots are claimed. The existing PNGs in this folder show the earlier shell. |

## Open items

- **Turned off on purpose:** `ShellSupport::draft_events` and `resolve_dialog`. The views send the draft events and the dialog works in its tests. However, turning drafts on fails B's `journeys.rs` test `escape_after_typing_a_valid_name_should_keep_it`, which expects the earlier live-commit behavior (request 10 in AGENT-A-UI.md §6). `region_focus` is off because the shell does not move focus to regions yet.
- **Audit limit:** the audit cannot see scroll viewports, so content inside a scrolling panel that continues below the window is reported as off-window. For this reason, the object search in the 800×500 project drawer is audited only at 1280×800 and 1440×900.
- **Contract still needed from B (requests 1–9):**
  - save badge and history labels;
  - tab close;
  - kind chooser and Arrange menu;
  - navigator tabs and row actions;
  - edges, previews and document style;
  - the other inspector layouts;
  - status commands;
  - the other dialogs and the validation drawer;
  - moving the command search to its own view.
- **Font:** Noto Sans as the primary face needs the entry layouts reworked first. Arabic shaping, bidi, IME and screen readers stay in R0-04 and UI-10.
