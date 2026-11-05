# Diagram Studio — UI work: what is done and what is next

This page is for anyone following the project. It summarises the user-interface work, links the evidence for each finished task, and lists what comes next. The full specification is the [UI plan](../../../plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md). Task status lives in the [roadmap tracker](../../../plan-rust-diagram-editor/roadmap.json), and the [dashboard](../../../plan-rust-diagram-editor/ROADMAP.html) shows it.

## How UI work is checked

Every UI change follows the [UI rules](../../../plan-rust-diagram-editor/UI-RULES.md): visible labels, keyboard access, reasons for disabled controls, errors in words, measured contrast, and layouts checked at 800×500 to 1440×900. An automated **usability audit** (`tests/ui_audit.rs`) checks every screen for unnamed controls, controls too small to hit, overlapping or off-window controls, and controls Tab cannot reach. A self-test proves the audit catches each of these.

## Where the project stands

- **Working today:** a native Windows/Linux desktop window (egui) with a toolbar, project sidebar, canvas with two sample classes, inspector and status bar. It opens on a welcome screen where you can start a new (empty) project or the sample project; in the sample you can select and rename classes, search commands and classes with Ctrl+K, open the panels as drawers on small windows, and switch between light and dark themes instantly.
- **Not built yet:** the real project model, adding or connecting classes, undo, saving and export. Those buttons are visible but disabled, with an explanation.
- **Order of work:** finish the native foundation gate (M0) → design every screen as native fixtures (UX milestone, UI-01 to UI-12) → build the model and storage (M1) → connect the screens to the model (M2) → reliability and the v0.1 release (M3).

## UI task status

| Task | What it delivers | Status |
|---|---|---|
| UI-01 | Design tokens, components and states | **Done** — [evidence](UI-01-TOKENS-AND-COMPONENTS.md) |
| UI-02 | Type scale, text roles and font files | In progress — [evidence](UI-02-TYPE-SCALE-AND-TEXT-ROLES.md); reviewed font files remain |
| UI-03 | Responsive workspace | **Done** — [evidence](UI-03-RESPONSIVE-WORKSPACE.md) |
| UI-04 | Welcome, new-project and open screens | **Done** — [evidence](UI-04-WELCOME-AND-ENTRY.md) |
| UI-05 | Navigator and diagram tabs | **Done** — [evidence](UI-05-NAVIGATOR-AND-TABS.md) |
| UI-06 | Tool and command catalog | **Done** — [evidence](UI-06-COMMAND-CATALOG.md) |
| UI-07 | Canvas state presentations | Not started |
| UI-08 | Contextual inspector | **Done** — [evidence](UI-08-CONTEXTUAL-INSPECTOR.md) |
| UI-09 | Dialogs, status bar and validation drawer | Not started |
| UI-10 | Keyboard, focus and accessibility checks | Not started |
| UI-11 | Notation capability contract | **Done** — [evidence](UI-11-NOTATION-CONTRACT.md) |
| UI-12 | Native UI review gate | Not started |
| UI-13 – UI-17 | Bind the screens to the real model (M2) | Not started |
| UI-18 | Final visual, accessibility, recovery and export review (M3) | Not started |

## What has been done

### UI-01 — Design tokens and base components
- One set of named colours for light and dark themes, instead of colours scattered through the code. Every text and control colour is measured against its background and meets the accessibility targets (4.5:1 for text, 3:1 for control borders).
- **Problem found and fixed:** the border colour in the plan was too faint for buttons and fields. It measured 1.2–2.3:1, where 3:1 is needed. New border colours pass in both themes.
- Reusable components (titles, labels, labeled fields, list rows, buttons that explain why they are disabled, warnings in words rather than colour alone). The shell uses only these.
- Theme choice (Light, Dark, System) in the status bar, applied instantly. Diagrams keep their own colours, so switching theme never changes how a diagram looks or exports.
- 8 new automated tests check the colours and the theme switch.

| Light | Dark |
|---|---|
| ![Light theme](ui-01-light.png) | ![Dark theme](ui-01-dark.png) |

- **Completed later:** focus, error and disabled states are measured in both themes, fields can show an error (red border plus an "Error:" message), and a **component showcase** (Menu → View → Component showcase, or Ctrl+K "showcase") displays every control in every state as a labeled design fixture.

| Component showcase, light | Dark |
|---|---|
| ![Showcase light](ui-01-showcase-light.png) | ![Showcase dark](ui-01-showcase-dark.png) |

### UI-02 — Type scale and text roles
- Five text roles define every text size in one place: page title 24, panel title 16, body 15, labels 14, code 14 (monospace).
- Rules for text direction are written down. Showing Arabic and mixed Arabic/English text in the right order stays **open** until the R0-04 check, because the GUI library cannot reorder right-to-left text yet.
- **Two problems found and fixed** by testing a 70-character class name in the smallest window (800×500): list rows wrapped onto several lines, and the name spilled out of its box on the canvas. Both now shorten with "…", and screen readers still hear the full name.
- 9 new automated tests check sizes and that nothing overlaps or leaves the window at 800×500, 1280×800 and 1440×900.

![Smallest window with a very long class name](ui-02-800x500-long-name.png)

### UI-06 — Tool and command catalog
- One list of commands now feeds the toolbar, the future menus, keyboard shortcuts and a new **command search** (Ctrl+K or the **Search** button), so they can't drift apart. Every unavailable command says why, for example "Not available yet: this prototype cannot save."
- Typing in a text field keeps its own keys: Ctrl+Z undoes typing and Escape leaves the field. They don't trigger app commands.
- The search finds commands and classes; Enter runs the highlighted result and Escape closes it.
- **Two problems found and fixed:** keyboard focus did not return to where you were after closing the search, and Enter did nothing because the highlight started on an unavailable command.
- 17 new automated tests.

![Command search](ui-06-command-search.png)

### UI-03 — Responsive workspace
- The full window layout from the design: a header with a **Menu** (every command, grouped, with shortcuts), Undo/Redo, Search, Save and Export; a tab row; a tool row with Select, Pan, Class, Association, Note and Arrange; the project panel and inspector at their planned widths.
- **Small windows:** below 1020 points wide, the panels become drawers over the canvas, opened by the **Project panel** and **Inspector** buttons. **Escape** closes a drawer and returns keyboard focus to the button that opened it. A panel you hide stays hidden when the window is resized.
- **Two problems found and fixed:** Escape was swallowed whenever *any* button had keyboard focus, so keyboard users couldn't close a drawer or clear the selection; and unpressed toggle buttons had no border, so they looked like plain text.
- 15 new automated tests; all 15 header and row controls fit without overlapping from 800×500 to 1440×900.

| Docked, 1280×800 | Small window with the inspector drawer |
|---|---|
| ![Docked](ui-03-docked-1280x800.png) | ![Inspector drawer](ui-03-narrow-inspector-drawer.png) |

### UI-04 — Welcome and project entry screens
- The app now opens on a **welcome screen**: New project, Open project (disabled until project storage exists, and it says so), Continue with the sample project, a searchable list of recent projects and a recovery notice. Both the list and the notice are clearly labeled as sample entries.
- **New project** asks for a name, checked by the same rule the core uses. A blank name shows the error right beside the field. **Create** opens an empty diagram with that exact name; **Cancel** or Escape goes back.
- **Four problems found and fixed:** the app would have **crashed on start** (a text style was missing on the very first frame); the name field **lost keyboard focus** as soon as its error appeared; the recovery notice looked like a real recovery; and "No selection" showed on the welcome screen.
- 17 new automated tests.

| Welcome | Invalid name | Created project |
|---|---|---|
| ![Welcome](ui-04-welcome.png) | ![Name error](ui-04-new-project-error.png) | ![Empty diagram](ui-04-created-empty-diagram.png) |

### UI-05 — Project panel and diagram tabs
- The project panel has a **search** and two sections: **Diagrams** (views of the model) and **Model** (classes shared by all diagrams). Each section says clearly when it is empty or when nothing matches.
- Picking a class that isn't on the current diagram **opens a diagram that shows it**. The inspector says which diagrams show the class, so it's clear that renaming changes it everywhere.
- **Tabs:** one per open diagram, each with a labeled Close button. Each tab remembers its own selection; closing a tab never deletes anything. Tabs that don't fit go into a **More tabs** menu.
- 15 new automated tests, plus new usability-audit scenes.

| Two tabs, dark theme | Search with no matches |
|---|---|
| ![Two tabs](ui-05-two-tabs-dark.png) | ![No matches](ui-05-search-no-match.png) |

- **Completed later (UI-06):** buttons, menus, tabs and lists no longer change anything directly. They send typed **intents** that one coordinator applies, which keeps behaviour consistent and testable. Every command also has a written **contract**: what it needs, what it changes, who carries it out and how it interacts with undo, plus the test that covers it. A test fails if any contract points to a test that doesn't exist.

### UI-08 — Contextual inspector
- Editing in the inspector is now **safe**: a change is a draft until you press **Enter** (or move to another field), **Escape** cancels it, and an invalid value stays in the field with its error instead of being saved.
- The inspector adapts to what's selected: the **diagram** when nothing is selected; a **class** with its members, collapsed Appearance and Advanced sections; **several classes**, showing which values differ and refusing edits that would silently apply to all; or a **relationship**, with each end's role and multiplicity (checked: `1`, `0..*`, `2..5` and so on).
- **Ctrl+click** adds classes to the selection; the sample now has a relationship ("Customer places Order") drawn on the canvas.
- **Five problems found and fixed**, including the name field changing the class on every keystroke, an arrow symbol missing from the font, and relationship labels drawn on top of each other.
- 14 new automated tests and new usability-audit scenes.

| Relationship (dark) | Several classes | Invalid name |
|---|---|---|
| ![Relationship](ui-08-relation-dark.png) | ![Mixed](ui-08-mixed.png) | ![Invalid draft](ui-08-invalid.png) |

### UI-11 — Notation contract
- One table now says what each diagram type offers: its tools, inspector sections, allowed relationships, and how its canvas behaves. The tool row, the inspector and the new-project form all read it.
- Only **Class diagram** can be created today. Flowchart and Use Case (M4), and Entity-relationship and Sequence (M5), are described for planning but never offered. Sequence diagrams keep their own written rules (messages ordered top to bottom, aligned lifelines), so they won't be forced into the free-form class canvas.
- 8 new automated tests.

## What is next
1. **UI-07 — canvas states:** blank, selection, move/resize/connect previews, invalid targets, zoom and fit.
2. **UI-09 — dialogs, status bar and validation drawer**, then **UI-02** font files and **UI-10** keyboard and accessibility checks (with R0-04 on Windows).
3. **In parallel (needs a Windows desktop):** R0-04 text and accessibility matrix — Arabic and mixed text, IME, Narrator/NVDA and 100/150/200% scaling — then the R0-05 framework decision.

## How to try it
From `app/`:

```sh
cargo run -p diagram-desktop --locked
```

Click a class or a name in the Model list, rename it in the inspector, press Ctrl+K to search commands and classes, and switch Light / Dark / System in the bottom-right corner. Run all checks with `scripts/check.ps1` on Windows, or the commands in the [workspace README](../../README.md).
