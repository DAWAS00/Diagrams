# 07 — Complete native UI plan

Status 2026-11-05: UX milestone started; UI-01 is done ([UI progress](../app/docs/ui/README.md)). The running app is the R0-02 prototype (toolbar, sidebar, canvas with two sample classes, inspector, status bar). This document specifies every surface the product needs, how each connects to the model that M1 will build, and the tasks that deliver it. [03-GUI-AND-WORKFLOWS.md](03-GUI-AND-WORKFLOWS.md) remains the interaction direction; this plan makes it complete and testable. [ADR-001](decisions/ADR-001-GUI.md) stays conditional until R0-05.

## 1. Goal and boundaries

Deliver a desktop editor where a new user can create a class diagram, name and edit members, connect classes, reuse a class in another diagram, undo mistakes, save, reopen, export SVG and recover from a failed save, without reading documentation. The UI is designed in full **before** M1 so the model, commands and storage are built against real screens, then bound to them in M2.

In scope: the complete v0.1 Class workflow and the shared shell for later notations. Out of scope until separately planned: other diagram families' editors, collaboration, plugins, web or mobile UI.

## 2. Decisions and dependency boundaries

| Area | Decision | Notes |
|---|---|---|
| Framework | eframe/egui 0.37.0 with wgpu | Conditional on R0-05. Compare Iced/Slint only if a gate item fails |
| Layout | Fixed resizable panels, no docking | Per ADR-001 |
| Tests | `egui_kittest` 0.37.0 for event and accessibility-tree behavior, plus real Windows checks. Add small reviewed image fixtures only where useful | Image snapshots need extra `egui_kittest` features that pull in more dependencies; enable them when the first image test is written |
| Accessibility | Existing AccessKit integration (`accesskit` 0.25 via eframe's default feature), named controls, keyboard object-list alternative, meaningful selection and relation descriptions. Real Narrator/NVDA testing remains required | Checked: the headless accessibility text for a mixed-text name differed from the editable value (Windows review 2026-11-05) |
| Fonts | System-font fallback is already active (eframe default `system_fonts`). Candidate bundled Noto Sans + Noto Sans Arabic; verify exact files, version, metrics and notices before adding assets. Arabic upstream publishes OFL-1.1 | **egui 0.37 shapes text with HarfRust but has no bidirectional reordering** (its source notes bidi as future work). A font cannot fix mixed Arabic/English order. R0-04 must test this before UI-02 fixes any text rules. If it fails, the ADR-001 revisit trigger applies |
| Icons | Original simple vector line icons drawn with the egui painter, plus visible labels for common tools | SVG icon files would need `egui_extras` with its `svg` feature (pulls in `resvg`). Decide explicitly if wanted. Preserve notices for any external icon source |
| Native file dialogs | `rfd` when real Open/Save is implemented (0.17.2 resolved today, MIT, declares no minimum Rust version, so compile it on 1.99.1 when added). Cancel is a normal outcome | Windows uses Win32 dialogs; the default Linux portal features are irrelevant on Windows |
| Preferences | Versioned per-user theme/panel/scale/recent-file settings, separate from the project and dirty state; no project bodies in preferences | Needs serialization. Choose at implementation time between eframe's `persistence` feature (adds `ron` and `serde`) and `serde_json`. This is a new dependency, so the choice is reviewed then |
| Persistence | Existing `rusqlite` (bundled SQLite 3.54.1) for project storage, executed by a worker; the UI never performs a database save during painting | Verified on Windows MSVC (R0-03) |

No framework change, runtime, new dependencies or asset download occurs in this planning update. Avoid adding docking, animation, webview or chart frameworks without a demonstrated need.

Official references checked: [eframe](https://docs.rs/eframe/0.37.0/eframe/), [egui source](https://github.com/emilk/egui), [rfd](https://docs.rs/rfd/latest/rfd/), [Noto Arabic and its license](https://github.com/notofonts/arabic). Verify exact APIs and fonts again at implementation time.

## 3. Visual specification

Use semantic tokens rather than scattered RGB constants. Tokens cover background, surface, raised surface, canvas, text, secondary text, border, accent, selection, focus, warning and danger. The shell currently follows the operating system theme; tokens must be applied explicitly through egui's visuals, with a user preference for light, dark or system. Start with:

| Token | Light target | Dark target |
|---|---|---|
| Workspace / panels | #F4F6FA / #FFFFFF | #141A24 / #1D2633 |
| Canvas | #FCFCFD | #202B39 |
| Primary text | #182438 | #EDF2F8 |
| Secondary text | #526078 | #B6C2D3 |
| Accent / focus | #245CC5 | #91B8FF |
| Border | #D9E0EB | #465469 |
| Warning / danger text | #8A4B00 / #B42318 | #FFD08A / #FFB4AD |

These are starting design values. Target 4.5:1 for normal text and 3:1 for large text and essential control boundaries, with labels, icons or patterns supplementing color.

**Initial contrast check (computed with the WCAG formula, 2026-11-05).** Ratios of each token against workspace / panel / canvas:

| Token | Light | Dark | Result |
|---|---|---|---|
| Primary text | 14.4 / 15.6 / 15.2 | 15.5 / 13.6 / 12.7 | Pass |
| Secondary text | 5.9 / 6.4 / 6.2 | 9.7 / 8.5 / 7.9 | Pass |
| Accent / focus | 5.7 / 6.2 / 6.0 | 8.7 / 7.6 / 7.2 | Pass (also meets 3:1 for focus rings) |
| Warning text | 6.3 / 6.8 / 6.6 | 12.2 / 10.6 / 10.0 | Pass |
| Danger text | 6.1 / 6.6 / 6.4 | 10.3 / 9.0 / 8.5 | Pass |
| **Border** | **1.2 / 1.3 / 1.3** | **2.3 / 2.0 / 1.9** | **Fails 3:1 in both themes** |

The border token is acceptable for decorative panel separators but **not** for input fields, buttons and other essential control boundaries. UI-01 defines a stronger control-border token and keeps the light border only for separators. **Done:** `#8A97AB` was tried and also failed (2.6–3.0:1); the implemented values are `#76839A` (light, 3.3–3.8:1) and `#6E7F98` (dark, 3.1–4.3:1). See [UI-01 evidence](../app/docs/ui/UI-01-TOKENS-AND-COMPONENTS.md). Disabled, hover, selected and pressed states, and text on the accent color, are still unchecked and belong to UI-10.

Use an 8-unit rhythm, with 4-unit internal gaps; 6-unit control radii and 8-unit dialog radii. Default body text 15 logical pixels, labels 14, panel titles 16, page titles 24. Code/member lines use a readable monospace fallback at 14. Starting control heights: 34–36; hit targets at least 32 units, handles visually smaller but with larger hit regions. No motion is necessary for core workflows; respect reduced-motion settings if transitions are added.

Target 1440×900 and verify 1280×800. Header 48, diagram-tab row 38, tool row 40, status row 30; left panel 240 (range 180–360), inspector 300 (240–420). Panel dimensions are logical units. At narrower widths, collapse the navigator and open the inspector as a dismissible drawer; keep at least 480 units of usable canvas when possible. At the supported 800×500 minimum, use labeled panel buttons and toolbar overflow, never overlapping controls. Restore focus after closing a drawer. Test minimum sizes at 100/150/200% OS scale and monitor changes.

Canvas document styling is separate from chrome theme. Export defaults to a declared light document background; changing the application theme must not silently recolor saved diagram objects or change export meaning. Grid, handles and selection highlights are editor overlays and excluded from export.

## 4. Every surface and state

| Surface | Required content and behavior |
|---|---|
| Welcome | New/Open, searchable recent paths with missing-file handling, recovery entry when present. No account or template wizard required |
| New project | Name, location when saving, implemented diagram-type selector, validation beside fields. Open an empty diagram; unsupported types absent from creation |
| Header | Project menu, name/path hint, dirty marker, named Undo/Redo, command search, Save, Export. Disabled actions explain their prerequisite |
| Navigator | Diagrams/Model tabs, search, hierarchy, new/rename/duplicate, keyboard navigation, reveal selected model/view. Distinguish no items from no search results |
| Diagram tabs | Title, dirty indication where applicable, close, overflow, add; preserve per-tab viewport/selection drafts. Project dirty state is project-wide; closing a diagram tab does not discard model changes |
| Tool row | Select, pan, notation-specific creation/connection tools, note, arrange. Persistent active-tool indication; Escape cancels a preview; tool changes cancel safely |
| Canvas | Blank, none selected, one node, relation, multi-selection, inline edit, move/resize/connect preview, invalid target and read-only states. Pan/zoom/fit; snapping with visible feedback |
| Inspector | No selection: help/diagram settings. Node: Model, Appearance. Relation: endpoints, roles, multiplicity and routing. Multi-select: common fields and explicit mixed values. No silent replacement of mixed values |
| Members | Ordered attributes/operations, add/edit/remove/reorder, name/type/visibility, validation, keyboard row actions; advanced parameter details in an owned editor surface |
| Context menu | Relevant actions mirrored in keyboard/toolbar/palette; remove view separated from Delete model everywhere. Keep primary workflows discoverable without right-click |
| Command palette | Search commands and objects, shortcut hints, capability/selection prerequisites, reason for unavailable commands; focus restored on cancel |
| Validation drawer | Severity, readable issue, affected model/view, Go to object, next action; stale results are discarded. Errors use text and icons, not color alone |
| Save/open/recovery | Loading, cancel when safe, saving, saved, newer edits still dirty, failed save with Retry/Save As, dirty close Save/Discard/Cancel, recovery source/time and explicit restore/ignore choices |
| Export | Current diagram, SVG, bounds/background, destination, preview summary, progress/success/failure; unsupported formats not presented as working |
| Settings | Theme, text/UI scale, grid/snap defaults, shortcuts reference, about/version/licenses. Preferences never mutate diagram semantics |
| Status | Actual save state/path, selection count or short active-tool hint, validation count, grid/snap, zoom and Fit. No transient success toast in place of a persistent save error |

Transient messages announce success or nonblocking information without stealing focus. A modal is reserved for destructive impact review, close decisions and required input; ordinary property edits stay inline. Native-dialog cancellation leaves selection/document unchanged.

## 5. Interaction contracts for later functionality

Each visible action has an intent, prerequisites, effect owner, pending/success/error states, undo policy and test. UI components receive borrowed presentation data and emit owned intents; no callbacks carry mutable document references into workers.

| Action | Model effect / integration | Failure and undo |
|---|---|---|
| Create class | CreateClass creates semantic object + view atomically; placement draft first, name field focused | Escape before commit creates nothing; one undo reverses the committed action |
| Rename | Draft → validated RenameElement; all views update after commit | Invalid draft stays editable with explanation; cancel preserves original; text undo belongs to the field until commit |
| Add/edit member | Ordered typed member command with stable MemberId | Validate before commit; keep row/field focus on error; one undo per accepted edit |
| Move/resize/connect | Temporary preview in EditorState, then one validated command on release | Capture loss/Escape discards preview; invalid edge never partially edits graph |
| Style | View-local style command, separate from semantic Model fields | One transaction for multi-select changes; undo restores each previous value |
| Show in diagram | New view references existing model identity | Rename shared, geometry independent; undo removes the new view only |
| Duplicate model | New model/member/relation/view IDs under an explicit duplication policy | Dialog/summary says independent copy; no aliasing by accident |
| Delete | Remove view by default; Delete model everywhere shows dependent objects and views | Explicit impact confirmation for global deletion; atomic inverse restores complete affected graph |
| Save | Snapshot → bounded worker job with SessionId/Revision/StateId/SaveRequestId | Keep dirty when newer edits exist or save fails; close waits for the correct acknowledgment |
| Validation/export | Derived snapshot job, results tagged with session/revision | Ignore stale results; I/O failure does not mutate project; export has no semantic undo entry |

Shortcut priority: modal/native dialog → focused text field → active gesture → focused canvas → application commands. Preserve Ctrl+A/C/V/Z and selection editing in text fields. Ctrl+N/O/S, Ctrl+Shift+S/Z, F2, Delete, Escape, Space+drag and zoom/fit have visible menu/help equivalents. No single-letter creation shortcut fires while typing. Keyboard users can select/reveal objects through the navigator and issue every essential action through named controls.

**Implementation rule learned in R0-02:** egui drops text focus in the same frame an Escape is pressed. Every shortcut must therefore decide using the text-focus state from the **previous** frame, as the shell's Escape handling does. Apply this to F2, Delete, Ctrl+Z and every other key that a text field also uses, and cover each with a headless test that types in a field first.

## 6. Rust structure for the UI

Grow `diagram-desktop` around actual units: `ui/theme.rs`, `ui/components.rs`, `ui/workspace.rs`, `ui/welcome.rs`, `ui/navigator.rs`, `ui/toolbar.rs`, `ui/inspector.rs`, `ui/dialogs.rs`, `ui/validation.rs`, `ui/commands.rs`, plus `editor_state.rs` and `presentation.rs`. Introduce modules when implemented, not empty-file scaffolding. Today's single `shell.rs` is split into these modules as each is built; keep its headless tests passing through every move.

- UI thread owns the current session and authoritative Document (once M1 exists).
- EditorState owns selected ViewIds, tab/viewport preferences, tool/gesture enum, focus return target and local draft buffers.
- Presentation structs expose borrowed/read-only model information and explicit availability states. Initially use a small concrete enum/struct set; no generic plugin framework.
- UiIntent is a typed enum for user intentions. A coordinator maps supported intents to domain Commands or worker requests after widget borrowing ends.
- ActionAvailability is explicit (enabled or disabled with a reason), derived from document capability, selection, history and pending work. Use one command catalog for menus, toolbar, shortcuts and command search so behaviors do not drift.
- DialogState is an enum with contextual payload; only one blocking modal owns focus at a time. Loading a file does not replace the current session until validation succeeds.
- No second writable document in a view model, no global Arc<Mutex<Document>>, no SQLite/OS dialog invocation inside canvas painting. See [Rust engineering rules](RUST-ENGINEERING.md).

Before M1, use explicitly labeled in-memory presentation fixtures and deterministic scenario states. They validate layout/focus/availability only. Do not simulate disk-save success, recoverability or functional undo in the shipped app. Keep currently unsupported actions disabled there. Once a real command is available, replace its fixture adapter and test the same action through the native UI.

## 7. Scope across diagram families

The shell reserves one notation definition for display name, tools, inspector sections, allowable relationships and capabilities. v0.1 delivers Class only. Flowchart and Use Case follow in M4, then ER/activity/state/requirements/sequence in M5. A planned-family catalog may appear in documentation; a user-facing type picker contains only implemented families. Model → view navigation, save status, validation and shortcuts remain shared. A sequence editor may replace canvas interactions and inspector sections while retaining project/tab/dialog infrastructure.

## 8. New tasks and order

The machine-readable tracker ([roadmap.json](roadmap.json)) adds **18 UI tasks** and a new **UX** milestone between M0 and M1. Existing completion evidence is unchanged.

**UX milestone, before M1:**

| Task | Deliverable |
|---|---|
| UI-01 | Design tokens (including the stronger control-border token) and reusable components |
| UI-02 | Type scale, text roles and text-direction rules; font files and the bidi result stay with R0-04 |
| UI-03 | Responsive workspace at the supported sizes |
| UI-04 | Welcome, new-project and open screens |
| UI-05 | Navigator and diagram tabs |
| UI-06 | Tool and command catalog (one source for menus, toolbar, shortcuts, search) |
| UI-07 | Canvas state presentations |
| UI-08 | Contextual inspector and member editor surfaces |
| UI-09 | Dialogs, status bar and validation drawer |
| UI-10 | Keyboard, focus and accessibility checks, including contrast of every state |
| UI-11 | Notation capability contract |
| UI-12 | Native UI review gate |

UI-01 and layout exploration can start now on the prototype. UI-02 specifies roles and rules but **must not finalize right-to-left or mixed-text rules until R0-04 records the bidi result** (section 2, Fonts). The UI-12 gate requires R0-05 and all UX tasks; R0-04 keeps ownership of the framework-wide native text and accessibility evidence. Do not duplicate its status or bypass it. UI-12 confirms the designed native surfaces, realistic fixtures and explicit integration contracts; it does not claim persistence or command semantics exist.

**M2 integration:** UI-13 project/navigation lifecycle binding; UI-14 inspector/member binding; UI-15 canvas/history binding; UI-16 save/validation status binding; UI-17 complete keyboard workflow. Each depends on the necessary M1/M2 functionality. The R2-08 walkthrough requires these bindings.

**M3 release:** UI-18 final visual/accessibility/recovery/export review; the R3-07 release gate requires it. Save/recovery implementation remains R3-03; these tasks integrate and review it rather than implementing a second storage stack.

New order: complete M0 evidence and design UX surfaces → pass UI-12 → M1 domain/storage → M2 real bindings/class workflow → M3 release polish → later families. This moves more visible UI work before M1; it increases upfront design work and review but makes the intended product concrete. UI details can be revised when actual domain constraints emerge.

**Trade-off to decide:** the tracker gates M1 on UI-12 because this plan says so. M1's first tasks (R1-01 to R1-04) are headless and do not need a window, so they could safely run in parallel with the UX work if saving calendar time matters more than designing first. To allow that, remove UI-12 from R1-01's prerequisites in `roadmap.json`.

## 9. What counts as the final design

Deliver both themes, all selection/dialog/error/loading states, a component/token catalog, focus/shortcut map, notation capability contract, native reviewed screenshots at supported sizes, and complete bindings for the released Class subset. Walk through new/open → create → edit members → connect → reuse → move/style → undo → save/reopen → export → recover from a failed operation.

Measure time, errors, dialog count and keyboard access in representative tasks before claiming easier use than VP. The HTML preview ([GUI-CONCEPT.html](GUI-CONCEPT.html)) shows layout and representative states only; it cannot validate egui text, GPU behavior or accessibility. UI completion requires native evidence. Functional completion requires real model and storage results as well. No calendar dates are promised before sizing the new work.
