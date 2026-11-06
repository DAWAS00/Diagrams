# Ideas plan: what to build next for Diagram Studio

Date: 2026-11-06. Written by Claude, with an independent pass by Codex (`codex-cli 0.167.0`, read-only, same source files). Inputs: [HANDS-ON-VP-TEST-VS-RUST-BUILD.md](../vp-comparison/HANDS-ON-VP-TEST-VS-RUST-BUILD.md), [RUN-TEST-2026-11-06.md](../vp-comparison/RUN-TEST-2026-11-06.md), the roadmap in `plan-rust-diagram-editor/`, and the Rust source. The companion file [EXECUTION-PLAN.md](EXECUTION-PLAN.md) turns these ideas into files, tests and UI changes.

## 1. The finding that drives everything

The window is not connected to the real model. `ShellState` (`app/crates/diagram-desktop/src/shell.rs`) still draws `PreviewClass` and `PreviewRelation` fixtures addressed by list index (`SelectClass(usize)`). `diagram-core` already has typed IDs, validation, 15 reversible commands and an undo history, but nothing in the window calls them. That is why Class, Association, Save and Undo are greyed out and dragging does nothing, even though 192 tests pass.

So the next work is **binding, not new features**. Both Codex and I reached this independently.

## 2. Ideas, ranked

Agreement is marked **both**, **Codex only** or **Claude only**.

| # | Idea | Why (Visual Paradigm evidence versus our gap) | Who |
|---|---|---|---|
| 1 | Replace the preview fixtures with a real session over `diagram-core`, and make every edit a command with undo | VP undoes moves; our rename works but Undo stays disabled | both |
| 2 | Create a class with one click and edit its name at once | VP does this; `CreateClass` exists, the Class tool is disabled | both |
| 3 | Move classes by dragging, with connectors following; one drag is one undo step | VP connectors follow moves; `MoveViews` exists, dragging does nothing | both |
| 4 | Draw association and generalization, with the parent end labelled in words | VP's drag direction is non-obvious; core validates cycles and duplicates | both |
| 5 | Add, edit, remove and reorder attributes and operations in the inspector | VP uses menus and a specification dialog; our buttons do nothing and core lacks a member-update command | both |
| 6 | Safe Save, Open, Save As and Close project, then a return to Welcome | VP saves projects; ours saves nothing and offers no way back to Welcome | both |
| 7 | Show the same class in several diagrams, with an explicit "Show in diagram" versus "Duplicate model" | VP's auxiliary view and Paste View versus Paste Model Element | both |
| 8 | Separate "Remove from this diagram" from "Delete from project", with an impact count | VP's delete dialog offers Referenced Model Elements; core has `delete_impact` | both |
| 9 | Right-click context menu on classes and on the canvas | VP puts Add, Delete, Duplicate and Open Specification there; ours has no canvas menu | Claude only |
| 10 | Pan, zoom-to-fit, multi-select and basic alignment | VP has zoom, grid and layouts; our Pan and Arrange are disabled | both |
| 11 | SVG export from the same scene the canvas draws | VP exports images; ours exports nothing | both |
| 12 | Keyboard, text and accessibility proof on Windows (Arabic, IME, NVDA, scaling) | VP's flaky inline editing shows how bad text entry can feel; our test was mouse-only | both |
| 13 | A short member syntax such as `name : String` and `placeOrder() : Order`, parsed in core | VP accepts this typing; ours has no text-to-member parser | Claude only |
| 14 | A duplicate-name decision shown in words | VP asks "Change to Auxiliary View"; we need our own rule | Claude only |

## 3. Risks

1. **A polished shell hiding disconnected behaviour** (both). Accept a task only when the whole path works in the real window.
2. **Data loss or a false "saved" status** (both). Failed writes and late results must never mark newer edits as saved.
3. **Index-based identity** (Codex). Intents target classes by list position, while core uses typed IDs. With shared views, delete and undo, an index can point at the wrong object.
4. **Unverified native text and accessibility** (both). Arabic, IME, screen readers and DPI can force UI changes after the fact.
5. **Misleading progress** (both). The roadmap has conflicting historical status text, and the structure check reports 11 warnings (`ShellState` has 19 public fields against a budget of 12; `shell.rs` is 1,659 lines). Adding notations first would make both worse.

## 4. Deliberately not in v0.1

- Other notations (flowchart, use case, ER, sequence, activity, state): later milestones, with the VP behaviours already recorded.
- Dependency, aggregation, composition, realization, association classes and stereotypes. See decision D3.
- VP `.vpp` and XML import, code generation, reverse engineering, database tools.
- Collaboration, cloud templates, AI generation, reporting, plugins.
- Seven layout algorithms, a general drawing suite, PNG and PDF export, rebindable shortcuts.

## 5. Decisions the project must make

**Decided 2026-11-06:** the user accepted every recommendation below. They are recorded in [ADR-003](../../../plan-rust-diagram-editor/decisions/ADR-003-UI-DECISIONS.md).

| ID | Decision | Options | Recommendation |
|---|---|---|---|
| D1 | Generalization direction in the UI | drag child to parent, or parent to child | Draw child to parent and show "Parent: X, child: Y" in the inspector. Core already treats source as child. |
| D2 | Typing a name that already exists | silently allow, block, or offer "show the existing class here" | Offer a choice in words, like VP, but never switch to an existing model without asking. |
| D3 | Relations in v0.1 | association and generalization only, or add dependency | Keep two kinds for v0.1; keep `RelationKind` extensible. Revisit after the vertical slice (R2-08). |
| D4 | Adding a member when the class is shown elsewhere | always show everywhere, ask every time | Show everywhere; the model is shared. VP's popup is a prompt we can skip. |
| D5 | Persistence unit | one SQLite file per project (planned) | Keep ADR-002; define schema v1 before any UI save code. |
| D6 | Fix status text | none | Make `roadmap.json` the only status source and regenerate the summaries. |

## 6. Where Codex and I differ

- Codex put UI-to-core binding first and treated persistence third. I agree, but would not start persistence UI until the schema (R1-05) is written, because Save and Open need it.
- Codex wants explicit parent and child labels on generalization; I turned that into D1.
- I added the context menu (idea 9), member syntax (13) and the duplicate-name rule (14), which Codex did not list.
- Codex lists dependency, aggregation and composition as out of scope; I flag it as a decision (D3) because users will draw them, but I do not recommend adding it before the class slice works.

## 7. Evidence limits

Codex did not run the app or edit files. I did not run the planned changes. Everything here is a proposal, and no estimate of effort is given.
