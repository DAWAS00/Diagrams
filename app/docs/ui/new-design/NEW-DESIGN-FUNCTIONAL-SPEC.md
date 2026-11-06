# Diagram Studio — new design, functionality and shortcuts

Date: 2026-11-05. Status: **proposed implementation specification**, based on our native prototype and the [hands-on FigJam review](FIGJAM-BOARD-REVIEW.md). This file describes what we intend to build, not features already delivered. Shortcuts below are proposed Windows defaults and must be tested in the native app.

The product is an offline Rust desktop application for drawing diagrams with structured, reusable models. Its first complete release focuses on Class diagrams. The main experience is a spacious board, clearly named tools, a searchable project navigator and a stable contextual inspector. No account is required.

## Contents

1. [Scope and implementation status](#1-scope-and-implementation-status)
2. [Workspace and visual design](#2-workspace-and-visual-design)
3. [Project and navigation functions](#3-project-and-navigation-functions)
4. [Board and editing functions](#4-board-and-editing-functions)
5. [Classes, members and relationships](#5-classes-members-and-relationships)
6. [Inspector, appearance and validation](#6-inspector-appearance-and-validation)
7. [Saving, recovery and export](#7-saving-recovery-and-export)
8. [Keyboard shortcuts and focus](#8-keyboard-shortcuts-and-focus)
9. [Rust implementation structure](#9-rust-implementation-structure)
10. [Complete user workflows](#10-complete-user-workflows)
11. [Implementation order and acceptance](#11-implementation-order-and-acceptance)

## Synchronization note

Before publishing this design, the repository was updated to `5dc8977`, which adds theme/components, responsive panels, welcome/new-project presentation and a partial command catalog. The prototype row below records the earlier research baseline. Current implementation evidence and task status are maintained in [UI progress](../README.md) and the [roadmap](../../../../plan-rust-diagram-editor/roadmap.json). The proposed shortcut map is a design target: the current catalog uses Ctrl+K for search; changing that to Ctrl+Shift+P needs an explicit implementation decision and tests. This document does not override existing bindings automatically.

## 1. Scope and implementation status

| Scope | What belongs here |
|---|---|
| Existing prototype | Native egui shell, two preview class boxes, selection from canvas/list, inspector name editing and blank-name feedback. Data is prototype state, not the final command-driven project model |
| New v0.1 design | Project lifecycle, Class diagrams, members, association/generalization, multiple diagram views, board manipulation, contextual properties, history, validation, native persistence, recovery and SVG export |
| Later diagram milestones | Flowchart and Use Case; then ER, Activity, State, Requirements and Sequence. Each needs its own semantic rules and editing tests |
| Later product proposals | Notes/annotations, visual sections, advanced layout, rich clipboard interoperability, additional formats, collaboration, AI and interchange with other modeling tools |

Only implemented diagram families appear in the creation picker. The larger [comparison catalog](../../../../plan-rust-diagram-editor/comparison/README.md) is a research inventory, not a promise of supported notation.

Some earlier mockups show Note or other illustrative controls. For the v0.1 implementation, omit these until they have a scoped model, commands, storage and acceptance criteria. Similarly, automatic diagram layout, curved routing, arbitrary arrowhead selection and Figma-style collaboration are not implied by this design.

**Three concepts must stay visibly distinct:** a model is the shared meaning of an object; a view is its appearance and placement in one diagram; a viewport is the user's current pan and zoom. Renaming a model updates its views. Moving a view changes one diagram. Panning changes neither.

## 2. Workspace and visual design

```text
Project menu | Project name · Unsaved | Undo Redo | Commands | Save | Export
+--------------------+-------------------------------------+----------------------+
| Diagrams / Model   | Diagram tabs                    [+] | Inspector            |
| Search             | Select Hand Class Relationship      | Model / Appearance   |
|                    | Arrange                             |                      |
| Class overview     +-------------------------------------+ Name                 |
| Domain details     |                                     | Attributes           |
|                    |    Customer ----------- Order       | Operations           |
| Models             |                                     | Relationships        |
|   Customer         |        Large diagram board          |                      |
|   Order            |                                     | Scope: shared model  |
| [+ Diagram]        +-------------------------------------+                      |
|                    | Validation drawer, when opened      |                      |
+--------------------+-------------------------------------+----------------------+
Save state / path | Tool hint or selection count | Issues | Snap Grid | Zoom Fit
```

### Layout and visual rules

- Use the labeled top tool row as the implementation baseline. Compare a compact bottom dock in a prototype before changing its position; never display two duplicate toolbars.
- Header: 48 logical units; tabs: 38; tools: 40; status: 30. Navigator starts at 240 wide, inspector at 300; both resize and collapse. These are starting targets, not rigid screen-pixel values.
- At 1440×900 and 1280×800, show both panels where usable. At the supported 800×500 minimum, use labeled panel buttons, drawers and toolbar overflow. Do not cover essential controls or force hidden horizontal form scrolling.
- Preserve at least 480 logical units of canvas width where possible. When space is insufficient, collapse panels rather than shrinking text indefinitely. Closing a drawer returns focus to its opener.
- Light theme is default; dark theme uses the same hierarchy and control positions. Application theme does not rewrite diagram colors or exported artwork.
- Body text starts at 15 logical pixels, labels 14, panel titles 16. Use an 8-unit spacing rhythm and 4-unit internal gaps. Primary controls start at 34–36 high; small visible handles get larger picking areas.
- Use the semantic color tokens in [the existing UI specification](../../../../plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md#3-visual-specification). Color never carries selection, warning or error meaning alone: add outlines, icons and readable text.
- Common tools keep visible labels. Tooltips provide a short explanation and shortcut; essential information cannot depend on hovering.
- Font candidates remain Noto Sans and Noto Sans Arabic, subject to license, shaping, bidi and native input verification. A font asset alone does not resolve Arabic editing behavior.

### Surface responsibilities

| Surface | What it does | Implementation and state rules |
|---|---|---|
| Welcome | New/Open, recent projects, recovery choices | Recent paths live in user preferences; missing files have Locate/Remove actions. Remove removes the recent entry only |
| Header | Project identity, dirty state, history, command search, save/export | Render command availability and real worker state; do not show fake success |
| Navigator | Switch diagrams, find models, reveal views and reuse models | Diagrams and Model are separate tabs. Search results include enough location/type context to distinguish duplicate names |
| Diagram tabs | Switch open diagrams and add one | Keep viewport and selection per tab. Closing a tab does not delete a diagram or discard project changes |
| Tool row | Select, Hand, Class, Relationship and Arrange | Show the active tool and relationship kind. Unsupported tools are absent; temporarily unavailable tools explain why |
| Board | Draw, select, place, move, resize and connect views | Preview gestures in editor state; commit a validated command once at completion |
| Inspector | Edit selected model, view or relation | Stable panel with context-specific sections and explicit shared-model/this-view scope |
| Validation drawer | Explain issues and navigate to affected objects | Results carry session/revision identity; old results cannot appear as current |
| Status | Save state, selected count/tool instruction, issues, grid/snap and zoom | Save failures remain visible until resolved or acknowledged; status is accessible without color |
| Command search | Find actions, shortcuts and unavailable-action reasons | One catalog supplies the same action to menu, toolbar, context menu and shortcut |

## 3. Project and navigation functions

Every document change in these tables is one undoable transaction unless explicitly described as navigation, preference or file I/O. Detailed keyboard assignments are in section 8; actions without dedicated shortcuts are reachable through command search.

| Function | Exact user behavior | How we build it / edge cases |
|---|---|---|
| New project | Choose New, enter a nonblank name and create an empty Class diagram | Resolve current unsaved work first; initialize new typed project/diagram IDs. A new project is Unsaved until an actual save succeeds |
| Open project | Pick a `.dgram` file or a recent entry | Load and validate in a worker; replace the active session only after success and the dirty-project decision. Cancel or error preserves the current session |
| Recent projects | Reopen an existing path, locate a missing file, remove a recent entry | Store bounded recent metadata outside the project; do not store diagram bodies in preferences |
| Add diagram | Choose an implemented type, enter a name, open its tab | Create a distinct DiagramId; no new semantic class is created by merely adding a diagram |
| Rename diagram | Edit its title through navigator action or inspector | Validate a nonblank title and commit RenameDiagram; do not rename any model it displays |
| Close diagram tab | Close its visible workspace tab | Remove only tab state; document remains unchanged. If an edit draft is unresolved, resolve it before closing the tab |
| Delete diagram | Use an explicitly named navigator action and review affected views | Remove diagram/view records atomically, preserve shared model objects and semantic relations; undo restores its views |
| Switch diagram | Click a tab, use navigator or cycle tabs | Restore that tab's viewport and valid selection; never move objects to make the viewport match |
| Find object | Search names in the Model tab and show matching type/location | Search a derived index of the authoritative model. Empty/no-results states differ. Selecting a result does not rename it |
| Reveal object | Focus its existing view in a chosen diagram | If several views exist, choose the target. If none exists there, offer Show in diagram; do not silently create a view |
| Show in diagram | Choose an existing model and target compatible diagram, then place its view | Create a ViewId referencing the same ElementId. If that model already has a view in the target diagram, reveal it in v0.1 rather than creating ambiguous duplicates |
| Duplicate model | Create an independent copy of one selected class and its members | Generate new element/member/view IDs; copy member values and selected view style. Do not automatically duplicate incident relationships; explain this in the action description |
| Preferences | Change theme, interface scale and grid/snap defaults; open shortcut reference | Versioned per-user preferences. Theme/viewport/panel changes do not enter document history or mark the project dirty |

A diagram-delete impact summary counts views, not models being deleted. Global model deletion is a separate command with different consequences. If whole-diagram duplication is added later, specify whether it reuses models or copies them before exposing the action.

## 4. Board and editing functions

### Selection and movement

| Function | Interaction | Required result and implementation |
|---|---|---|
| Select | Click a node/relation, or select its entry in the object list | Highlight the corresponding ViewId and update inspector; picking follows visible stacking order and screen-sized tolerance |
| Multi-select | Shift-click toggles membership; drag an empty-area selection rectangle | Default rectangle selects fully enclosed node views; Shift adds to selection. Relations can be selected individually without also selecting endpoints |
| Deselect | Click empty space or press Escape after resolving any draft/gesture | Clear selection only; no document mutation |
| Select all | Use Select all with canvas focus | Select all editable node and edge views in the active diagram; text fields retain their own Select all behavior |
| Pan | Hand tool drag, middle-button drag or temporary Space+drag | Change viewport only. A drag starting in Hand mode cannot move a node |
| Zoom | Visible controls, shortcut or Ctrl+wheel over canvas | Clamp proposed range to 10–400%; cursor anchors pointer zoom, canvas center anchors keyboard zoom. Validate finite invertible transforms |
| Scroll | Wheel over canvas pans vertically; Shift+wheel pans horizontally | A scrollable panel consumes its own wheel events. Do not accidentally pan behind inspector controls |
| Fit all / selection | Use Fit menu or keyboard | Compute padded visible bounds; never modify view geometry. Empty diagram resets to a useful centered viewport; empty-selection Fit selection is unavailable |
| Create class | Choose Class, click placement position, enter its name | Hold an uncommitted placement/name draft. Enter with valid name creates model+view together; Escape beforehand creates nothing. Return to Select after one creation |
| Keyboard placement | Choose Class and use arrow keys to move preview, then Enter to name it | Start at viewport center; show coordinates and a visible preview. Same commit path as pointer placement |
| Move | Drag selected nodes or nudge with arrows | Preview all selected node bounds; commit once on release/key gesture completion. Refresh incident edge routes and preserve endpoint identity |
| Resize | Drag a handle or edit Width/Height in Appearance | Enforce content-aware minimum sizes and positive finite geometry. Text remains readable; reject invalid geometry without changing the document |
| Cancel gesture | Escape, lost pointer capture or interrupted gesture | Discard preview and restore original geometry; no partial history entry |
| Grid / snap | Independent status switches | Grid affects display; snap affects placement/movement. Start with a 16-world-unit grid. Numeric geometry entries and keyboard nudges are exact; snapping applies to pointer placement/drag |
| Align | Arrange → left/center/right/top/middle/bottom | At least two node views required. Align to selected bounds (center uses selection center); one undo restores all positions |
| Distribute | Arrange → horizontal/vertical equal gaps | At least three node views required; keep outer bounds fixed, divide remaining space equally. Disable and explain when widths/heights leave negative available gaps |
| Remove view | Delete or menu action on canvas selection | Remove selected views and dependent edge views in this diagram; keep their semantic models/relations. Undo restores geometry and routing |
| Delete model everywhere | Explicit Model action, then impact review | List affected model/member/relation/view counts across diagrams. Apply deletion atomically; undo restores the complete affected graph |
| Undo / redo | Named controls show the pending action label | One history entry per accepted user action. New document edit after Undo clears redo; view navigation does not |

Straight and simple orthogonal relationship routes are supported in the first design. Orthogonal means horizontal/vertical segments; it does not promise automatic obstacle avoidance. Route hit testing and visible invalid-target feedback belong to the canvas layer.

For keyboard movement, use 1 world unit per arrow and 10 with Shift. A held key becomes one coalesced move for the same selection until release; a selection or modifier change ends that group. Numeric inspector fields provide the non-drag resizing route. Edge-only selection does not enable node movement or resize.

### Interaction states

Use explicit states for Idle, Placement, Pan, Marquee, MovePreview, ResizePreview, ConnectSource, ConnectTarget and InlineEdit. A tool switch cancels an uncommitted geometric preview. It must first resolve an unresolved text draft under the policy below. Do not combine contradictory flags such as dragging, connecting and editing simultaneously.

## 5. Classes, members and relationships

### Class and member functions

| Function | Fields / behavior | Command and validation contract |
|---|---|---|
| Rename class | Name in inspector or inline editor; shared-model scope label | Nonblank Unicode name; preserve spelling. RenameElement updates every view of that model after commit |
| Add attribute | Name, type text, visibility, optional multiplicity | Stable MemberId; nonblank name, supported visibility, parse multiplicity if supplied. Type text is descriptive in v0.1, not a programming-language type checker |
| Add operation | Name, visibility, ordered parameters and optional return type | Operation and parameter data commit together. An owned expanded editor handles details without losing the selected class |
| Edit parameter | Name, type text; add/remove/reorder rows | Keep order and draft IDs stable during editing; validate the operation before committing the compound change |
| Edit member | Open row editor from Members | Name/type/visibility and supported optional fields become one accepted edit; invalid values stay visible with a specific explanation |
| Reorder members | Move up/down buttons, optional drag enhancement later | Change ordered member IDs; stable IDs preserve identity. Buttons give a complete keyboard route |
| Remove member | Explicit row action | Remove that member and any supported dependent member anchors atomically; show impact if dependencies exist. Do not delete the owning class |
| Show/hide compartments | Appearance → Attributes / Operations | Change this view's presentation only; hidden members remain in the model and other views |

Supported visibility choices are public, private, protected and package, displayed with readable names and their notation symbols. Advanced flags, stereotypes, constraints and full UML conformance are outside the initial subset unless separately specified. Duplicate names are distinguished by identity/location; do not invent source-code uniqueness rules for descriptive modeling names.

### Relationship functions

| Function | User interaction | Model/renderer behavior |
|---|---|---|
| Association | Relationship → Association; choose source and target | Create RelationId plus EdgeView atomically, with class endpoint IDs separate from visual anchors |
| Generalization | Choose child first, parent second | Store child → parent; draw the generalization marker at parent. Reject self-generalization and inheritance cycles before commit |
| Pointer connection | Drag from a connection affordance, or click source then target | Both routes share validation and preview. Valid/invalid targets have text/icon feedback; invalid completion creates nothing |
| Keyboard connection | Open endpoint chooser, select kind/source/target, activate Create | Searchable lists show class names and locations. Require endpoint views in the active diagram or offer explicit Show in diagram first |
| Association ends | Source/target class, role names, optional multiplicities | Inspect each end separately; support blank, nonnegative integer, `*`, and `lower..upper` with upper integer or `*`. Reject inverted/invalid ranges |
| Change endpoint | Relation inspector → endpoint picker | This changes shared semantic meaning; show affected diagrams. Update compatible edge views; block with explanation when another view cannot resolve an endpoint, until explicitly resolved |
| Route style | Straight or orthogonal in Appearance | View-local route change; semantic type and endpoints remain unchanged |
| Adjust attachment | Auto or edge/side position through route controls | Store view anchors, not raw screen positions. Movement recalculates attached routes; impossible geometry is reported, never serialized as NaN |
| Remove relation view | Select connector and Delete | Remove only its EdgeView; keep semantic relation and any other views |
| Delete relation everywhere | Explicit semantic action with affected-view summary | Remove relation and all associated EdgeViews in one reversible command |

Association and generalization do not share identical validation. For v0.1, self-association is deferred until a readable loop renderer exists; explain that limitation instead of labelling it invalid UML. Reverse-direction generalization is a semantic endpoint edit and must rerun cycle checks. Cosmetic arrowhead changes must never silently change relation type.

## 6. Inspector, appearance and validation

| Selection | Inspector contents | Scope and behavior |
|---|---|---|
| None | Diagram name, grid/snap controls, short creation guidance | Diagram rename is a document command; grid/snap are editor preferences |
| One class | Model: Name, Attributes, Operations, related models. Appearance: position, size, fill, border, compartment visibility | Persistent labels explain “Shared model — affects all views” versus “This diagram view” |
| One relationship | Kind summary, named endpoints, relevant end properties; Appearance: route/attachment | Kind-specific fields only. Association roles/multiplicity are absent for generalization |
| Multiple nodes | Selection count, shared appearance fields, align/distribute | Differing values display Mixed. Editing one field updates only that field for supported selected views in one command |
| Mixed nodes/edges | Selection count and genuinely common supported actions | Do not display irrelevant member or resize fields; explain why a requested action is unavailable |

Initial appearance controls are a small fill/border color palette, border width and class compartment visibility, plus numeric geometry. Apply these to the view. Reject colors/widths that cannot be represented or rendered; avoid a broad effects panel before the basic workflow is usable.

### Draft and focus policy

1. Text input edits a draft, not the authoritative model on every keystroke. Labels show whether a change is committed.
2. Enter commits a valid single-line edit. Expanded member editors use Apply/Cancel; Enter in a multiline field keeps its text behavior.
3. Escape cancels the current draft and restores its original value. It must not simultaneously clear selection or close the project.
4. A valid single-field draft commits when leaving the field. Invalid drafts remain visible and uncommitted; selection/tab/tool changes that would lose them show Resolve edit with Return to field / Discard edit.
5. Save, export and close resolve outstanding drafts first, so the visible name is not silently omitted from the saved snapshot. Cancellation preserves the draft and current document.
6. Text-field Undo affects draft text. After commit, document Undo reverses the whole accepted edit. Losing focus does not create one command per character.

### Validation and messages

Domain commands validate before mutation; a rejected action explains the exact rule beside the relevant field/preview. Whole-document checks populate a drawer with severity, message, object location and Go to object. Clicking an issue reveals/selects an existing view or opens the model inspector if it has none. Mark old results as checking/stale until current results arrive.

Distinguish blocking structural errors (dangling identity, invalid geometry, unsupported schema) from nonblocking modeling warnings. Save/export policies must say which category blocks the action. A nonblocking warning does not silently prevent saving. Keyboard focus moves to the first failing required input on explicit Apply, but background validation does not steal focus.

Loading, empty, no-results, unavailable, pending and failed are separate states. An unavailable command stays discoverable in command search with its reason; unsupported product features stay out of the active creation UI. Success messages do not replace persistent save errors.

## 7. Saving, recovery and export

| Function | What the user sees | Implementation and failure behavior |
|---|---|---|
| Save | Unsaved / Saving / Saved / Newer changes unsaved / Save failed, with path | Send an immutable validated snapshot to a storage worker. Acknowledgment identifies session, request and saved state; newer edits remain dirty |
| Save As | Native destination dialog and overwrite decision if needed | Publish a completed validated temporary database using a tested replacement procedure. Switch current path only after success; cancellation leaves it unchanged |
| Open failure | Readable path/schema/data error with a way to choose another file | Preserve current session and source file. Reject unsupported newer schemas without rewriting them |
| Save failure | Persistent explanation plus Retry and Save As | Keep document/history in memory; preserve prior committed file state. Never say Saved because a request was merely queued |
| Close project/app | Save / Discard / Cancel when unsaved | Save waits for the relevant successful result before close. Failure or a newer unsaved state blocks automatic close. Discard is explicit |
| Recovery | Startup item with project source and snapshot time; Restore / Ignore | Recovery snapshot is separate from explicit save, proposed after 30 seconds of dirty activity. Restore opens recoverable state without silently overwriting the source; explain any future snapshot deletion separately |
| Export SVG | Current diagram, content bounds/background, destination and preview summary | Export a stable validated scene snapshot without selection/grid/handles. Escape XML correctly; preserve labels, compartments and relation markers |
| Export result | Actual success path or persistent error with retry | Worker writes output safely; export never changes semantic history or marks the project saved. Record font dependency/fallback limits |

The native file is a proposed `.dgram` SQLite schema, independent of Visual Paradigm files. “Validated XML” in the earlier scope means well-formed SVG XML output here; a separate model-XML interchange format is not part of this UI release.

## 8. Keyboard shortcuts and focus

These are **proposed defaults, not currently implemented guarantees**. Show them in menus, tooltips, command results and a searchable Help → Keyboard shortcuts reference. A user can reach every essential function through named controls even without memorizing a binding. Full shortcut customization is later scope.

### Application and navigation

| Shortcut | Action | Availability / behavior |
|---|---|---|
| Ctrl+N | New project | Resolve current drafts/unsaved project before replacement |
| Ctrl+O | Open project | Open the native file chooser |
| Ctrl+S | Save | Resolve drafts; first save asks for a path |
| Ctrl+Shift+S | Save As | Save current project to a chosen destination |
| Ctrl+Shift+E | Export SVG | Active exportable diagram required |
| Ctrl+Shift+P | Search commands | Focus the command palette; show unavailable reasons |
| Ctrl+F | Find model/object | Focus navigator search when not inside a text editor that owns Find |
| Ctrl+Shift+N | Add diagram | Open notation/name creation form |
| Ctrl+Tab | Next diagram tab | Only when no text control/modal owns this key combination |
| Ctrl+Shift+Tab | Previous diagram tab | Same focus rule as Next diagram |
| Ctrl+W | Close active diagram tab | Does not delete diagram or close project |
| Ctrl+, | Preferences | Open theme/scale/defaults settings |
| F1 | Help and shortcuts | Context-sensitive help if available; otherwise searchable reference |
| F6 | Next workspace region | Cycle header, navigator, board/object list, inspector, open validation drawer and status; skip hidden regions |
| Shift+F6 | Previous workspace region | Reverse the same focus sequence |
| Tab / Shift+Tab | Next / previous control | Stay inside the active modal when one is open |
| Shift+F10 | Context menu | Open actions for focused object/control, with a visible menu-button alternative |

### Canvas and edit commands

Single-letter and digit tool shortcuts below operate only with canvas focus, no text entry, no IME composition and no modal. They must never activate while naming a class, searching, or editing members.

| Shortcut | Action | Availability / behavior |
|---|---|---|
| V | Select tool | Cancel geometric preview safely and activate Select |
| H | Hand tool | Subsequent canvas drags pan |
| C | Place Class | Start a preview at viewport center; click or use arrows/Enter |
| R | Relationship tool | Use current supported kind; named chooser changes kind. Keyboard path opens endpoint selection |
| Space+drag | Temporary pan | Restore previous tool after release; do not intercept text spaces |
| F2 | Rename selected model | Exactly one renameable selected item; navigator diagram item instead renames that diagram |
| Enter | Edit/activate selected item | Open selected object's main editor, or advance placement, according to active state |
| Escape | Cancel innermost action | Modal/draft → gesture/tool → selection, one layer per press |
| Shift+click | Toggle selection membership | Canvas selection only |
| Ctrl+A | Select all active-diagram views | Text controls retain Select all text |
| Delete | Remove selected views | Canvas only; text deletion remains local. No default global-model deletion shortcut |
| Arrow keys | Nudge selected node views by 1 unit | In placement mode move preview; in list/text controls navigate normally |
| Shift+Arrow keys | Nudge by 10 units | Same focus rules; text selection remains local |
| Ctrl+Z | Undo | Text draft first; otherwise project history |
| Ctrl+Shift+Z or Ctrl+Y | Redo | Text draft first; otherwise project history |
| Ctrl++ | Zoom in | Accept the plus key with its required keyboard-layout modifiers and numpad Add |
| Ctrl+- | Zoom out | Also accept numpad Subtract |
| Ctrl+0 | Actual size, 100% | Preserve canvas-center world position |
| Shift+1 | Fit all | Include diagram content, exclude editor overlays |
| Shift+2 | Fit selection | Requires selected visible content |
| G | Toggle grid | Display preference only |
| Shift+G | Toggle snap | Placement/movement preference only |

Add attribute, Add operation, Show in diagram, Duplicate model, Delete model everywhere, Align, Distribute and validation navigation use named controls and command search initially. Avoid assigning obscure shortcuts to every action. Command search accepts terms such as “attribute,” “reuse,” “fit,” and “issues,” while displaying one canonical action label.

No diagram Ctrl+C/Ctrl+V behavior is promised in v0.1; ordinary text copy/paste remains available. Later clipboard support must specify when a paste reuses identities versus creates independent objects.

### Input dispatch and accessibility

Priority is active native/modal dialog → IME/focused text control → transient gesture → focused region → applicable application command. An input event is consumed once; matching Ctrl+S must not also trigger a single-letter tool. Only regions that own an action may handle it. Native dialogs use their standard keys.

Expose meaningful control names, roles, selected/expanded/disabled states and unavailable reasons. Provide a keyboard-accessible object list containing nodes and relations, named endpoint pickers, numeric geometry fields and member reorder buttons. Pointer hover and drag are enhancements, not the sole route.

Test with Narrator/NVDA, English/Arabic text, IME composition, 100/150/200% scaling, high contrast and small windows. Shortcut choices and screen-reader compatibility remain unverified until those checks pass. No accessibility-complete claim follows from using egui or AccessKit alone.

## 9. Rust implementation structure

Continue the existing four-crate workspace and locked eframe/egui stack; do not introduce a web frontend for this design. The GUI decision remains subject to the existing R0-04/R0-05 native text/accessibility gate. See [engineering rules](../../../../plan-rust-diagram-editor/RUST-ENGINEERING.md) and [architecture](../../../../plan-rust-diagram-editor/02-ARCHITECTURE.md).

| Owner | Proposed responsibilities | What must stay outside it |
|---|---|---|
| `diagram-core` | Typed IDs, classes/members/relations, diagram views, commands, validation, reversible patches and history | UI widgets, file dialogs and SQLite connections |
| `diagram-canvas` | World/screen transforms, geometry, picking, gesture calculations, scene construction, routes and SVG scene output | Owning a second writable document or saving files |
| `diagram-storage` | Versioned schema, snapshot validation/serialization, transactions, migrations and recovery-file operations | Interpreting pointer input or rendering widgets |
| `diagram-desktop` | Active session, editor state, egui components, focus, command dispatch, worker coordination and OS integration | Reimplementing domain rules separately in each panel |

### Proposed desktop modules

| Module | Responsibility |
|---|---|
| `ui/theme.rs`, `ui/components.rs` | Semantic colors, spacing, typography, labeled actions, field errors and section components |
| `ui/workspace.rs`, `ui/welcome.rs` | Responsive panel layout, entry/recovery screens and region focus |
| `ui/navigator.rs`, `ui/toolbar.rs` | Model/diagram navigation, tools, availability and tab actions |
| `ui/inspector.rs` | Selection-specific forms, shared/view scope and controlled draft editors |
| `ui/dialogs.rs`, `ui/validation.rs` | Owned dialog state, impact review, failure/recovery and issue navigation |
| `ui/commands.rs` | Action IDs, labels, search aliases, shortcut specifications, prerequisites and dispatch metadata |
| `editor_state.rs` | Per-tab viewport/selection, active gesture/tool, draft buffers and focus return target |
| `presentation.rs` | Read-only presentation data derived from the authoritative document |

These are target modules to introduce when implementing behavior, not instructions to create empty scaffolding now.

### One action pipeline

```text
Toolbar / menu / keyboard / command search / canvas gesture
                            |
                     Typed UiIntent
                            |
                Availability + draft resolution
                            |
          +-----------------+------------------+
          |                 |                  |
   Editor-state action  Domain command    Worker request
   pan/focus/select     validate + patch  open/save/export
          |                 |                  |
          |           history + revision   typed result
          +-----------------+------------------+
                            |
              Derived scene and UI presentation
```

- Define owned intent/command enums and explicit availability (`enabled` or `disabled with reason`); do not pass mutable document references into widgets or background jobs.
- Document changes validate and apply atomically. History stores complete forward/inverse patches and action labels; eviction never splits a transaction. Follow the existing history budget specification.
- Editor previews hold original bounds and temporary values. Pointer release produces one command; cancellation produces none.
- Store typed `ElementId`, `MemberId`, `RelationId`, `DiagramId` and `ViewId`. Scene hit targets map to these IDs; vector indices are not persistent identity.
- Keep model/style/viewport changes separate. Dirty state compares the current document StateId with the successfully saved StateId; a monotonically increasing revision also detects stale worker results. Undo back to a saved state can become clean.
- Use `screen = canvas_origin + pan + world * zoom`, with pan in screen units, for display; use a checked inverse for picking. Zoom remains positive/finite, and hit tolerance stays useful at every scale.
- Generate diagram display and SVG from a shared renderer-neutral scene. Selection outlines, handles and grid are a separate overlay layer.
- The UI thread owns session/document/editor state. A bounded worker channel handles storage/export; save acknowledgments include session/request/state identity. Painting performs no blocking database or file-dialog work.
- Use borrowed read-only presentation data, explicit error types and small concrete enums. Avoid a global shared mutable document or a generic plugin framework before notation needs justify one.

## 10. Complete user workflows

### First two connected classes

1. New project opens an empty Class diagram with an Add Class prompt.
2. Activate Class, place Customer, enter its name and commit.
3. Repeat for Order. Select Customer; add `id: String` in Members.
4. Choose Relationship → Association; choose Customer then Order. Keyboard users use the same command's endpoint picker.
5. Set Customer-end multiplicity `1` and Order-end multiplicity `0..*` in the relation inspector.
6. Fit all and save. The status shows Saved only after the actual write succeeds.

The usability target is two connected named classes within two minutes for a first-time user, subject to actual user testing. The target is not current measured performance.

### Reuse without confusing copies

1. Add a second Class diagram and choose Customer → Show in diagram.
2. Rename it Client. Both diagrams show Client because they share ElementId.
3. Change its fill and position in the second diagram. The first view keeps its appearance and geometry.
4. Choose Duplicate model to create an independent class; wording states that relationships are not duplicated.
5. Delete a selected view to remove it from that diagram only. Use Delete model everywhere only after reviewing its wider impact.

### Keyboard-only creation

1. Ctrl+N creates a project through accessible fields.
2. F6 reaches the board; C starts placement, arrows position it, Enter enters naming, Enter commits.
3. Repeat for another class; R opens the keyboard relationship path and named endpoint selectors.
4. F6 reaches inspector; Tab navigates fields and member actions; Escape cancels a draft without erasing the class.
5. Ctrl+S saves; Ctrl+Shift+E opens export. Every action also has a named control or command-search entry.

### Invalid edit and failed save

1. Clear a class name and attempt to leave. Show the nonblank-name error and resolve the draft; keep the committed model intact.
2. Attempt a generalization cycle. Explain why it fails; create no relation and no history entry.
3. Fail a save in a controlled test. Preserve the previous disk state and current in-memory changes; show Retry/Save As.
4. Request close. Cancel keeps the project open; Save cannot close it until the relevant save succeeds; Discard is an explicit choice.

## 11. Implementation order and acceptance

Use the existing [roadmap](../../../../plan-rust-diagram-editor/roadmap.json) as task/status authority. This document refines behavior and does not mark tasks complete or introduce new milestones. Newly clarified details such as diagram deletion and keyboard placement belong in the listed tasks' acceptance criteria when implemented.

| Order | Existing tasks | Concrete deliverable / acceptance |
|---|---|---|
| 1 | R0-04/R0-05; UI-01/UI-02 | Pass native text/input/accessibility feasibility; establish tokens, typography and component examples |
| 2 | UI-03–UI-06 | Responsive shell, welcome, navigator/tabs and one action catalog; compare top tools versus bottom dock using identical workflows |
| 3 | UI-07–UI-11 | All canvas/inspector/dialog states as clearly labeled presentation fixtures; reviewed focus, shortcut and notation-capability contracts |
| 4 | UI-12 | Native UI review at target sizes/scales; no false save/undo behavior in the shipped prototype |
| 5 | M1 tasks | Typed document, validation, transactional commands/history and actual storage foundations |
| 6 | UI-13–UI-17 with M2 tasks | Bind real navigation, inspector, canvas/history and save/validation; complete pointer and keyboard Class walkthroughs |
| 7 | UI-18 with M3 tasks | Verify actual save/reopen, SVG, recovery, failure paths, scaling and accessibility before release |

### Required evidence

- **Core:** rejected commands leave state unchanged; delete/duplicate/reuse preserve reference integrity; undo/redo restore complete transactions and shared-name propagation.
- **Canvas:** transformed picking works across zoom/pan; canceled gestures do not commit; movement preserves attached endpoints; keyboard placement and pointer placement yield equivalent valid objects.
- **Storage:** round-trip IDs, member order, relation ends, geometry and style; failures preserve previous data; stale results cannot replace a different session or falsely clear dirty state.
- **Native UI:** menus, shortcuts and search invoke the same action; IME/text input never triggers canvas tools; drawers/modals restore focus; mixed edits change only requested fields.
- **Usability:** observe first-time users creating, connecting, reusing, removing a view and recovering from error. Record hesitation, time and mistakes before claiming improvement over Visual Paradigm.
- **Accessibility and layout:** exercise real assistive technology, Arabic/English input, keyboard-only workflows, 800×500/1280×800/1440×900 and 100/150/200% scaling. A static mockup is not evidence of native success.

The release design is complete only when these behaviors work against real document/storage state. Until then, mockups and this specification remain design artifacts.

## Related planning files

- [UI design and integration baseline](../../../../plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md)
- [Interactive layout preview](../../../../plan-rust-diagram-editor/UI-DESIGN.html) — earlier visual concept, not a full implementation of this specification
- [FigJam interaction findings](FIGJAM-BOARD-REVIEW.md)
- [UI research and usability protocol](UI-RESEARCH.md)
- [28-action discoverability map](UI-FUNCTION-MAP.csv)
- [Architecture and persistence contracts](../../../../plan-rust-diagram-editor/02-ARCHITECTURE.md)
- [Detailed roadmap](../../../../plan-rust-diagram-editor/RUST-ROADMAP.md)
