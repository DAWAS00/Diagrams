# Diagram Studio — UI work: what is done and what is next

This page is for anyone following the project. It summarises the user-interface work, links the evidence for each finished task, and lists what comes next. The full specification is the [UI plan](../../../plan-rust-diagram-editor/07-UI-PLAN.md). Task status lives in the [roadmap tracker](../../../plan-rust-diagram-editor/roadmap.json), and the [dashboard](../../../plan-rust-diagram-editor/ROADMAP.html) shows it.

## Where the project stands

- **Working today:** a native Windows/Linux desktop window (egui) with a toolbar, project sidebar, canvas with two sample classes, inspector and status bar. You can select a class and rename it. Light and dark themes switch instantly.
- **Not built yet:** the real project model, adding or connecting classes, undo, saving and export. Those buttons are visible but disabled, with an explanation.
- **Order of work:** finish the native foundation gate (M0) → design every screen as native fixtures (UX milestone, UI-01 to UI-12) → build the model and storage (M1) → connect the screens to the model (M2) → reliability and the v0.1 release (M3).

## UI task status

| Task | What it delivers | Status |
|---|---|---|
| UI-01 | Design tokens and base components | **Done** — [evidence](UI-01-TOKENS-AND-COMPONENTS.md) |
| UI-02 | Type scale and text roles | Next |
| UI-03 | Responsive workspace | Not started |
| UI-04 | Welcome, new-project and open screens | Not started |
| UI-05 | Navigator and diagram tabs | Not started |
| UI-06 | Tool and command catalog | Next |
| UI-07 | Canvas state presentations | Not started |
| UI-08 | Contextual inspector | Not started |
| UI-09 | Dialogs, status bar and validation drawer | Not started |
| UI-10 | Keyboard, focus and accessibility checks | Not started |
| UI-11 | Notation capability contract | Not started |
| UI-12 | Native UI review gate | Not started |
| UI-13 – UI-17 | Bind the screens to the real model (M2) | Not started |
| UI-18 | Final visual, accessibility, recovery and export review (M3) | Not started |

## What has been done

### UI-01 — Design tokens and base components
- One set of named colours for light and dark themes, instead of colours scattered through the code. Every text and control colour is measured against its background and meets the accessibility targets (4.5:1 for text, 3:1 for control borders).
- **Problem found and fixed:** the border colour in the plan was too faint for buttons and fields. It measured 1.2–2.3:1, where 3:1 is needed. New border colours pass in both themes.
- Reusable components (titles, labels, labeled fields, list rows, buttons that explain why they are disabled, warnings in words rather than colour alone). The shell uses only these.
- Theme choice (Light, Dark, System) in the status bar, applied instantly. Diagrams keep their own colours, so switching theme never changes how a diagram looks or exports.
- 8 new automated tests; all 27 desktop tests pass.

| Light | Dark |
|---|---|
| ![Light theme](ui-01-light.png) | ![Dark theme](ui-01-dark.png) |

## What is next
1. **UI-02 — type scale and text roles:** page title 24, panel title 16, body 15, labels and code 14, applied from one place. Right-to-left text rules wait for the Arabic/bidi result of R0-04.
2. **UI-06 — command catalog:** one list of commands feeds the toolbar, menus, shortcuts and search, so they can't drift apart. Every disabled command states its reason.
3. **UI-03 — responsive workspace:** the full header, tab row and tool row; panels that collapse into drawers on small windows (down to 800×500).
4. **In parallel (needs a Windows desktop):** R0-04 text and accessibility matrix — Arabic and mixed text, IME, Narrator/NVDA and 100/150/200% scaling — then the R0-05 framework decision.

## How to try it
From `app/`:

```sh
cargo run -p diagram-desktop --locked
```

Click a class or a name in the Model list, rename it in the inspector, and switch Light / Dark / System in the bottom-right corner. Run all checks with `scripts/check.ps1` on Windows, or the commands in the [workspace README](../../README.md).
