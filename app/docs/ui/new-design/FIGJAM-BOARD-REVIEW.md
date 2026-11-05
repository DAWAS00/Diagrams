# Hands-on FigJam board review

Date: 2026-11-05. Purpose: inspect how Figma's board tools can inform Diagram Studio. Opened Figma in the user's signed-in Chrome session and reviewed an existing architecture FigJam board after the user allowed any diagram. Used Figma's connector to inspect its shape/section/connector structure, and the live browser to test interaction. No board objects, labels, styling, connectors, sharing or permissions were intentionally modified. No private board content or screenshots are copied into this repository.

## What was actually tested

| Action | Observed result | Limit |
|---|---|---|
| Open existing architecture board | Canvas shows multiple diagram sections and a compact bottom tool dock | This is FigJam, not the Figma Design editor |
| Open Shapes and connectors | Secondary palette exposes named shapes and bent/curved/straight connector choices | Did not place or draw new objects in the existing board |
| Switch to Select and click a shape | Selection outline/handles and contextual formatting bar appear | No resize, rename or formatting mutation tested |
| Zoom in with visible control | Diagram scale changes while the tool dock stays available | No full trackpad, keyboard or monitor-scale matrix |
| Activate Hand and drag empty canvas | Viewport pans; diagram moves together on screen | No diagram node movement committed |
| Open Actions and search Connector | Search exposes a Connector action with shortcut X | Did not execute AI/plugin actions or create a connector |
| Dismiss Actions with Escape | Search closes and focus returns to the board | Not a comprehensive keyboard-accessibility assessment |
| Inspect board through Figma connector | Sections contain shapes; connectors reference endpoint node IDs and cap styles | Does not prove our domain model or native routing implementation |

The board's accessibility notice said board screen-reader support was disabled. It was not enabled or tested. Presence of a toolbar accessibility tree is not proof of accessible canvas content. The board was left open for the user. This was interaction research, not a benchmark or an app implementation pass.

## Decisions for our app

| FigJam pattern | Adopt or adapt for Diagram Studio | Rust implementation / existing task |
|---|---|---|
| Canvas occupies most of the window | Adopt a large central board, with collapsible navigator and inspector | egui panels/drawers, UI-03; retain named controls on small windows |
| Compact tool dock expands to a palette | Adapt: labeled Select, Class, Relationship, Note and Arrange; one place for active notation tools | Shared command catalog and availability, UI-06/UI-11. Compare compact dock versus top tool row before choosing placement |
| Contextual formatting bar near selection | Use sparingly for short actions; keep a stable inspector for semantic properties | UI-08: Class members and relation ends need persistent space; a floating bar must not cover member text |
| Actions search with matching command and shortcut | Adopt command search, separate from object search | UI-06: one typed command entry used by menu/toolbar/shortcut/search; unavailable commands explain why |
| Separate Select and Hand modes | Adopt explicit active-tool indication plus temporary pan gesture | Gesture enum, pointer capture and cancellation in UI-07/UI-15; never interpret pan as node drag |
| Related shapes grouped in sections | Adapt as visual groups, distinct from semantic packages and separate diagram documents | UI-05/UI-11; grouping alone must not change model ownership |
| Shape-specific palette and connector styles | Adapt into notation-specific tools and validated relation types | Class uses association/generalization, not merely arbitrary arrowheads; UI-11/UI-15 |
| Named connected endpoints in board structure | Adopt stable model/view references with independent route geometry | M1 domain IDs and references; M2 endpoint/route rendering; do not save only screen coordinates |
| Minimal persistent chrome | Adopt the space efficiency, retain labels and stable locations | UI-01/UI-03/UI-10; do not copy icon-only discovery assumptions |

## Recommended workspace after this review

Keep the project header and left navigator from our design. The central board gets tabs, a compact labeled tool row, large canvas, visible zoom/Fit and an optional compact quick-action surface. The right inspector is contextual but stable: no selection → diagram settings; class → Model/Members/Appearance; relation → endpoints/roles/multiplicity/route; multiple selection → common fields with mixed values.

A bottom floating tool dock is now a candidate to compare with the planned top tool row, not an automatically accepted replacement. They must not both duplicate the same tools. Test at small windows and high DPI, with menus and selected objects near screen edges. Keep save/error status in a predictable location.

Adopt the board interaction model, not FigJam's entire collaboration/whiteboard suite. Voting, timers, reactions, decorative modes, AI generation and plugin marketplaces are outside the first Class-editor scope. No cloud/account requirement is introduced.

## How the Rust board will work

1. **Viewport:** store pan/zoom separately from semantic document state. With pan in screen units, use `screen = canvas_origin + pan + world * zoom` and its checked inverse for picking. Zoom around a chosen anchor; clamp scale; Fit and Reveal calculate bounds without moving models.
2. **Input:** `Select`, `Pan`, `PlaceClass`, `Connect`, and transient move/resize/text states are explicit variants. Resolve focused text/modal input first. Escape/capture loss cancels only the current transient action. A completed gesture emits one intent.
3. **Scene:** derive class compartments, text, route paths, labels, arrowheads and hit regions from model/view records. Selection handles, connection affordances and grid are editor overlays. Use screen-sized hit tolerances so handles remain usable at different zooms.
4. **Commands:** one dispatcher translates UI intents into validated domain changes. Do not mutate the future document directly in each painting pass. UI availability comes from the same command description used by search and menus.
5. **Connections:** allow both drag and click-source/click-target routes, plus endpoint controls. Validate kind/direction/cycles; commit relation and view together. Moving a view updates incident route geometry while retaining endpoint identity.
6. **Properties:** keep drafts for inline and inspector edits, with one commit/cancel policy. Model name/member edits affect shared model views; geometry/style edits affect the selected view. Do not import the prototype's direct mutable strings as the final document architecture.
7. **Storage:** snapshots and typed worker results drive actual save feedback. Figma's file handling is not a substitute for our `.dgram` schema, recovery and stale-result checks.
8. **Validation:** extend egui_kittest with tool activation, menu/search invocation equivalence, focus protection and viewport picking tests. Verify pointer/keyboard/Arabic/assistive-tech behavior on native Windows after implementation.

## From a future Figma design file to native controls

If we produce dedicated Diagram Studio frames in Figma Design, inspect component variants, spacing, colors, fonts, resizing constraints and all error/selection states. Translate those into our semantic design tokens and egui components. Export only needed original/licensed icons or images. Rebuild layout rules in Rust; do not use a screenshot as the interface or treat generated HTML/React as the native implementation.

A FigJam board describes objects, connectors and layout; it is not a full application UI specification. This review complements [UI-DESIGN-AND-INTEGRATION.md](../../../../plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md), the [function map](UI-FUNCTION-MAP.csv) and [desk research](UI-RESEARCH.md).

## Next concrete design experiment

Compare two low-cost variants using identical Class fixtures: **A**, top labeled tool row with stable inspector; **B**, compact bottom tool dock with the same stable inspector. Walk through add/name/connect/reveal and repeat without dragging. Measure discovery, pointer travel, canvas obstruction, focus and errors. Choose placement from that evidence; keep the rest of the shared shell stable. UI-03/UI-06/UI-07/UI-08/UI-10/UI-12 own these decisions; no task is marked done by this inspection.
