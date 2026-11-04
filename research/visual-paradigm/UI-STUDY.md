# Diagram creation and UI investigation

Target: `C:\Program Files\Visual Paradigm CE 18.1\bin\Visual Paradigm.exe`. Date: 2026-11-04.

This pass extends the initial static survey with direct interaction in the running CE application. It focuses on the diagramming product and the UI an independent rebuild needs. It maps every installed diagram constant but does not claim that every editor or feature was exercised.

## Findings that change the rebuild plan

1. **A shared canvas is only the foundation.** Class, sequence, and ER editors use substantially different semantic properties and interaction rules.
2. **The product contains more than native modeling.** The creation catalog also advertises web-based diagrams and AI creation. Those are separate feature paths with service dependencies, even though they appear in one shell.
3. **Navigation has several representations.** The model tree, diagram browser with thumbnails, current diagram header, selection, and properties must remain consistent.
4. **Persistence separates semantics from presentation.** Both the new UI-saved SQLite project and installed API support this distinction; a relationship model and its connector view have different identities and fields.
5. **Feature visibility is not entitlement proof.** The CE ribbons and menus show entries for code, databases, AI, agile, ITSM, and exchange operations. Only invoked and successfully completed operations count as tested support.

## Scope and evidence status

| Question | Status | Evidence |
|---|---|---|
| What is the EXE's role? | Structurally answered | PE launcher plus Java entrypoint/classpath and JAR inventory from initial survey; no native decompilation |
| Which diagram types are represented in the installation? | Inventory complete | 101 shared API constants, classified once into 17 families |
| How is the desktop workspace organized? | Directly observed | Running CE workspace, palette, side panels, ribbons, browser |
| How are diagrams created? | Directly observed for three native types | Search → type → blank/template → name/location/description → editor |
| Do class creation, connections, movement and undo work? | Directly exercised | Customer, Order, association, movement and undo |
| Does sequence creation have specialized behavior? | Directly exercised at basic level | Two lifelines and a message with activation bars and displayed number `1:` |
| What does ER modeling expose? | Directly observed and basic entity creation exercised | Physical/conceptual/logical selector; entity specification and column table |
| Is a new 18.1 project SQLite? | Confirmed | Read-only query of `ui-research.vpp`, integrity check `ok` |
| Can CE export these diagrams to XML? | Confirmed for selected-diagram Simple export | `xml-export/project.xml` and `data.zip` were produced |
| Do every diagram family and all ribbon actions work? | Unresolved | API/documentation coverage is broader than executed workflows |
| Are native/private rendering algorithms recovered? | Unresolved | No connected REA MCP/Ghidra analysis in this session |

## Observed UI map

The workspace initially opened as an empty `untitled` project. It was used to create a disposable research project; existing user documents were not opened or edited.

### Main shell

- Ribbon tabs: Dash, Project, ITSM, Agile, Diagram, View, Team / Unified Platform, Tools, Modeling, Window, Help.
- Left side tools: Diagram Navigator, Model Explorer, Property, Diagram Backlog.
- Current diagram header and notation-specific palette.
- Large central canvas with selection handles and context controls.
- Separate Project Browser with Diagrams, Model Structure, Files, My Recent, Team Recent, search, grouped thumbnails and a close control.
- Modified-state `*` in the window title; save completion feedback in the status area.

### Creation wizard

The New Diagram dialog contains a search field and diagram cards. Recent native types rise to the top after use. The footer identifies VP Online web diagram technology; this does not imply every listed editor is online.

For Class, Sequence and Entity Relationship diagrams, the tested flow was:

1. Find and select the diagram type.
2. Choose Blank or a template. Template previews may load asynchronously.
3. Set diagram name, containing location and description.
4. Open a native editor with a notation-specific palette.

An optional AI-generation prompt appeared on new canvases. It was dismissed; no AI request or external data submission was performed.

### Shared editing controls

The Diagram ribbon contains New, Undo, Redo, Copy, Paste, Select All, Format, Format Copier, Search, Group, Ungroup, Alignment, Distribution, Layout, Link, Jump and Bookmarks. The View ribbon exposes Project Browser, Panes, Layers, Styles, Full Screen, zoom controls, Grid, spell checking and model/connector highlighting controls.

The class context menu includes Add, Open Specification, Stereotypes, Model Element Properties, Sub Diagrams, parent creation, clipboard commands, Delete, Duplicate, Selection, Order, Grouping, Styles and Formatting, Presentation Options, Layer, and Related Elements. The menu is observed; deletion, duplication, and every submenu were not executed.

Important documented workflows such as Paste View versus Paste Model and master versus auxiliary containment are tracked in [DOCUMENTED-WORKFLOWS.md](DOCUMENTED-WORKFLOWS.md), not reported as tested here.

## Class editor experiment

Created `REA Class Study`, then `Customer` using palette selection and canvas placement. Created `Order` by dragging the Class palette item onto the canvas. The default name is immediately editable inline, and Return commits it.

Selected Association and dragged from Customer to Order. Moving Customer changed the connector geometry while retaining its endpoint connection. Ctrl+Z returned Customer to its previous position and restored the connector route.

Class Specification has General, Attributes, Operations and Receptions tabs. General contains name, parent, visibility, description, Abstract, Leaf, Root, Active, Final specialization and Business model fields. Attributes has an ordered table with Name, Classifier, Visibility, Type and Initial Value, plus add/remove/specification controls and inherited-member visibility.

Attribute Specification exposes name, classifier, initial value, multiplicity, Ordered/Unique, visibility, type, type modifier, scope, aggregation, documentation, derived/read-only/ID options and accessor flags. The dialog was inspected and canceled; an attribute was not committed.

**Rebuild implication:** use typed member records and property schemas. Treat compound creation and movement as command transactions. Separate semantic relationship endpoints from connector attachment points.

## Sequence editor experiment

Created `Sequence Diagram1`, two default-named lifelines, and a Message by dragging between their dashed lines. The result displayed number `1:` and activation bars. This demonstrates basic numbering and activation creation, not every sequencing or nested-fragment rule.

Observed palette: Lifeline, Message, Duration Message, Create Message, Self Message, Recursive Message, Found Message, Lost Message, Reentrant Message, Alternative Combined Fragment, Interaction Use, Frame, Actor, Concurrent, Continuation, Gate, Duration Constraint, Element Group, Note, Anchor and Constraint.

**Rebuild implication:** the temporal ordering, activation spans, frames, lifeline headers, message semantics and special message types need a sequence-specific editing strategy. Advanced reordering, return messages, numbering modes and combined-fragment behavior remain untested.

## ER editor experiment

Created `Entity Relationship Diagram1` and a default Entity. The editor offers Conceptual, Logical and Physical model modes; Physical was selected by default in this test.

Observed palette includes Entity, View, Sequence, one-to-one/one-to-many/many-to-one/many-to-many relationships, Stored Procedures, Stored Procedure ResultSet, Triggers and Generalization, plus annotations and generic shapes. Presence in the palette does not establish that every operation is licensed or implemented in the same way.

Entity Specification contains General, Columns, Indices and Constraints. General includes domain, full-text index, discriminator/version columns, schema, model level, DDL clauses and documentation. Columns includes primary-key constraint settings and Name, Type, Length, Primary Key, Nullable and Unique columns, with ordering and add/specification/remove controls. No columns, keys or relationships were committed in this experiment.

**Rebuild implication:** ER diagrams need typed column/key/index/constraint models, cardinality-aware relationship editors and model-level metadata. Do not model a database table as an unstructured multiline label.

## Other program feature surfaces inspected

| UI surface | Observed entries | Tested depth |
|---|---|---|
| Project | New/Open/Save/Close/Print, export/import, properties | Saved project and completed XML export |
| Export menu | Project, XMI, XML, default formatting, image variants, Excel, pipeline/Confluence/Jira entries | XML route completed; other entries only observed |
| Tools | Document Composer, Publish Project, DB, Hibernate, Code, API, Quality Control, AI Diagram Generation, More Features | Ribbon inspection only |
| Modeling | Requirement List, Storyboard, Animation, Profile | Ribbon inspection only |
| Agile | Scrum canvases, Nexus, story map/list, affinity table, sprints | Ribbon inspection only |
| ITSM | TOGAF ADM, Project Management Process, Business Motivation Model | Ribbon inspection only |
| Team / Unified Platform | Select Repository | Ribbon inspection only; no service connection |

This survey records the whole-program feature entrypoints visible in this installation, while keeping execution coverage explicit.

## Native save and XML exchange evidence

Research project: `ce-evidence/ui-research.vpp`. It contains the three diagrams and their test elements. `inspect_ui_project.py` reads it without modifying it and records hashes, file/WAL stability, schema, model/view identities, and literal payload references.

Association definitions refer to the source/target class model IDs. The separate association view contains shape endpoint references, route points and presentation settings. Sequence Message records refer to lifelines/frame, while visual elements retain separate representations.

The XML dialog supports Traditional versus Simple structure, whole-project versus selected-diagram/model scope, subdiagram inclusion, and optional image export. This run selected all three diagrams, used Simple structure, and left optional image export and whole-project export off. It produced `project.xml` and `data.zip` in the requested local output directory. Import roundtrip and image export are not validated.

## What UI we need to build

Proposed arrangement, based on observed responsibilities rather than a pixel copy:

```text
+---------------------------------------------------------------------+
| Project actions | Diagram actions | View | Modeling | Export          |
+----------------+-----------------------------------+----------------+
| Project/model  | Open diagram tabs + breadcrumb     | Properties     |
| tree & search  +---------+-------------------------+|                |
|                | Palette | Canvas                  || Semantic fields|
| Diagram browser| by type | nodes, edges, labels     || View/style     |
| with previews  |         | handles, guides, menus   || Documentation  |
+----------------+---------+-------------------------++----------------+
| Selection/status | Validation results | Zoom | Save/recovery          |
+---------------------------------------------------------------------+
```

Implement shared shell, navigator, command registry, searchable diagram picker, template picker, canvas primitives, contextual tools and property-editor framework. Register separate notation definitions and specialized interaction strategies. This lets all families use a common foundation without forcing every diagram into the same data model or gestures.

## Remaining validation before parity claims

Class members and association-end properties; multiple views of one model; clipboard identity rules; master-view containment; resize and endpoint pinning; layout and alignment; keyboard/accessibility behavior; delete semantics; all sequence variants; ER columns/FKs; use-case/activity/state/SysML/requirements workflows; online diagram boundaries; export rendering; import/reopen roundtrips; project recovery; large documents; plugin and collaboration behavior.

The complete family inventory is [DIAGRAM-CATALOG.md](DIAGRAM-CATALOG.md). Proposed implementation order and requirements are [PLANNING-BACKLOG.md](PLANNING-BACKLOG.md). Screenshots are available in the local [UI gallery](UI-GALLERY.html).
