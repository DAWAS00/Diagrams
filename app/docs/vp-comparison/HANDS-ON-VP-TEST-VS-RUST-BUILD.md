# Hands-on Visual Paradigm test versus what we have built

Date: 2026-11-06. This report joins three things: the earlier static study of the Visual Paradigm (VP) CE 18.1 install, a hands-on test of the running application, and the current state of the Rust code in `app/`.

**Honesty rules for this report.** VP statements come from what I saw on screen or in files during this session. Rust statements come from reading source files, the roadmap and the project docs. **I did not build or run the Rust app in this pass**, so "built" means "present in source and described as tested by the project", not "I verified it". Anything I did not test is listed under [Not covered](#not-covered).

## 1. How VP works (install level)

| Finding | Evidence |
|---|---|
| `bin/Visual Paradigm.exe` is a 64-bit native launcher built with install4j/exe4j. It finds a JVM, loads `jvm.dll` and starts Java. | PE header and strings in the executable |
| The launcher passes `-Xmx1024m -Xms256m` and reads extra options from `%APPDATA%\VisualParadigm\vplauncher.vmoptions`. `scripts/VisualParadigm.bat` starts class `RV` directly with `-Xmx768m`. | Launcher strings; the batch file |
| The runtime is a bundled JRE 11.0.16.1 with JavaFX. | `jre/release` |
| The application is Java: `vpplatform.jar` holds 36,515 classes, and `lib01`–`lib09` and `ormlib` hold libraries. The classpath lists `lib01`–`lib20`, but only some of those jars are installed. | Jar listing; launcher classpath string |
| The launcher's main class name was not read. No decompilation was done. | Not established |

The earlier research notes agree with all of the above. The `.exe`/`.bat` heap difference is new detail.

## 2. What I tested in the running VP

All work happened in an open project, `ui-research`, with nothing saved. Test elements remain in the open project until it is closed.

### 2.1 Class diagrams
- **Create a class:** pick **Class**, click the canvas, type the name. The shape appears at once and the name is already in edit mode.
- **Duplicate name:** typing an existing name (Order) raised **Change to Auxiliary View**. Accepting it makes the new shape a second view of the existing model.
- **Members:** add them from the right-click **Add** menu (Attribute, Attribute with getter and setter, Operation, Constructor, Reception, Template parameter). The Class Specification dialog has General, Attributes, Operations and Reception tabs, a parent field, visibility, and the flags Abstract, Leaf, Root, Active, Final specialization and Business model. "Show inherited" is on by default.
- **Relations:** Association, Generalization, Dependency and others from the palette. The association dialog has name, visibility, role, element, multiplicity (0, 0..1, 0..*, 1, 1..*, *) and navigable for each end. Aggregation is not in that dialog. Multiplicity and name appeared on the canvas as `1`, `1..*` and `contains`.
- **Generalization direction:** drag from the parent to the child. The hollow triangle sits at the drag start.
- **Move:** dragging a class carries its connectors with it. Ctrl+Z undid the move.
- **New member popup:** adding an attribute asks "Show New Member in Other Views?" which confirms one model can have several views.
- **Copy/paste:** the **Copy** submenu offers copy within VP, image (JPG, EMF), XML (simple, traditional) and Copy to Diagram. Paste within VP made a second view of the same model, not a second model. The context menu has separate **Paste View** and **Paste Model Element** items.
- **Delete:** asks for confirmation and offers **Referenced Model Elements**, so deleting a view and deleting the model are separate.
- **Layout:** Layout menu with Hierarchical, Orthogonal, Tree, Circular, Organic, Compact and connector routing. Hierarchical put the parent above the child. Ctrl+Z reverted it.
- **Zoom and grid:** Zoom In and Zoom 100% worked; the grid is a toggle.

### 2.2 Sequence diagrams
- Lifelines, messages and self messages worked. Messages got hierarchical numbers automatically (1, 1.1, 1.2), and the activation bar grew over nested calls.
- A combined fragment (`alt`) created two operand regions and a guard label, and could be named.
- The palette also has duration, create, recursive, found, lost and reentrant messages, interaction use, frame, actor, gate and others. I did not exercise those.
- Pressing Enter after placing a lifeline opens its specification dialog; the dialog works.

### 2.3 Entity-relationship diagrams
- An Entity Specification dialog has General, Columns, Indices and Constraints tabs. A column dialog has name, domain, type, length, default, check, generated, collation, and Primary key, Nullable, Unique and Index flags.
- A One-to-Many relationship automatically added a `CustomerId integer(10)` foreign-key column to the child entity.
- A Conceptual, Logical and Physical model switch sits on the canvas.

### 2.4 Free shapes
Rectangle, oval and note drew by dragging. A generic connector attached them and followed when a shape moved. I could not get typed text into the free shapes.

### 2.5 Application surface
- Ribbon tabs: Dash, Project, ITSM, Agile, Diagram, View, Team/Unified Platform, Tools, Modeling, Window, Help (Help not opened).
- Diagram Navigator strip lists every diagram as a thumbnail.
- Search has a bottom bar with Highlight All, Match Case and Advanced; "invoice" found 2 hits.
- Link shows a copyable diagram address. Bookmarks has Add, Bookmarks and Organize.
- **Application Options** has General, Diagramming, View, User Path, File Types, Spell Checking and Keys. The Keys page lists every command with a search box and Import/Export, so every command can be rebound.
- Several Tools buttons (checked: Doc Composer) show "Non-Supported Feature, available in Visual Paradigm Standard". Diagram creation also labels some types "Standard Edition".
- New diagram templates showed "Loading…", so they appear to need the network.
- Image export wrote a JPG with a "Powered By Visual Paradigm Community Edition" mark.
- Starting an app launch twice produced "Workspace in use, please choose a different one."

### 2.6 Annoyances seen
- An "AI Diagram Generation" popup appears on every new diagram.
- Inline editing of rows is easy to mis-hit; keyboard shortcut Alt+Shift+A did not work in my attempt.
- Some member text with special characters did not enter.

## 3. What the Rust project has built

From `roadmap.json` (updated 2026-11-05): **14 tasks done, 2 in progress, 41 not started** of 57. Done: R0-01 to R0-03 (workspace, native shell, bundled SQLite); UI-01, 03, 04, 05, 06, 08, 11; R1-01 to R1-04. In progress: R0-04 (text/accessibility) and UI-02 (typography).

| Layer | What exists in source |
|---|---|
| `diagram-core` | Typed IDs; `Name` (blank rejected); `Visibility` (+, -, #); `Multiplicity` parser; `Attribute`, `Operation`, `Member`; `Element` with a name shared by all views; `Relation` (association or generalization) with role and multiplicity per end; `NodeView`, `EdgeView`, `Diagram`, `Document`; validation (self-generalization, generalization cycle, duplicate generalization, ends on generalization, self-association unsupported); reversible commands (`CreateClass`, `RenameElement`, `AddAttribute`, `AddOperation`, `RemoveMember`, `MoveMember`, `Connect`, `SetEnds`, `MoveViews`, `RemoveViews`, `ShowInDiagram`, `DeleteElement`, `AddDiagram`, `RenameDiagram`, `DeleteDiagram`); `delete_impact`; history with undo, redo, labels and saved-state identity. |
| `diagram-canvas` | Viewport transforms and picking (small: about 230 lines). |
| `diagram-storage` | SQLite round trip proven by a smoke test; no document save/load yet (18-line file). |
| `diagram-desktop` | egui shell: welcome and new-project screens, project panel, diagram tabs, command catalog and search, contextual inspector with drafts, mixed values and relation ends, status bar, theme tokens, usability audit tests. The notation table delivers **Class diagram only**; flowchart and use case are planned for M4, ER and sequence for M5. |

Typed UI intents currently cover selecting and renaming classes, setting relation fields, opening, activating and closing tabs, theme and drawers. **I did not find intents that create classes, add members or draw relations from the canvas yet.** The core commands exist, but the screen path to them is not built.

## 4. Side-by-side comparison

Legend: **Built** = in source and described as tested; **Core only** = domain code exists, no UI path found; **UI only** = screen exists, not backed by real behavior yet; **Planned** = in the roadmap, not started; **Not planned** = outside the v0.1 plan.

| Feature (VP tested) | VP behavior | Rust today | Gap / note |
|---|---|---|---|
| Create class | One click, name in edit mode | Core only (`CreateClass`) | Needs canvas tool and inline naming |
| Model shared by views | Auxiliary view on same name; Paste View versus Paste Model Element | Core only (`ShowInDiagram`, `views_of`, shared `Element.name`) | Same idea as VP. Our explicit "Show in diagram" versus "Duplicate model" is a deliberate redesign |
| Rename | Inline; duplicate name warning offers auxiliary view | Built in UI (rename field) and core | Need an equivalent duplicate-name decision |
| Attributes and operations | Right-click Add; popups for other views | Core only (`AddAttribute`, `AddOperation`, `RemoveMember`, `MoveMember`) | No member UI path found |
| Visibility | Specification dialog | Core (+, -, #) | VP also has package/other values; confirm needed set |
| Association with multiplicity and name | Dialog with per-end role and multiplicity | Core only (`Connect`, `SetEnds`, multiplicity parser); inspector can edit relation fields | Fixed list of VP multiplicities versus our parser: parser is more flexible |
| Generalization with direction rule | Drag parent to child; triangle at drag start | Core (child is source, parent target); validation blocks self, cycles, duplicates | VP's drag direction is unusual; decide ours and write it down |
| Dependency, aggregation, composition, realization, association class | Palette items | **Not in plan for v0.1** (only association and generalization) | Largest class-diagram gap |
| Abstract/final/other class flags, parent field in dialog, stereotypes | Present | Not planned for v0.1 | Inspector has an Advanced section reserved |
| Move with connectors following | Yes | Core (`MoveViews`), canvas picking | Routing is simple; VP has many layout modes |
| Undo and redo | Ctrl+Z worked; Undo/Redo ribbon | Built in core with labels; command catalog | Strong point: labelled, saved-state aware |
| Delete view versus delete model | Dialog with "Referenced Model Elements" | Core (`RemoveViews`, `DeleteElement`, `delete_impact`) | Same split; ours has an impact preview |
| Delete confirmation / "don't ask again" | Present | Impact report in core | UI flow not built |
| Auto layout | Seven layout styles | Arrange tool listed, not delivered | Planned |
| Zoom, grid | Zoom in/out/100%/region, grid | Viewport transforms in canvas crate | UI controls not found |
| Multiple diagrams, thumbnails | Navigator strip | Built (project panel, diagram tabs) | Ours uses tabs and a project panel |
| Search | Bottom bar with highlight | Command search built; no in-diagram search found | Planned search |
| Keyboard shortcuts | Fully rebindable table with import/export | Command catalog and shortcut rules (`ui/commands.rs`) | Rebinding not planned |
| Sequence diagram | Lifelines, auto-numbering, activations, fragments, many message kinds | **Planned, M5**; temporal requirements written down | Entire notation outstanding |
| ER diagram | Columns, PK/FK, auto foreign key on relationships, model level switch | **Planned, M5** | Entire notation outstanding |
| Free shapes and generic connectors | Rectangle, oval, note, polygon, text, line | Not planned (Note tool listed in class notation) | Decide whether v0.1 needs free shapes |
| Specification dialogs | Tabbed dialogs per element | Inspector panel instead | Deliberate redesign; the inspector must still cover the properties the dialogs expose |
| Save / open project | `.vpp` project | SQLite round trip proven; no document persistence yet | R2 work |
| Export | Project, XML, image (JPG, PNG, EMF), Excel, Confluence, Jira | SVG export planned for v0.1 | Image formats, XML and others are not planned |
| Editions and licensing | Many features gated; online templates; AI popup | None | Our app has no gating |
| Keyboard accessibility, focus, minimum control size, tests | Not measured | Strong written rules (`UI-RULES.md`), headless tests and audit | Not compared head-to-head |
| Performance, memory, startup | Not measured (JVM-based) | Not measured | No claim possible |

## 5. What this adds to earlier conclusions

- **Confirmed by use:** shared model with several views, view-versus-model delete, labelled undo, connectors that follow moves, hierarchical sequence numbering. These match the architecture you chose.
- **New concrete requirements the earlier notes only guessed:**
  - The duplicate-name prompt that offers a second view.
  - The "Show New Member in Other Views?" decision when adding members.
  - Automatic foreign-key creation in ER one-to-many.
  - Automatic message numbering and activation growth in sequence diagrams.
  - A visible hint about which end of a generalization is parent.
- **Still unresolved:** whether renaming a model in one view updates the other view (the popup suggests it does; I did not test it directly), independent styling per view, XML re-import, crash recovery.

## 6. Recommendations

1. **Wire the existing core to the canvas first.** The most visible gap is the path from tool to command: create class, add member, connect, move, delete. The commands are done; the intents and canvas tools are not.
2. **Decide the duplicate-name rule** and show it in words, since VP's prompt is the closest real behavior to copy.
3. **Decide whether aggregation, composition and dependency belong in v0.1.** Users will draw them in any class diagram; today only association and generalization exist.
4. **Keep the inspector but audit it against VP's specification dialogs** so no property disappears silently.
5. **Write the generalization direction into the UI** (label or tooltip), because even VP's drag direction is non-obvious.
6. **Plan sequence and ER as separate milestones** using the behaviors in section 2, especially auto-numbering and automatic foreign keys.
7. **Record sizes for undo, save and large diagrams** before claiming any performance advantage over VP.

## Not covered

- I did not run, build or test the Rust app; section 3 comes from source and docs.
- VP features not exercised: Group and Alignment/Distribution, Format Copier, the Property pane in depth, Print, Import, Team/Unified Platform, ITSM and Agile diagrams, most Tools buttons, VP Standard/Enterprise features, plugins, repository collaboration, file reopening and XML re-import.
- Text entry into free shapes failed in my attempts, so I cannot say how VP edits shape text.
- No decompilation or tracing of the Java platform was done.

## Evidence and files

- Test run on Windows 11 with VP CE 18.1; screenshots were viewed during the session but are not stored in the repo.
- Images exported by VP during the test (both carry VP's Community Edition mark):
  - [Class diagram built in the test](images/vp-shop-class.jpg)
  - [ER diagram built in the test](images/vp-er-export.jpg)
- Prior evidence: [VALIDATION-GAPS.md](../../../plan-rust-diagram-editor/comparison/VALIDATION-GAPS.md), [FEATURE-DIFFERENCES.md](../../../plan-rust-diagram-editor/comparison/FEATURE-DIFFERENCES.md), [comparison README](../../../plan-rust-diagram-editor/comparison/README.md), and [research/visual-paradigm](../../../research/visual-paradigm/README.md).
