# Slice S5: the Class tool and the duplicate-name dialog (feat/p2-class-tool)

Base: `main` at `e26dcde` (S4 merged). Baseline: 318 tests pass; maintainability shows 0 FAIL and 7 WARN. Two of the warnings are file sizes: `shell.rs` at 1,055 lines and `ui/commands.rs` at 634.

Roadmap: this slice contributes to R2-04, UI-14 and UI-09 (the duplicate-name dialog only). No status changes, because R2-04 depends on R2-02 (S7) and UI-14 depends on R2-04. Decision D2 is in [ADR-003](../../../plan-rust-diagram-editor/decisions/ADR-003-UI-DECISIONS.md).

## What the user gets

1. **Choose the Class tool.** Choose **Class** in the tool row, or search for it with Ctrl+K. It shows as pressed, and the status bar says what to do next.
2. **Place the class.** Click the canvas to place a new class there. From the keyboard, Tab to the canvas and press Enter, and the class goes to the next free visible spot.
3. **Name it.** A small naming panel appears where the class goes. It has the label "New class name", a field already holding "Class 1" (the first unused number) with all of it selected, so typing replaces it, and the hint "Enter creates the class. Escape cancels."
   - **Enter, or moving focus away,** creates the class as one undoable step ("Add class"). The new class is selected and the tool is back to Select.
   - **Escape** cancels, and nothing is created.
4. **A name that already exists** opens a dialog in words (D2) with three choices:
   - "Show the existing class here": one step that adds a view of the existing class to this diagram;
   - "Keep a new class": one step that creates a separate class with the same name;
   - "Cancel" (focused first; also Escape or a click outside): returns to the naming panel with the typed text, and focus goes back to the field.

## One placement, one lifecycle

The coordinator owns one private record:

```rust
struct Placement { id: PlacementId, diagram: DiagramId, at: CanvasPoint, visible: CanvasSize, phase: Phase }
enum Phase { Naming, Asking { existing: Option<ElementId> } }
```

- **Fields:** `PlacementId` is a counter. `CanvasPoint { x: i32, y: i32 }` is a whole-point canvas position, and `CanvasSize` is a whole-point size (the visible canvas when the placement began). Being whole numbers, they let intents and dialogs keep their `Eq` derives.
- **Intents name the placement, never coordinates:** `CommitNewClass { placement, name }`, `CancelNewClass(placement)`, and the dialog `DuplicateClassName { placement, name }`.
- **Every intent and answer is checked against the live placement:** the ID must match, and the phase must be right (`Naming` for a commit, `Asking` for a dialog answer). If not, it does nothing. So a late, repeated or stale commit can never create a second class, and a commit cannot reopen the dialog while it is asking.
- **When the placement ends:** it is consumed on success, on Cancel and on a refusal by the core. A refusal shows a notice: "Error: … Nothing was changed."
- **Preconditions for `BeginNewClass`:** the Class tool is active, the target diagram exists and is the active diagram, no dialog or command search is open, and no placement exists. Otherwise it does nothing.

## Any other action first finishes the naming

**Placement intents** are `BeginNewClass`, `CommitNewClass` and `CancelNewClass`. `BeginNewClass` while a placement exists does nothing at all: same placement ID, same draft, same revision, same history.

**One finish step.** `finish_naming(state, ctx, placement, name) -> Finish` is the only code that ends naming by its name. Both `CommitNewClass` and the preflight call it.

**The preflight.** While a placement is `Naming`, the coordinator runs it before the first intent in the batch that is not a placement intent. It calls `finish_naming` with the current draft (`drafts[NewClassName]`), exactly as Enter would.

**Results.** Whichever caller ran `finish_naming`, its result decides the rest of the batch:

- **Completed:** the class was created (or the core refused, which ends the placement with a notice). The batch goes on, so a tab click creates the class in its own, still-active diagram, selected there, and then switches. Close project then sees the change and asks as usual. Ctrl+K and the showcase open after the class exists.
- **Blocked:** the name is invalid. The rest of the batch is dropped and the placement stays. Focus returns to the field, and the notice says "Error: A class name must contain a non-whitespace character. Name the new class, or press Escape to cancel."
- **Asking:** the name exists. The duplicate dialog opens, and the rest of the batch is dropped, because the question must be answered first.

So `[CommitNewClass(duplicate), ActivateTab(other)]` and `[ActivateTab(other), CommitNewClass(duplicate)]` both end with the dialog open and the tab unchanged. In the second order, the preflight before `ActivateTab` is what asks.

This does not depend on a focus-loss commit existing: Ctrl+K never takes focus from the field, and an invalid draft queues nothing. A focus-loss commit that arrives later in the same batch finds the placement consumed or `Asking`, and does nothing.

`Run(Undo)` and `Run(Redo)` are the exception; see "Undo, Redo and other ways a placement ends".

## Who owns the keyboard while naming

- **Shortcuts:** while a placement is `Naming`, shortcut routing treats text as focused (`text_had_focus || naming`), so the keys a text field owns (Ctrl+Z, Delete, …) never reach the catalog. This holds in the very first frame after `BeginNewClass`, before the field has focus.
- **Escape:** in `ui/keys.rs`, Escape while `Naming`, when the field did not have focus in the previous frame, queues `CancelNewClass`. When the field has focus, the field itself turns Escape into `CancelNewClass`. Either way, Escape while naming cancels it, and never reaches Clear selection.
- **The naming panel acts only while it owns the interaction:** placement `Naming`, no dialog, and no command search open. Only then does it queue a commit or a cancel, or request focus back for an invalid name.
  - The search and the showcase cannot open over a `Naming` placement, because the preflight finishes the naming first.
  - While the dialog is open, the field is not focused, and Escape and outside clicks belong to the dialog.
- **Enter:** Escape is checked before the general focus-loss path. Any other loss of focus queues a commit. Enter on a single-line field gives up focus, so Enter arrives through focus loss, not through `has_focus()`.
- **An invalid name** shows "Error: A class name must contain a non-whitespace character." beside the field, and the panel requests focus back; nothing is queued.
- **`BeginNewClass` prepares the field:** it stores a fresh `TextEditState` (empty text undo history, all of the default name selected) under the field's fixed `Id`. Focus comes through `pending_focus`, so the field is focused in the frame it first appears. The panel is an ordinary part of the canvas `Ui` (see Geometry), not an `egui::Area`, so there is no invisible sizing pass that could drop the focus or the selection.
- **A cold placement** (no test focusing the field by hand) proves the field is focused with all of the default name selected in its first interactive frame.
- **Cancel in the dialog** sets `pending_focus` to the field, and the phase goes back to `Naming`. The field gains focus; it does not lose it, so the dialog does not reopen.

## Undo, Redo and other ways a placement ends

| Event | Effect |
|---|---|
| Any `Run(Undo)` or `Run(Redo)` in the batch, available or not | The placement is cancelled first. Its commit and any committing dialog answer in the same batch are skipped. Its dialog is closed and its focus return cleared. Then Undo or Redo runs as usual. This extends the existing `history_in_batch` rule for drafts. |
| Opening or closing a project | The placement is dropped (the epoch changes; `open_project` and `close_project` reset it). |
| `BeginNewClass` | Closes any open drawer, so the panel is never under a drawer. |
| Its diagram no longer exists | The commit is refused by the core and becomes a notice; the placement is dropped. |
| `DraftKey::NewClassName` | Kept by `after_document_change` only while a placement exists. |

## Geometry

- **The class box:**
  - `CLASS_SIZE` is an empty class box (`canvas::card_size` of an empty card).
  - A click becomes `at = fit_inside(click, CLASS_SIZE, visible)`, so the whole box is inside the visible canvas. `fit_inside` is never negative, and it gives 0 on an axis where the canvas is smaller than the box.
- **The naming panel:**
  - It is drawn inside the canvas `Ui` with `ui.scope_builder(UiBuilder::new().max_rect(panel))`, after the class boxes, so it is on top and its widgets take clicks before the background does.
  - Its size is **measured before drawing**, every frame. The width is `PANEL_WIDTH`. The height is the label, plus the field (`CONTROL_HEIGHT`), plus the **taller** of the hint and the error, each laid out with the fonts at `PANEL_WIDTH`, plus padding.
  - Reserving the taller of the two keeps the panel the same size whether the error shows or not. So the first frame with an error neither jumps nor clips.
  - `panel = fit_inside(at, size, canvas)`.
  - It overlays the box, and it is not part of the stored geometry.
- **Keyboard placement, `next_free_spot(occupied, CLASS_SIZE, visible)`:**
  - `occupied` holds the rendered rectangles of the diagram's class views (`canvas::card_size` of each read-model card, so members count);
  - it scans a grid with a 48-point margin and a pitch of `CLASS_SIZE` plus 40, with as many columns and rows as fit in `visible`, row by row;
  - it returns the first cell whose rectangle overlaps nothing;
  - if no visible cell is free, it returns `fit_inside((48, 48), CLASS_SIZE, visible)`. The overlap is allowed and documented, because pan and zoom arrive in S12.
- **Bounds stored in the core:**
  - `CreateClass` gets `CLASS_SIZE` at `at`.
  - "Show the existing class here" gets that class's **rendered** size (its card with members), at `fit_inside(at, that size, placement.visible)`. So a tall class near the bottom edge moves up to fit. When it is taller or wider than the visible canvas, it starts at 0 on that axis, and the rest is reached with pan in S12.

## Names

- **Desktop policy:** a new class's name is trimmed before creation. Duplicate lookup compares trimmed names and is case-sensitive. So " Customer " matches "Customer", "customer" does not, and an existing core name stored with surrounding spaces matches its trimmed form.
- **`Name` itself is unchanged:** `diagram-core` stores the spelling it is given.
- **`decide(document, diagram, typed) -> Decision`:**
  - `Create(trimmed)` when no class has the trimmed name;
  - otherwise `Ask { existing }`, where `existing` is the lowest-ID class with that name not already shown in `diagram`, or `None` when every class with that name is already in it.
- **`default_class_name(document)`:** "Class N" with the smallest N ≥ 1 that no class uses.

## Scope by commit

1. **Structure only** (no behaviour change, tests green):
   - `drawer()` moves to `ui/drawer.rs` *(new)*;
   - `scrolling()` moves to `ui/components.rs`, where the docked panels use it too;
   - the Escape and catalog-shortcut routing at the top of `show()` moves to `ui/keys.rs::route_keys` *(new)*;
   - `contract()` and `ActionContract` move from `ui/commands.rs` to `ui/contracts.rs` *(new)*, as the skill's debt list says;
   - the new modules are registered in `ui/mod.rs`.
2. **The showcase window queues `CloseShowcase`** instead of calling `close_showcase()` while drawing. This is a small behaviour change, made in place in `shell.rs`, with a test.
3. **Structure only:** `showcase_window()` moves unchanged to `ui/showcase.rs`.
4. **The S5 behaviour:**

| File | Work |
|---|---|
| `ui/class_tool.rs` *(new)* | `Tool`, `CanvasPoint`, `CanvasSize`, `PlacementId`, `Decision`, the pure rules (`default_class_name`, `next_free_spot`, `fit_inside`, `decide`, `CLASS_SIZE`) and `naming_panel(ui, state, canvas)`, which draws the panel and queues intents. |
| `ui/intents.rs` | `BeginNewClass { diagram, at: CanvasPoint, visible: CanvasSize }`, `CommitNewClass { placement, name }` and `CancelNewClass(PlacementId)`. |
| `ui/commands.rs` | `AddClass` is enabled when a diagram is open. Otherwise its reason is "Open a diagram first: a new class is placed on the diagram you are viewing." New `CommandContext::has_diagram`. |
| `ui/contracts.rs` | `AddClass` becomes Editor and `NoProjectChange` (it only arms the tool; creation is the one-step command); `AddClass` and `SelectTool` name their new tests. |
| `ui/components.rs` | `tool_choice(ui, selected, label, disabled_reason)`: a selectable button that keeps its border, is disabled with hover text when it has a reason, and returns its `Response`. `text_field_at(ui, label, value, id, error)`: the labelled field with a fixed `Id`; `text_field` and `text_field_with_error` delegate to it. |
| `ui/tool_row.rs` | Select and Class use `tool_choice`, pressed from `active_tool()`, with Class's availability reason. |
| `ui/canvas.rs` | With the Class tool and no placement: a click on the background or on a box queues `BeginNewClass` at `fit_inside` of the click (`response.interact_pointer_pos()`). A keyboard or accessibility activation (no pointer position) uses `next_free_spot`. With a placement, it draws `naming_panel`. Its other clicks queue their usual intents, which the preflight runs after the naming is finished. New empty-diagram message: "This diagram is empty. Choose Class in the tool row, then click here to place a class." |
| `ui/dialogs.rs` | `DialogKind::DuplicateClassName { placement, name, existing: Option<ElementId> }`, `DialogAnswer::Alternative`, and `DialogText` with `alternative` and `alternative_disabled` (a reason shown as visible text beside the disabled button). The words name the class: "A class named Customer already exists in this project. Show that class here, so both diagrams share it, or keep a new, separate class with the same name." |
| `ui/inspector.rs` | `DraftKey::NewClassName`. |
| `ui/keys.rs` | Escape while naming cancels it (see "Who owns the keyboard while naming"). A Class-tool Escape step comes after the drawer step: with the tool active, no placement, no dialog and no text focus in the previous frame, Escape returns to Select. Shortcut routing treats a `Naming` placement as text focus. |
| `ui/status.rs` | While the Class tool is active: "Class tool: click the canvas, or press Enter on it, to place a class. Escape returns to Select." |
| `shell.rs` | Private `tool` and `placement` fields with `active_tool()` and `placement()` read methods, plus the coordinator rules above. No new public fields. |

`diagram-core` does not change: `CreateClass` ("Add class") and `ShowInDiagram` ("Show in diagram") are already one undo step each.

**Size budget:** `shell.rs` ends below 1,000 lines, `ui/commands.rs` below 634, and no file that is already over a limit grows. The individual measurements are compared, not only the warning count.

## Escape order

Escape works in this order: **text field** (cancel naming) → **dialog** (Cancel, back to the field) → **drawer** → **Class tool** (back to Select) → **selection**. While the field has focus, the catalog's canvas shortcuts (Ctrl+Z, Delete, …) stay with the field.

## Tests (written first)

Ordering-sensitive checks use `Harness::step()` before `run()`, and the helpers live in `tests/common/mod.rs`, including a `click_canvas_at` that takes a point relative to the canvas node's bounds.

**`tests/class_tool.rs`** *(new):*

- **Availability:** Class is enabled with an open diagram, and disabled with its reason once every tab is closed. Choosing it shows Class pressed and Select unpressed.
- **Clicking the canvas** shows the panel's field "New class name" focused, with "Class 1" fully selected. Typing "Invoice" then Enter gives one new class "Invoice":
  - its view is at the click point;
  - it is selected, and the tool is Select;
  - the undo label is "Add class", and one Undo removes it.
- **Enter with the unchanged default** creates "Class 1".
- **Normal text undo inside a fresh placement** does not touch the document.
- **Escape** creates nothing and leaves the revision unchanged. **Moving focus away** commits.
- **A blank name:** Enter, then a blur, then a click on the canvas each keep the error beside the field and keep focus there, and nothing is created. Fixing the text and pressing Enter creates the class.
- **Default names:** "Class 1" taken gives "Class 2".
- **Escape with the Class tool active and no placement** returns to Select and keeps the selection.
- **The keyboard path** uses real Tab presses (a bounded loop until the canvas node is focused), then Enter, with the pointer resting elsewhere: the panel is at a free visible spot.
- **Ctrl+Z inside the field** does not undo the document.
- **Stale and repeated commits:**
  - two commits for one placement in one batch create one class;
  - a commit after Cancel does nothing;
  - a commit naming an old placement ID does nothing;
  - `BeginNewClass` while a placement exists does nothing.
- **Undo and Redo, both orders** in one batch with a commit, including when nothing can be undone: the placement is cancelled, the commit skipped, and only Undo or Redo happens.
- **The preflight**, with intermediate frames checked using `step()`:
  - a tab click while naming creates the class in the first diagram, selected there, and then switches; closing the tab does the same;
  - Close project while naming creates the class, then the prompt opens;
  - Ctrl+K while naming, with no pointer interaction, creates the class, then the search opens;
  - with a **blank** name, a tab click, closing a tab, Close project and Ctrl+K each change nothing: the placement stays, the error and the notice show, and focus is back in the field;
  - clicking another class box while naming creates the class, then selects that box.
- **First frame after Begin:** Ctrl+Z and Escape sent before the panel has rendered once leave the history unchanged, and Escape cancels the naming.
- **A cold placement:** without the test focusing the field, the first interactive frame shows the field focused with "Class 1" fully selected.
- **A redundant Begin** keeps the placement ID, the draft, the revision and the history.
- **A duplicate commit followed by navigation or selection,** in both orders: the dialog is open, and the tab and selection are unchanged.
- **The first frame with an error at the bottom-right edge:** the panel's bounds are the same as without the error, and inside the canvas.
- **While the dialog asks:** the field queues nothing, and Escape closes only the dialog.
- **Geometry:** a click near the right and bottom edges keeps the class box and the whole panel inside the canvas, including the error state. A crowded diagram falls back to the clamped (48, 48). A long name stays on one line in the field.

**`tests/dialogs.rs`** *(new):* in "Order details", typing "Customer" opens the dialog with its words and three buttons, with focus on Cancel.

- **Show here:** there is no new element. Customer gains a view in "Order details" with its rendered size (it has members), and is selected. One Undo removes the view. Placed near the bottom edge, a class with many members is moved up to fit.
- **Keep new:** there are two classes named Customer. One Undo removes the new one.
- **Cancel, Escape and a click outside:** the document is unchanged, there is no click-through to the canvas, and the field holds "Customer" with focus. Five more stepped frames keep the dialog closed.
- **In "Class diagram 1":** "Show the existing class here" is disabled with its reason as visible text.
- **The keyboard path:** Tab moves between buttons and Enter activates.
- **A tab click whose commit opens the dialog:** the tab does not switch.
- **Name matching:** " Customer " opens the dialog; "customer" creates without asking.

**Pure tests** (in `tests/class_tool.rs`):

- `default_class_name`;
- `next_free_spot`: empty, a crowded first row, a full visible area, and occupied rectangles with members;
- `fit_inside`: the edges, a tiny canvas, and a footprint larger than the canvas;
- `decide`: no match, a match elsewhere, a match only here, several matches, trimming and case.

**Changed tests:**

- `tests/shell.rs::unimplemented_toolbar_actions_should_be_disabled` no longer expects Class disabled.
- `tests/commands.rs::queued_unavailable_commands_should_do_nothing` swaps `AddClass` for `Connect`.
- The contract tests point at the moved `ui/contracts.rs`.
- The showcase close test proves the queued close.

**`tests/ui_audit.rs`:** new scenes at 800×500, 1280×800 and 1440×900:

- the Class tool active;
- naming, near the bottom-right edge;
- naming with the error, near the bottom-right edge;
- the duplicate dialog in both variants.

**Proven to fail without the fix** (mutation script):

1. `decide` always returning `Create`;
2. the select-all of the default name;
3. the tool returning to Select;
4. the Escape order (tool before selection);
5. cancelling the placement on a history move;
6. the placement-ID and phase check (a repeated commit);
7. the preflight (a tab click while naming, and Ctrl+K);
8. focus returning to the field after Cancel;
9. "Show here" using `ShowInDiagram`;
10. treating `Naming` as text focus (Ctrl+Z in the first frame);
11. stopping the batch after an explicit commit that asks (a commit, then a tab click).

## Real window

Run the app on Windows at 1280×800 and 800×500, in light and dark. Choose Class, click the canvas, type a name and press Enter. Then place a duplicate, and open and answer the dialog. Save the screenshots to `app/docs/ui/p2-class-tool/`, with an evidence page `app/docs/ui/S5-CLASS-TOOL.md`.

## Gates and documents

- **Gates:** fmt, clippy `-D warnings`, test and doc from `app/`; the maintainability check (0 FAIL, WARN not above 7, and the size budget above); the roadmap `--check`.
- **`app/docs/ui/README.md`:** an S5 section.
- **`BUILD-PROGRAM.md`:** the S4 row becomes "Merged (PR #17)", and an S5 row is added.

## Not in this slice

- Marquee selection, grid snapping, drag to move and pan or zoom (S8, S12).
- Checking for a duplicate name when an existing class is renamed: D2 covers new classes only.
- Editing members (S6).
