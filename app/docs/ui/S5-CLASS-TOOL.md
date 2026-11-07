# S5 — the Class tool and the duplicate-name dialog

Branch `feat/p2-class-tool`. Plan: [SLICE-p2-class-tool.md](../next-steps/SLICE-p2-class-tool.md). Roadmap: R2-04 stays **in progress**, because its members half is S6. UI-14 and UI-09 gain the Class tool and the duplicate-name dialog.

## What changed for the user

- **Class** in the tool row is enabled whenever a diagram is open. With no diagram open it is disabled, and its tooltip says why: "Open a diagram first: a new class is placed on the diagram you are viewing." While it is active it shows as pressed, and the status bar says how to use it.
- **Clicking the canvas** places a class box there and opens a "New class name" field inside the canvas. The field has focus, and "Class N" (the smallest free number) is selected, so typing replaces it.
  - Enter, or moving focus away, creates the class as one undo step ("Add class"). The new class is selected, and the tool goes back to Select.
  - Escape cancels, and nothing is created.
  - A blank name shows "Error: …" beside the field. Enter, a blur or a click elsewhere keeps the field and its focus.
- **Any other action while naming** finishes the naming first. This covers a tab click, closing a tab, Close project, Ctrl+K and clicking another class. A blank name stops that action: the placement stays, and focus returns to the field. Undo or Redo instead cancel the naming.
- **The keyboard:** Tab to the canvas and press Enter to place a class at the first free grid spot. Escape with the Class tool armed returns to Select and keeps the selection.
- **The duplicate-name dialog (D2).** Names are compared trimmed and case-sensitive. When the name exists, a dialog asks what to do, with Cancel focused first:
  - **Show the existing class here** adds a view of that class, at its drawn size and kept inside the canvas. It is disabled, with the reason as visible text, when every class with that name is already in this diagram.
  - **Keep a new class** creates a second class with the same name.
  - **Cancel**, Escape, or a click outside goes back to the field with the name kept.

## Structure

- `ui/class_tool.rs` holds:
  - the tool and placement data (`ClassTool`, `Placement`, `PlacementId`, `Phase`);
  - the pure rules `decide`, `default_class_name`, `fit_inside`, `next_free_spot` and `bounds`;
  - the naming panel.
- The coordinator's part (begin, preflight, finish, place, show here, cancel, answer) is in `shell/placing.rs`. The per-request helpers moved to `shell/actions.rs`, unchanged. `shell.rs` went from 1,177 to 861 lines.
- Earlier structure-only commits on this branch moved key routing to `ui/keys.rs`, the drawer to `ui/drawer.rs`, command contracts to `ui/contracts.rs` and the showcase to `ui/showcase.rs`.
- The command search's result list publishes a clipping scroll region, so the audit no longer reports scrolled-out rows as outside the window.

## Tests

- `tests/class_tool.rs` (37 tests) covers:
  - the pure rules;
  - availability;
  - one-step placing and naming;
  - Escape and blur;
  - blank names;
  - text undo inside the field;
  - the first frame after Begin;
  - the keyboard path;
  - repeated and stale commits;
  - Undo and Redo in a batch;
  - the preflight for tabs, Ctrl+K, Close project and class clicks;
  - blank names blocking each of them;
  - a duplicate commit with navigation, in both orders;
  - geometry at the edges.
- `tests/dialogs.rs` (12 tests) covers:
  - the words;
  - focus on Cancel;
  - Show here, including a tall class near the bottom;
  - Keep new;
  - Cancel, Escape and an outside click;
  - the disabled reason;
  - the keyboard path;
  - a tab click whose commit asks;
  - name matching.
- `tests/ui_audit.rs` adds the Class tool active, naming at the bottom-right edge with and without the error, and both dialog variants, at 800×500, 1280×800 and 1440×900.

### Proven to fail without the fix

Each fix below was removed in turn, and its test failed. The script is in the session scratchpad.

| # | Fix removed | Test that failed |
|---|---|---|
| 1 | `decide` asks when the name exists | `an_existing_name_should_open_the_dialog_with_focus_on_cancel` |
| 2 | the default name is selected | `clicking_the_canvas_should_place_and_name_a_class_in_one_step` |
| 3 | the tool returns to Select | same |
| 4 | Escape returns the Class tool to Select before clearing the selection | `escape_should_return_the_class_tool_to_select_and_keep_the_selection` |
| 5 | a history move cancels the placement | `undo_or_redo_in_a_batch_should_cancel_the_placement_and_skip_the_commit` |
| 6 | the placement-ID check | `repeated_and_stale_commits_should_create_one_class` |
| 7 | the preflight | `switching_tabs_while_naming_should_create_the_class_first`, `ctrl_k_while_naming_should_create_the_class_then_open_search` |
| 8 | focus returns to the field after Cancel | `cancel_escape_and_outside_click_should_return_to_the_field` |
| 9 | Show here uses `ShowInDiagram` | `show_here_should_share_the_class_at_its_drawn_size` |
| 10 | naming owns the text keys in its first frame | `ctrl_z_and_escape_before_the_panel_renders_should_not_touch_the_history` |
| 11 | a commit that asks stops the batch | `a_duplicate_commit_with_navigation_should_ask_and_stay` |

| 12 | command search takes every key while open | `escape_in_command_search_should_not_change_the_tool` |
| 13 | Enter checks the text as edited this frame | `text_and_enter_in_one_frame_should_create_the_class` |
| 14 | the naming panel blocks clicks through its padding | `a_click_on_the_panel_padding_should_not_reach_a_class_under_it` |
| 15 | placement IDs survive project changes | `a_late_request_from_a_closed_project_should_not_touch_a_new_placement` |
| 16 | long names are shortened in the dialog | `a_long_duplicate_name_should_keep_the_buttons_in_a_small_window` |

Fixes 12–16 came from the Codex code review (xhigh, read-only). Round 1 found seven warnings and round 2 one more. Round 3 approved.

The tests also found one bug, fixed here: `next_free_spot` counted one grid cell on a canvas narrower than a cell, because integer division rounds toward zero. It now uses Euclidean division. The test is `next_free_spot_should_take_the_first_free_grid_cell`.

## Real window (Windows 11, release build)

| What | Result | Screenshot |
|---|---|---|
| Sample project, Select pressed | Pass | [01](p2-class-tool/01-sample-1280x800-light.png) |
| Class pressed, status hint shown | Pass | [02](p2-class-tool/02-class-tool-active-1280x800-light.png) |
| Canvas click: field focused, "Class 1" selected | Pass | [03](p2-class-tool/03-naming-1280x800-light.png) |
| "Invoice" + Enter: created at the click point and selected | Pass | [04](p2-class-tool/04-created-invoice-1280x800-light.png) |
| "Customer" in Class diagram 1: dialog, Show here disabled with its reason | Pass | [05](p2-class-tool/05-duplicate-already-here-1280x800-light.png) |
| Escape: back in the field, name kept | Pass | [06](p2-class-tool/06-dialog-cancelled-back-in-field-1280x800-light.png) |
| "Customer" in Order details: Show here enabled | Pass | [07](p2-class-tool/07-duplicate-show-here-1280x800-light.png) |
| Show here: Customer shown with its members, selected; inspector says "Shown in Class diagram 1 and Order details" | Pass | [08](p2-class-tool/08-shown-here-1280x800-light.png) |
| Dark, 800×500 | Pass | [09](p2-class-tool/09-dark-800x500.png) |
| Naming at the bottom-right edge: panel inside the canvas | Pass | [10](p2-class-tool/10-naming-edge-dark-800x500.png) |
| Blank name: error beside the field, panel not moved | Pass | [11](p2-class-tool/11-naming-error-edge-dark-800x500.png) |
| Escape: cancelled, nothing created | Pass | [12](p2-class-tool/12-escape-cancelled-dark-800x500.png) |
| Duplicate dialog, dark, 1280×800 | Pass | [13](p2-class-tool/13-duplicate-dark-1280x800.png) |
| Class tool, light, 800×500: the status hint shortens with an ellipsis | Pass | [14](p2-class-tool/14-class-tool-light-800x500.png) |

Known limits, for later slices:

- At 800×500, the Customer box placed at 1280×800 sits below the visible canvas. Pan and zoom arrive in S12.
- When the visible canvas is full, a new class may overlap others until then.
