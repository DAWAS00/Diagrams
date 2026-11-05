# Complete UI design and integration plan

Date: 2026-11-05. Proposed v0.1 design baseline. This extends the roadmap at the user's request to prioritize the complete UI before broad functional implementation. It does not mark any new UI feature implemented. [Visual preview](UI-DESIGN.html) · [task tracker](roadmap.json) · [current evidence](../app/docs/WINDOWS-REVIEW-2026-11-05.md).

## 1. The intended final workspace

One focused desktop workspace, with a light default and a supported dark theme. A new user sees labeled tools and a blank diagram; an experienced user can use shortcuts and a command palette. The UI stays consistent when the active notation changes.

```text
Project menu  Customer model · unsaved     Undo Redo    Search commands    Save Export
┌ Project [search] ──────┬ Diagram tabs: Class overview | Domain details + ┬ Inspector ┐
│ Diagrams | Model      │ Select  Class  Association ▾  Note  Arrange ▾    │ Model     │
│ ▾ Customer model      ├─────────────────────────────────────────────────┤ Appearance│
│   Class overview      │                                                 │ Name      │
│   Domain details      │           Customer ───────── Order               │ Members   │
│ ▾ Classes             │                                                 │ Relations │
│   Customer            │     diagram canvas with selection / previews     │ Style     │
│   Order               │                                                 │ Advanced  │
│ + Diagram             ├ Validation drawer (collapsed unless requested) ─┤           │
└───────────────────────┴─────────────────────────────────────────────────┴───────────┘
Unsaved · local path             2 warnings                    Snap  Grid  − 100% + Fit
```

This is the final **v0.1 target**, subject to the M0 text/accessibility gate and UI review. It is not a claim to implement all Visual Paradigm features. Later notation editors reuse the shell and add tools/property sections through an explicit capability definition. Sequence will need dedicated temporal interactions rather than forced class-box behavior.

## 2. Technology and assets

| Part | Use / decision |
|---|---|
| Native app | Existing Rust workspace, eframe 0.37.0 and its aligned egui re-export; existing wgpu backend |
| Controls | egui text fields, buttons, menus, panels, scroll areas, popup/dialog surfaces and keyboard focus; small original components for repeated patterns |
| Canvas | Renderer-neutral geometry/scene in diagram-canvas; desktop painter adapter; no document ownership inside widgets |
| Tests | Existing egui_kittest 0.37.0 for event/accessibility-tree behavior, plus real Windows checks; small reviewed image fixtures only where useful |
| Accessibility | Existing AccessKit integration, named controls, keyboard object-list alternative, meaningful selection and relation descriptions; real Narrator/NVDA testing remains required |
| Fonts | Candidate bundled Noto Sans + Noto Sans Arabic; verify exact files, version, metrics and notices before adding assets. Arabic upstream publishes OFL-1.1. Font choice alone does not prove shaping/bidi/IME correctness |
| Icons | Original simple vector line icons plus visible labels for common tools; preserve any notices if an external icon source is later selected |
| Native file dialogs | Proposed rfd when real Open/Save is implemented; resolve a compatible locked version then. Cancel is a normal outcome |
| Preferences | Versioned per-user theme/panel/scale/recent-file settings, separate from the project and dirty state; no project bodies in preferences |
| Persistence | Existing rusqlite dependency for future project storage, executed by a worker; the UI never performs a database save during painting |

No framework change, runtime, new dependencies or asset download occurs in this planning update. Avoid adding docking, animation, webview or chart frameworks without a demonstrated need.

Official references checked: [eframe](https://docs.rs/eframe/0.37.0/eframe/), [egui source](https://github.com/emilk/egui), [rfd](https://docs.rs/rfd/latest/rfd/), [Noto Arabic and its license](https://github.com/notofonts/arabic). Verify exact APIs and fonts again at implementation time. ADR-001 remains conditional until R0-05.

## 3. Visual specification

Use semantic tokens rather than scattered RGB constants. Tokens cover background, surface, raised surface, canvas, text, secondary text, border, accent, selection, focus, warning and danger. Start with:

| Token | Light target | Dark target |
|---|---|---|
| Workspace / panels | #F4F6FA / #FFFFFF | #141A24 / #1D2633 |
| Canvas | #FCFCFD | #202B39 |
| Primary text | #182438 | #EDF2F8 |
| Secondary text | #526078 | #B6C2D3 |
| Accent / focus | #245CC5 | #91B8FF |
| Border | #D9E0EB | #465469 |
| Warning / danger text | #8A4B00 / #B42318 | #FFD08A / #FFB4AD |

These are starting design values, not a passed contrast audit. Check actual foreground/background pairs and disabled/focus states. Target 4.5:1 normal text and 3:1 large text/essential control boundaries, with labels/icons/patterns supplementing color.

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

## 6. Rust structure for the UI

Grow `diagram-desktop` around actual units: `ui/theme.rs`, `ui/components.rs`, `ui/workspace.rs`, `ui/welcome.rs`, `ui/navigator.rs`, `ui/toolbar.rs`, `ui/inspector.rs`, `ui/dialogs.rs`, `ui/validation.rs`, `ui/commands.rs`, plus `editor_state.rs` and `presentation.rs`. Introduce modules when implemented, not empty-file scaffolding.

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

The machine-readable tracker adds **18 UI tasks**. Existing completion evidence remains unchanged.

**UX milestone, before M1:** UI-01 design tokens/components; UI-02 font/typography specification; UI-03 responsive workspace; UI-04 welcome/new/open screens; UI-05 navigator/tabs; UI-06 tool/command catalog; UI-07 canvas state presentations; UI-08 contextual inspector; UI-09 dialogs/status/validation; UI-10 keyboard/accessibility checks; UI-11 notation capability contract; UI-12 native UI review gate.

UI-01/UI-02 and layout exploration can start now on the prototype. The UI-12 gate requires R0-05 and all UX tasks; R0-04 continues to own the framework-wide native text/accessibility evidence. Do not duplicate its status or bypass it. UI-12 confirms the designed native surfaces, realistic fixtures and explicit integration contracts; it does not claim persistence or command semantics exist.

**M2 integration:** UI-13 project/navigation lifecycle binding; UI-14 inspector/member binding; UI-15 canvas/history binding; UI-16 save/validation status binding; UI-17 complete keyboard workflow. Each depends on the necessary M1/M2 functionality. The R2-08 walkthrough requires these bindings.

**M3 release:** UI-18 final visual/accessibility/recovery/export review; the R3-07 release gate requires it. Save/recovery implementation remains R3-03; these tasks integrate and review it rather than implementing a second storage stack.

New order: complete M0 evidence and design UX surfaces → pass UI-12 → M1 domain/storage → M2 real bindings/class workflow → M3 release polish → later families. This moves more visible UI work before M1; it increases upfront design work and review but makes the intended product concrete. UI details can be revised when actual domain constraints emerge.

## 9. What counts as the final design

Deliver both themes, all selection/dialog/error/loading states, a component/token catalog, focus/shortcut map, notation capability contract, native reviewed screenshots at supported sizes, and complete bindings for the released Class subset. Walk through new/open → create → edit members → connect → reuse → move/style → undo → save/reopen → export → recover from a failed operation.

Measure time, errors, dialog count and keyboard access in representative tasks before claiming easier use than VP. The HTML preview shows layout and representative states only; it cannot validate egui text, GPU behavior or accessibility. UI completion requires native evidence. Functional completion requires real model and storage results as well. No calendar dates are promised before sizing the new work.

## 10. Findings from implementation (verified 2026-11-05)

These were checked against the code and resolved dependencies while building UI-01, UI-02 and UI-06. They refine sections 2–6; progress and evidence are in [the UI progress page](../app/docs/ui/README.md).

**Dependencies (section 2).**
- **Fonts and bidi.** eframe's default `system_fonts` feature is already active, so system fonts are the fallback today; that is most likely why `Order طلب` showed glyphs on Windows. egui 0.37 shapes text with HarfRust (Arabic letters join) but **has no bidirectional reordering**; its source notes this as future work. A bundled font cannot fix mixed Arabic/English order, so R0-04 must test it. If it fails, the ADR-001 revisit trigger applies.
- **Preferences** need serialization, which is a new dependency: eframe's `persistence` feature (adds `ron` and `serde`) or `serde_json`. Choose and review it when preferences are implemented.
- **Icons** drawn with the egui painter need no crate. SVG icon files would need `egui_extras` with `svg` (pulls in `resvg`).
- **Image fixtures** need extra `egui_kittest` features; enable them with the first image test.
- `rfd` 0.17.2 declares no minimum Rust version, so compile it on the pinned 1.99.1 when it is added.

**Contrast (section 3).** Measured with the WCAG formula. Every text, accent, warning and danger token passes 4.5:1 in both themes. The planned **border** token measures 1.2–1.3:1 (light) and 1.9–2.3:1 (dark), below the 3:1 needed for control boundaries; a first replacement, `#8A97AB`, also failed. The implementation keeps the planned border for decorative separators and adds a control-border token: `#76839A` (light, 3.3–3.8:1) and `#6E7F98` (dark, 3.1–4.3:1). Full table: [UI-01 evidence](../app/docs/ui/UI-01-TOKENS-AND-COMPONENTS.md).

**Text roles and direction (section 3).** Sizes come only from one `TextRole` definition (page title 24, panel title 16, body 15, label 14, code 14 monospace).

| Rule | Status |
|---|---|
| Names and member text are stored exactly as typed; the app never inserts direction marks | Decided |
| Application chrome is English and left-to-right in v0.1; mirroring for a right-to-left UI language needs a localisation plan | Decided |
| User text in lists, the status bar and canvas shapes stays on one line and ends in an ellipsis; the inspector field shows it in full | Decided, tested |
| Shortened text keeps its full value in the accessibility tree | Decided, tested |
| Display order of Arabic and mixed text; alignment and ellipsis side for right-to-left text | **Open** until R0-04 records the bidi result |

**Keyboard and focus (section 5).**
- egui drops text focus in the same frame Escape is pressed, so every shortcut decides from the **previous** frame's text focus. The command catalog applies this centrally (`commands::take_shortcut`).
- egui keeps a closed modal's layer on top for one more frame, and a widget behind a modal gives up focus. Return focus to the remembered widget only once `top_modal_layer()` is empty, and test that it really returns. Every dialog in UI-09 follows this.
