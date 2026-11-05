# UI research before implementation

Research date: 2026-11-05. Scope: current native prototype, proposed v0.1 UI and how released functions become discoverable and usable. **Desk research and a source-based heuristic audit, not a new user study or competitor hands-on benchmark.** No application UI is changed by this research. The [design preview](../../../../plan-rust-diagram-editor/UI-DESIGN.html) remains a proposal.

Publication note: this audit describes the prototype at the research date. Newer UI implementation arrived with commit `5dc8977`; consult the [current UI evidence](../README.md) for progress. Findings about missing surfaces are historical review inputs, not an audit of that newer implementation.

## Main decision

Subsequent hands-on evidence: [FigJam board review](FIGJAM-BOARD-REVIEW.md) records live palette, selection, zoom, pan and Actions-search tests on the user's computer. It adds a tool-placement experiment; the desk-research limits below still describe the original research pass.

Keep the navigator–canvas–inspector workspace, but test its interaction model before polishing every screen. Essential actions should be visible; every other released action must have a predictable, searchable location. Displaying all future suite features simultaneously would consume canvas space and make the first Class workflow harder to learn.

NN/G distinguishes recognition from remembering hidden commands and recommends exposing useful cues. Its progressive-disclosure guidance places common choices first and reveals specialized controls as needed. These support a discoverable hierarchy rather than an icon-only toolbar or an exhaustive ribbon. They do not prove our design is easier. [Recognition and recall](https://www.nngroup.com/articles/recognition-and-recall/), [progressive disclosure](https://www.nngroup.com/articles/progressive-disclosure/).

## 1. Evidence and limits

| Evidence | What it establishes | What it does not establish |
|---|---|---|
| Current `shell.rs` and [Windows review](../../WINDOWS-REVIEW-2026-11-05.md) | Two sample classes, selection, inspector name editing, disabled actions and Escape behavior | Full diagram editing, save/recovery, large-document usability |
| Existing VP study | Recorded workflow and model/view distinction from the installed reference product | Measured user preference or all VP features |
| Official draw.io and Lucid documentation | Documented interaction patterns worth considering | Their suitability or measured speed in our application |
| NN/G and W3C guidance | General discoverability and interaction principles | Native egui compliance or usability of our implementation |
| Existing egui tests | Implemented widget/state behavior in the prototype | Human comprehension, IME, screen-reader output or all DPI configurations |

No new interviews, timed sessions, eye tracking or fresh GUI execution were performed for this research pass. Recommendations below are hypotheses to validate. Earlier native observations remain documented separately.

## 2. Audit of the current prototype

| Finding from source / previous native evidence | User consequence to investigate | Priority and task |
|---|---|---|
| Toolbar exposes six disabled actions | A first-time user can see intended actions but cannot complete a diagram | P0: retain prototype honesty; UI-06 defines availability/reasons; real actions arrive in UI-13–17 |
| No visible task-oriented creation guidance beyond the prototype | User may not know the first meaningful action | P1: UI-04/UI-07 blank state with labeled Add Class and optional short example |
| Name editing is only in the inspector; sample names change directly in desktop state | Pointer travel and no real cancel/undo contract | P0: UI-08/UI-14 inline rename plus inspector, common draft commit/cancel policy |
| Class width is fixed at 180 and text sizes are hardcoded | Long or mixed-language names may overflow, wrap badly or become difficult to identify | P0: UI-02/UI-07 measure names, define wrapping/minimum width and expose full names accessibly |
| No real project search, tree, tabs or reveal action | No evidence for navigation beyond two objects | P1: UI-05 labeled search scope, hierarchical navigation, reveal/fit selected object |
| Sidebar/inspector use default sizes 220/280; full narrow-window UX is not present | Panels can compete with the canvas at high scaling | P0: UI-03 collapse/drawer/overflow behavior with focus return; test supported sizes |
| The canvas background/ink colors and member fonts are fixed | Theme changes alone cannot ensure consistent contrast or exported document styling | P1: UI-01 chrome tokens separate from document styles; actual contrast review |
| Previous-frame text focus is needed to protect Escape behavior | New shortcuts could steal text editing keys | P0: UI-06/UI-10 central focus priority and event tests |
| Model objects and views are not yet distinguished in controls | Future shared rename and deletion behavior may surprise users | P0: UI-05/UI-08/UI-09 explicit Model versus This view labels and impact summaries |
| No real save/export/history implementation | Attractive UI could imply safety that does not exist | P0: UI-09/UI-16 actual save acknowledgments, persistent errors and disabled unsupported actions |

Source anchors: [toolbar and layout](../../../crates/diagram-desktop/src/shell.rs), [command/storage architecture](../../../../plan-rust-diagram-editor/02-ARCHITECTURE.md). These are implementation gaps, not evidence of user failure rates.

## 3. External patterns and our interpretation

### Creation and connection

draw.io documents connection arrows, quick clone/connect and keyboard alternatives. The useful lesson is reducing repeated trips to the palette while retaining another way to perform the operation. [Official connection workflow](https://www.drawio.com/docs/manual/connectors/connect-shapes/).

**Proposal for our Class editor:** labeled Class and Relationship tools, plus a focused/hovered-node Connect action. Offer click source → click target as well as dragging; provide a dialog/list-based endpoint route for keyboard users. Show relationship kind and valid targets before committing. Do not copy generic clone-and-connect behavior blindly: in our semantic model, duplicate object and reuse existing object are different operations. Quick-create-connected-class is a later enhancement after ordinary connect is reliable.

### Contextual properties

Lucid documents a selection editing bar and a right panel for additional controls. This demonstrates an approach that places relevant controls near the selected object without keeping every property open. It is a product pattern, not proof that floating tools are always faster. [Lucid editor guide](https://help.lucid.co/hc/en-us/articles/11970952773652-Welcome-to-Lucidchart).

**Proposal:** keep the persistent inspector as the stable location. Use direct name editing and connection handles on the canvas; defer a floating mini-toolbar until testing shows a need. A floating bar must never obscure class members or edge labels. Within the inspector, start with semantic fields, then Appearance and Advanced. Label the scope as “Shared model — changes all views” or “This diagram view only.”

### Navigation and canvas space

draw.io documents shape search, zoom/pan and an outline view for larger diagrams. Our first release needs search and Fit/Reveal before adding a permanent minimap. [Editor documentation](https://www.drawio.com/docs/manual/editor/).

**Proposal:** separate project-object search from command search; return paths and notation/diagram context. Selecting a result reveals the object without changing its geometry. Test collapsed panels and Fit selection with realistic fixtures. Add an optional outline/minimap only if users still lose their place; it is not an unconditional v0.1 requirement.

### Keyboard, pointer and accessibility

W3C APG describes toolbar grouping and predictable focus movement, and its keyboard guidance distinguishes focus from selection. These are web patterns we can adapt as native interaction conventions, not ARIA attributes to paste into Rust. [Toolbar pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/), [keyboard interface](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/).

W3C's dragging guidance calls for a single-pointer non-drag alternative in applicable web interactions; its target-size guidance addresses difficult small targets. For this native product, use them as design benchmarks and validate actual Windows behavior rather than claiming WCAG conformance. [Dragging movements](https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements), [target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html).

**Proposal:** click-place and click-source/target alternatives, numeric geometry fields where appropriate, visible focus, large hit regions and object-list access. A keyboard alternative alone does not solve every pointer dexterity issue. Make disabled-action explanations available through adjacent help, menu descriptions or command search, not only hover. Test real assistive technology.

## 4. Recommended changes to the proposed design

1. **Keep a small stable primary row.** Project, Undo/Redo, Save and Export stay predictable. Notation tools belong in one labeled tool row. Move Arrange and advanced commands into discoverable menus. Do not show unsupported diagram families as working tools.
2. **Separate object search and command search.** “Find object…” searches the project; “Search commands…” searches actions with availability reasons. Results identify which diagram/model they refer to.
3. **Give selection a clear scope.** Inspector headings say Class/Relationship/Multiple views, and show whether changes affect the shared model or selected views. Mixed values are not silently overwritten.
4. **Make connections explicit and reversible.** Choose relation kind, select endpoints, preview rules/multiplicity, then commit once. Provide click and keyboard routes, not drag alone.
5. **Provide direct editing without two competing truths.** F2/double-click and the inspector use the same draft and validation rules. Escape cancels; Enter commits; text undo remains local until commit. Test focus-loss policy before freezing it.
6. **Reserve space for errors.** Field errors remain beside the field; save failures remain visible in project status with Retry/Save As; structural issues use the validation drawer with Reveal. Do not use toasts as the only failure record.
7. **Make empty states actionable.** Explain Add Class, add an existing model view, and keyboard access. Use one small hint, not a tutorial wall. Keep demo projects separate from the user's blank project.
8. **Fit real names and documents.** Test long English, Arabic and mixed names, many members, multiple tabs, offscreen objects and small windows. Tokens and font assets support this; colors alone cannot solve it.

The design HTML currently illustrates appearance and a few states. It does not demonstrate discoverability of all functions, actual commands, responsive egui behavior, error recovery or successful Arabic editing. It should remain a research candidate until tested.

## 5. Function discoverability audit

[UI-FUNCTION-MAP.csv](UI-FUNCTION-MAP.csv) maps v0.1 actions to their primary location, alternate route, prerequisite, feedback and owning task. This is a requirements checklist; it does not mark actions implemented.

Every released action must have a named primary route, a clear availability rule and visible success/error feedback. Shortcut-only, hover-only or context-menu-only essential actions fail the proposed review gate. Secondary routes must invoke the same command as the primary route. Future AI/collaboration/code generation is outside this release's function inventory.

## 6. Research validation before visual implementation

**Step A — walk through low-cost alternatives.** Compare a persistent inspector against an optional compact contextual editor; compare Diagrams/Model tabs against two simultaneously expanded trees. Use wireframes and fixtures, no domain implementation. Evaluate at 1280×800 and 800×500. Keep a decision log with the specific task and reason.

**Step B — formative sessions.** Recruit a small varied group if available: first-time modelers, experienced UML users and keyboard/assistive-tech users, including Arabic input experience. Five sessions is a practical starting proposal, not a statistical proof. Do not invent participant results if none are available. Until then, record expert walkthrough findings as such.

| Task | What to observe |
|---|---|
| Start a blank Class diagram and create Customer | Can the person identify the first action without instruction? |
| Add an attribute and rename through the canvas | Can they find fields, commit and cancel? Do they understand shared edits? |
| Connect Customer and Order, then cancel an invalid attempt | Can they choose kind/endpoints and understand why a target is invalid? |
| Show Customer in another diagram, then remove only one view | Do reuse, duplication and deletion scopes make sense? |
| Find an offscreen object and return to the full diagram | Search scope, Reveal, Fit and selection continuity |
| Save, encounter a write error, then recover or cancel close | Does the person know whether work is safe and which action to take? |
| Repeat essential actions without dragging and with keyboard | Focus visibility, efficient navigation and alternate routes |
| Edit a long mixed Arabic/English name at high scale | Glyphs, joining, caret movement, selection, paste and truncation |

For fixtures, label save/error simulation. Real persistence and recovery tests wait for their functional tasks. Record success without help, first action, time, wrong turns, help requests, mistaken scope, recovery outcome and a short difficulty rating. Counterbalance alternatives to reduce learning bias. Compare with the existing VP workflow only where tasks and conditions match.

**Proposed go/no-go criteria:** no essential action lacks a visible named route; all critical wrong-scope deletion/false-save findings are resolved; basic create/edit/connect/reveal is found without coaching in the formative sessions; no keyboard trap; text editing and full names remain usable at supported sizes/scales. A failed session triggers design revision and retest, not a claim of statistical failure. Timing targets should follow an actual baseline rather than invented speed claims.

## 7. How this affects the existing tasks

No new task or completed status is added by this research. It supplies review inputs to UI-01/02 (readability/fonts), UI-03 (space), UI-04 (entry), UI-05 (navigation), UI-06 (discoverability), UI-07 (creation/connections), UI-08 (scope/drafts), UI-09 (recovery), UI-10 (accessibility) and UI-12 (native review). UI-13–18 must later repeat these journeys with real commands and storage.

Start with the function map and two workflow wireframes, then validate first-use creation/connection and model/view comprehension. Apply visual tokens and polish after those choices are credible. The previously pushed UI design is the starting hypothesis, not an irreversible final design. No Rust source, dependencies or app task completion is changed in this research update.
