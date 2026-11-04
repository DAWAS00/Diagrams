# Diagram editing and UI: documented workflow reference

Research date: 2026-11-04. Target under study: Visual Paradigm CE 18.1 desktop on Windows. This document uses official vendor documentation only. **Everything below is documented behavior or a proposed planning consequence, not a claim of local UI observation.** The main investigation records local observations separately.

## Scope and product boundaries

The current edition table includes core UML, SysML, ER and flowchart tools in Community Edition. It leaves BPMN, DFD, C4, and several enterprise workflows outside CE. The table also includes online and service-dependent features, so it is not an offline desktop checklist. See the [edition comparison](https://www.visual-paradigm.com/support/edition-comparison.jsp).

VP distinguishes casual online drawing from structured desktop modeling, and offers conversion/import paths between them. Consequently, a browser editor screenshot or a cloud template catalog does not establish the native desktop editor's behavior. [Online versus model-based diagramming](https://www.visual-paradigm.com/features/online-diagram-editor/).

Most user-guide pages are version-neutral or older than 18.1. They are useful behavior specifications to test against the installed build; exact menu labels, option defaults and CE availability must be confirmed locally.

## Workspace and UI surfaces

The documented desktop editor has a breadcrumb showing model hierarchy and active diagram, a package header in applicable notations, a notation-specific diagram toolbar, the diagram itself, and a context-sensitive action bar. Palette utilities include navigation/spacing tools, while notation tools differ by diagram family. Breadcrumb segments can navigate into the model structure. [Diagram editor](https://www.visual-paradigm.com/support/documents/vpuserguide/12/512/85161_diagramedito.html).

The application toolbar groups commands into functional menus. Relevant groups are Project (file lifecycle and exchange), Diagram (editing and arrangement), View (panes, layers, zoom and grids), Modeling (specialized tools), and Window (options and workspace management). This catalog also documents paid and service-backed commands; reproducing every listed button is not equivalent to reproducing CE. [Toolbar reference](https://circle.visual-paradigm.com/docs/visual-paradigm-getting-started/user-interface/toolbar/).

Project Browser offers diagram thumbnails, model structure, supplementary files and recent-work perspectives. Search can find diagrams, model elements and views; the result opens either a diagram or a model's specification. Team recent views depend on collaborative project activity. [Project Browser](https://circle.visual-paradigm.com/docs/visual-paradigm-getting-started/user-interface/project-browser/).

Diagram Specification is a settings dialog with General, Grid Setting, References, Project Management and Comments tabs in the documented UI. It can be opened from diagram-background context menus. Diagram metadata, background/zoom, grids and references are thus different concerns from per-model semantic properties. [Diagram Specification](https://www.visual-paradigm.com/support/documents/vpuserguide/1283/27/41579_diagramspeci.html).

## Shared create/connect interaction

Resource Catalog is a contextual shortcut around a source shape. Dragging its handle to empty canvas offers valid related-shape/relationship combinations; dropping onto an existing shape offers connector choices for those endpoints. Creating a related shape can create both node and relationship before inline naming. Other documented capabilities include quick-connect name search, attached part shapes, multi-element branch structures and inserting a node into a supported connector. Frequent resources can be ranked. [Resource Catalog](https://circle.visual-paradigm.com/docs/diagramming-and-modeling/diagram-editing/resource-catalog/).

Planning consequence: the equivalent UI needs a source/target compatibility registry and compound editor commands. A universal menu listing every connector would lose a major part of VP's editing workflow. One undo transaction should be able to encompass creation of related models, their views and connection geometry. This transaction granularity is a proposed design; it remains to be measured in VP locally.

## Class-diagram workflow

The documented creation wizard selects diagram type, name/description and containing model. Users place classes from the palette; contextual resources create associations, aggregation, composition or generalization. Class member editing includes adding attributes/operations, continuous entry with Enter, member reorder/move/copy, and member-level relationship endpoints. Multiplicity is editable at association ends. Visibility options exist at workspace, diagram, class and individual-member scope. [Class diagram guide](https://circle.visual-paradigm.com/docs/uml-and-sysml/class-diagram/how-to-draw-a-class-diagram-in-uml/).

Planning consequence: class boxes contain individually addressable members, not one formatted label. Visibility must be presentation state rather than deletion of hidden members. Association endpoints need their own names, multiplicities and semantics; a single edge label field is insufficient.

## Sequence-diagram workflow

Sequence diagrams document a separate quick editor for lifelines and messages, linked selection between list and canvas, specialized message types, message reordering and direction reversal. Message numbering can be diagram-based or frame-based, with single-level or nested numbering. [Sequence diagram guide](https://circle.visual-paradigm.com/docs/uml-and-sysml/sequence-diagram/how-to-draw-a-sequence-diagram-in-uml/).

An official tutorial also creates a sequence diagram as a subdiagram from a class and models interaction between instances of classes. This establishes a documented navigation relationship between structural and behavioral models, rather than isolated drawing files. [Sequence-from-class tutorial](https://www.visual-paradigm.com/tutorials/how-to-draw-uml-sequence-diagram.jsp).

Planning consequence: sequence editing requires order-aware interactions, lifeline/activation geometry, frame ownership and numbering rules. Generic free-positioned node-and-edge editing will need specialized tools for this family.

## ER-diagram workflow

The documented ER editor selects conceptual/logical/physical data-model context. Entities support inline column entry, primary/composite keys and column specifications. Resource Catalog creates cardinality-specific relations; a many-to-many relation can introduce a linking entity and two one-to-many relations. The guide also describes identifying versus non-identifying relationships. Database/DDL generation described on the same page is broader product functionality and must not be assumed enabled in CE. [ER drawing guide](https://www.visual-paradigm.com/support/documents/vpuserguide/3563/3564/85375_drawingentit.html).

ER options cover presentation of schema/constraints and default column typing/nullability. Some options relate to higher-edition synchronization workflows. [ER options](https://circle.visual-paradigm.com/docs/project-options/diagramming/erd-orm/).

Planning consequence: entities own typed columns and key groups; relationship cardinalities and generated key behavior need explicit rules. Distinguish editing a logical design from applying database changes.

## Flowcharts and requirements

The flowchart tutorial uses a creation wizard with blank/template choice, palette placement, Resource Catalog creation of a connected next process, terminators and format editing. [Flowchart tutorial](https://www.visual-paradigm.com/tutorials/flow).

The newer release announcement explicitly labels its enhanced flowchart editor as a desktop feature, but its prose lists editions inconsistently with that label. Treat detailed validation claims as documentation to test, not verified CE behavior. [Flowchart release note](https://updates.visual-paradigm.com/releases/visual-paradigm-flowchart-creator/).

The requirement-diagram guide includes requirements, relationships and test-case modeling, with customization and test-plan topics. Those adjacent features can have separate edition boundaries. [Requirement guide index](https://www.visual-paradigm.com/support/documents/vpuserguide/94/158_requirementd.html).

Planning consequence: retain separate feature flags and scope labels for a notation, its rich specification tools, generators and simulations. The fact that a diagram can be created does not imply every accompanying workflow is available.

## Shared editing semantics with architectural impact

The desktop guide distinguishes **Paste View**, which creates another appearance of the same model, from **Paste Model**, which duplicates the semantic model. It also describes alignment guides during movement/resizing. These features require identity-aware clipboard payloads and geometry-aware feedback. [General shape editing](https://circle.visual-paradigm.com/docs/diagramming-and-modeling/diagram-editing/general-shape-editing/).

The master/auxiliary-view documentation adds an important exception to a simple model/view split: moving a master view into a different container can update the semantic model hierarchy, while moving an auxiliary view does not. This should be an explicit behavior decision in our rebuild, not an accidental consequence of visual grouping. [Master and auxiliary views](https://circle.visual-paradigm.com/docs/diagramming-and-modeling/visual-modeling/understanding-master-view-and-auxiliary-view/).

Alignment can anchor to the last selected shape, with a different selection marker. Distribution and equal-dimension commands are exposed through toolbar and contextual group resources. [Alignment and distribution](https://www.visual-paradigm.com/support/documents/vpuserguide/1283/28/6688_alignanddist.html).

Selection configuration distinguishes shapes fully enclosed by a selection rectangle from shapes merely overlapped. Copy defaults and drag-content preview are also configurable. [Environment options](https://circle.visual-paradigm.com/docs/application-options/diagramming/environment/).

## UI planning inventory

This is a proposed component inventory derived from the preceding behavior, not a recovered implementation:

| UI surface | State it must handle | Validation experiment |
|---|---|---|
| Project/diagram creation wizard | Type, target model, metadata, template choice | Create each enabled CE family and record defaults |
| Project tree/browser | Models, diagrams, views, file references, search results | Reveal same object from tree, search and canvas |
| Canvas viewport | Pan/zoom, selection, grids, guides, bounds | Move/resize at multiple zoom levels |
| Notation palette | Node/edge types, grouped tools, utility modes | Compare palettes across UML/SysML/ER/flowchart |
| Resource Catalog | Source type, target type, valid actions, recent/frequent actions | Drag to empty space, existing target, nested container |
| Inline editor | Name versus member versus property cell | Rename, repeated entry, cancel/commit and selection handoff |
| Properties/specification | Model fields, diagram fields, endpoint fields, presentation overrides | Compare semantic edit with view-only edit |
| Sequence quick editor | Ordered lifelines/messages and linked selection | Reorder messages and inspect numbering/geometry |
| Context/group resources | Multi-selection anchor and available actions | Align to last selected; equal width; distribute |
| Clipboard/duplicate tools | Reuse model versus clone model | Paste view/model and inspect IDs after save |
| Style/layer tools | Inheritance, per-view override, visibility/locking | Apply style; move view between layers |
| Save/export dialogs | Scope, format options, persistence outcome | Reopen and compare geometry/identity; inspect SVG/PDF |

## Priorities and coverage gaps

Recommended order: (1) shared workspace and identity-aware class editing; (2) connector routing, selection, styles, clipboard and persistence; (3) sequence and ER specialization; (4) remaining CE notation families; (5) optional online, enterprise and automation features. This order is an engineering proposal, not a claim about VP development history.

Before estimating full parity, record a per-diagram checklist: enabled locally, blank creation, palette inventory, property schemas, connector compatibility, special interactions, save/reopen and export. Add separate rows for master/auxiliary hierarchy, view/model deletion, undo grouping, style inheritance, keyboard access and performance. The full family inventory is maintained in the separate diagram catalog.

No documentation search can establish the exact implementation of connector routing, hit testing, incremental layout, command transactions, migration, or rendering. Those remain empirical/reverse-engineering questions. Nor does this document claim to enumerate every feature of the entire commercial/online product family.

## Newly observed local project XML export

The main investigation subsequently exported a disposable project through the installed CE UI. This paragraph is **local export evidence**, unlike the documentation-derived sections above. A read-only parse of `ce-evidence/xml-export/project.xml` found separate `ProjectInfo`, `Models` and `Diagrams` sections. The file contains one class diagram, one sequence diagram (serialized as `InteractionDiagram`), and one ER diagram. Their visual objects comprise two class views, an association, two lifeline views, two activations, one message and one database-table view. Every inventoried view's `Model` reference resolves to a semantic element in this export.

The class association has semantic endpoint IDs and separate association-end records; its visual connector references view IDs. The sequence message has semantic endpoint IDs, activation references and sequence number `1`, separately from visual endpoints/geometry. Shapes carry view IDs, model IDs, bounds and Z-order; diagram records carry viewport/grid/presentation settings. Project defaults include antialiasing, font metrics, connection-point/connector styles, line jumps, sequence numbering, and default column type. Numeric/escaped-character option enumerations were recorded without guessing their meanings.

The companion `data.zip` contains three entries, inventoried without extraction. The reproducible read-only parser is `inspect_xml_export.py`, and its output is `ce-evidence/xml-export-inspection.json`, including hashes and selected geometry/identity references. **`ExporterVersion="16.1.1"` identifies the export implementation; it does not mean the installed application is version 16.1.1.** Local installation metadata identifies 18.1. This single export does not establish import round-trip compatibility, multiple-view behavior or complete notation coverage.
