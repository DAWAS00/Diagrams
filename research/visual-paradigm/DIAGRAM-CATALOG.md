# Diagram engine and notation catalog

Target: Visual Paradigm CE 18.1, installed public API. Inspection date: 2026-11-04.

This catalog maps the shared diagram platform, the specific semantic APIs, and the editor capabilities an independent implementation would need. It does not claim that all bundled editors are enabled in CE or that all editing behavior was observed. UI observations are maintained separately by the main investigation.

## Evidence and confidence

| Evidence | What it establishes | What it does not establish |
|---|---|---|
| `ce-evidence/diagram-types.txt` | 101 diagram-type constants in installed `IDiagramTypeConstants` | Enabled CE menus, 101 distinct working editors, or current UI labels |
| `ce-evidence/public-api-signatures.txt` | Public diagram/view/connector operations | Exact implementation, gesture mapping, undo grouping or actual runtime results |
| `ce-evidence/model-api-signatures.txt` | Model factory methods and shared model API | Availability of every type in CE |
| `ce-evidence/diagram-family-api-signatures.txt` | Specific class, association, sequence, ER, requirement and process interfaces | Complete metamodel or observed behavior |
| `ce-evidence/bundled-project-schema.json` | Separate model, diagram and visual-element storage in bundled sample | All payload fields or newly saved 18.1 project compatibility |

The vendor's [edition comparison](https://www.visual-paradigm.com/support/edition-comparison.jsp) separately lists CE support for UML, SysML, ER, flowcharts and common editing/export capabilities. BPMN, DFD, C4 and project XMI exchange are not marked for CE in the inspected table. Documented availability should still be tested in the local UI; neither marketing pages nor shared constants replace a runtime check.

## Shared engine capabilities confirmed in the API

| Subsystem | Concrete installed API evidence | Rebuild requirement / inference |
|---|---|---|
| Model-to-view binding | `createDiagramElement(diagram, model)`; `getMetaModelElement`; master-view operations | Stable semantic ID and separate view ID; one model may have multiple presentations |
| Geometry | `setBounds`, `setLocation`, `setSize`, axis/dimension getters/setters | Persistent world coordinates, resizing, viewport transformations |
| Visual containment | `addChild`, `removeChild`, `getParent`, `getGroupParent` | Separate semantic ownership and view nesting/grouping |
| Stacking | `bringToFront`, `bringForward`, `sendToBack`, `sendBackward` | Z-order commands preserved across save/reopen |
| Shape style | Fill/line models, presentation options, image paths, custom text, stereotype/icon/tag visibility | Style and label pipeline independent of semantic data |
| Member selection | `getSelectedShapeMembers` | Class attributes/operations or table rows can be interaction targets |
| Connectors | Route-point collection; from/to shape or connector; member IDs; endpoint offsets | Multi-segment routing, endpoints on nodes/edges/member rows |
| Anchors | Endpoint pin type and ratio; connect type; use-shape-center options | Attachment constraints survive node movement and resizing |
| Route styles | Rectilinear, oblique, curve, rounded variants, arc | Shared connector renderer with replaceable routing strategies |
| Crossings and labels | Line-jump options; label orientation including angle/upright behavior | Crossing decoration and label transforms distinct from route geometry |
| Layout | `autoLayout`, explicit layout modes and layout-option factories | Strategy boundary for hierarchical, orthogonal, circular, tree and organic arrangements |
| View navigation | Active/opened diagrams, selection/highlight/open operations | Model tree, tabs, selection synchronization and locate/highlight actions |

These signatures define operations, not guarantees about automatic behavior. For example, a `setFromPinRatio` method establishes representable anchor state; it does not prove exactly when dragging recomputes a pin.

## Family coverage and specialized editor requirements

The following requirements are design deductions based on API structure and diagram purpose. They should become behavior-capture checklists, not assertions of experimentally recovered implementation.

| Family | Main contents / likely interaction needs | Questions still requiring runtime or deeper schema evidence |
|---|---|---|
| UML structural | Class/object compartments, packages, component ports, deployment nesting, profiles | Member editing/visibility; association endpoints; semantic versus visual containment; profile validation |
| UML behavioral | Actions/control flow, use-case actors, messages/lifelines, state transitions, timing tracks | Lane constraints, activity token semantics, sequence ordering, state hierarchy, timing geometry |
| SysML / requirements | Blocks and ports, constraint properties, requirement relationships | Allocation/port rules, stereotype display, constraint evaluation versus drawing only |
| Services / code-oriented | Service contracts/interfaces/participants, WSDL/EJB-oriented objects | Exact model mappings, schema/code synchronization and generation scope |
| Database / ORM | Table/column compartments, keys, cardinality, database types, object-relational mappings | FK propagation, domain types, conceptual/logical/physical transformations, generated DDL |
| Business process / case | Tasks, events, pools/lanes, flows, subprocesses, conversation/case structures | Shape/relationship admissibility, lane reparenting, boundary events, simulation support |
| Data flow | Processes, data stores, external actors, flows, decomposition | Notation-specific glyphs, balancing rules, shared process identity across levels |
| Enterprise architecture | Layered architecture models, viewpoints, roadmaps, Zachman/form/process-step artifacts | Allowed relation matrix, viewpoint filters, roadmap axis constraints and form schemas |
| Rules / facts / decisions | Facts, terms, rules, decision models and tables | Expression language, decision-table execution, references versus standalone table data |
| Analysis / specification | Analysis objects, root causes, maturity/competitor models, rich-text and use-case descriptions | Structured fields, document links, rich text references, diagram-specific calculations |
| Planning / experience | PERT networks, breakdown trees, story maps, customer journey views | Scheduling/calculation behavior, columns/lanes, drag reordering, cross-view consistency |
| General visuals | Freeform shapes, flowcharts, mind maps, fishbones, org charts | Constrained versus free placement, branch layout, collapse/expand, image/stencil support |
| Structured canvases | Process and quality-control canvases | Fixed sections, card/sticky-note behavior, export pagination and template storage |
| Data / tables | Charts, spreadsheet-like sheets, matrices, ETL/data tables | Cell types/formulas, axis mapping, model-backed versus independent rows, editing transactions |
| UI / wireflow | Desktop/web/mobile mockups, UI controls, wireflow connections | Widget states, responsive/resizable frames, screen navigation, prototype execution |
| Documents / forms | Reports, documentation cabinet, forms and view editors | Document composition, property binding, layout constraints, template mechanics |
| C4 architecture | Context/container/component/landscape/deployment/dynamic views | Hierarchy reuse, boundary nesting, dynamic step ordering and deployment instance semantics |

## Concrete semantic observations

### Classes and relationships

`IClass` exposes `createAttribute`, `createOperation`, arrays/iterators, and explicit reordering for both member kinds. A class renderer therefore needs ordered compartments; these members are not just strings painted into a box. `IAssociationEnd` exposes aggregation kind, multiplicity (including a separate `IMultiplicity` object), visibility, type, representative/referenced attributes, and default value. Store relationship-end properties separately from the connector's geometric endpoint anchors.

### Sequence diagrams

`IMessage` exposes return-message and branch-parent references, source/target activation references, and message type. `ILifeLine` is a model type with children. Sequence editing needs an ordering-aware model and layout policy; a generic freeform arrow editor cannot alone encode message/activation relationships. Exact sequence numbering and automatic activation updates remain unknown.

### Database diagrams

`IDBColumn` exposes nullable and primary-key flags, type/type-name information, default value, identity seed/increment, and related attribute/synchronization fields. `IDBTable` exposes primary-key constraint information and member structures. The ER properties UI should edit typed column data; visible PK/FK glyphs and column text should derive from it. Whether a particular CE operation generates or synchronizes foreign keys is a separate runtime question.

### Requirements and processes

`IRequirement` exposes text as a semantic property. `IBPTask` exists as a substantial typed interface, corroborating that business-process objects are represented beyond visual rectangles. That interface does not establish CE access to BPMN or simulation.

## Editor UI that follows from the structure

An independent implementation should support a diagram-type chooser, a notation palette, a canvas, a model/project navigator, active diagram tabs, a contextual properties surface, and common commands. This is a proposed UI decomposition; compare it with the observed local workspace before finalizing placement or labels.

Properties should be grouped by scope: semantic object properties, relationship-end properties, diagram/view properties, and style properties. Selection should expose the correct scope. A single visual selection can involve both a view and its model; moving should target the view, while renaming normally targets the model. Canvas menus, keyboard shortcuts and contextual handles should all issue the same underlying commands.

## Prioritization for an independent rebuild

1. Complete the shared engine with class diagram modeling, multiple views, connectors, undo/redo, storage and export.
2. Add use-case/activity and ER editors to prove notation registration, containment and member-row connectors.
3. Add sequence and requirements editors to test ordering-aware layouts and typed cross-references.
4. Expand specialized families only after documenting actual requested workflows. Shared constants alone are insufficient reason to implement all 101 entries.

## Coverage status

All 101 constants are classified below and in `ce-evidence/diagram-families.json`; no constants were omitted or duplicated. Shared visual and selected semantic APIs are inspected. Local CE enablement, complete palettes, gestures, dialogs, templates, import/export behavior and specialization rules remain separate runtime investigation work. This is a complete constant inventory, not a complete feature audit.

## Follow-up: newly saved CE 18.1 class project

The main investigation created and saved `ce-evidence/ui-research.vpp` through the local CE UI. The first read-only inspection confirms SQLite storage for this newly saved project, not merely the older bundled sample. `PRAGMA quick_check` returned `ok`, and the main file and WAL identity were unchanged during inspection. The rerunnable `inspect_ui_project.py` writes `ce-evidence/ui-project-inspection.json`, including file SHA-256, schema, model/view records, bounded definition prefixes, and literal serialized field observations.

The first saved state contained one `ClassDiagram` named `REA Class Study`, two model records of type `Class` named Customer and Order, one `Association` model, and three visual records: two Class views and one Association view. The schema again separates `MODEL_ELEMENT`, `DIAGRAM`, and `DIAGRAM_ELEMENT`.

The Association's serialized definition has explicit `from` and `to` `AssociationEnd` structures. Their `type`/`EndModelElement` reference fields point to the Customer and Order class model IDs. Its separate visual definition contains `_fromShape`/`_toShape` view references, `_points`, endpoint pin fields, and presentation flags. Class view definitions contain `x`, `y`, `width`, `height`, font/fill/line settings and compartment-display flags. This directly corroborates the semantic relationship versus connector-presentation split in a UI-generated fixture. The full payload grammar and the coordinate semantics of connector-relative points remain unresolved.

The project also contains many default/configuration model records. Counting every `MODEL_ELEMENT` row as a user-created entity is incorrect; type filtering is essential. The inspection JSON records the current saved state and will change if the main investigation adds diagrams and reruns the script. This observation does not establish the behavior of all diagram families.

## Exhaustive public-constant coverage appendix

All 101 installed constants appear exactly once below. Family assignment is an analytical grouping, not the original application's menu taxonomy. Constant names are retained verbatim, with the common `DIAGRAM_TYPE_` prefix omitted for readability.

### UML structural

`CLASS_DIAGRAM`, `OBJECT_DIAGRAM`, `PACKAGE_DIAGRAM`, `COMPONENT_DIAGRAM`, `DEPLOYMENT_DIAGRAM`, `COMPOSITE_STRUCTURE_DIAGRAM`, `PROFILE_DIAGRAM`.

### UML behavioral

`ACTIVITY_DIAGRAM`, `COMMUNICATION_DIAGRAM`, `INTERACTION_DIAGRAM`, `INTERACTION_OVERVIEW_DIAGRAM`, `STATE_DIAGRAM`, `TIMING_DIAGRAM`, `USE_CASE_DIAGRAM`.

### SysML and requirements

`BLOCK_DEFINITION_DIAGRAM`, `INTERNAL_BLOCK_DIAGRAM`, `PARAMETRIC_DIAGRAM`, `REQUIREMENT_DIAGRAM`.

### Services and code-oriented models

`SOA_ML_SERVICES_ARCHITECTURE_DIAGRAM`, `SOA_ML_SERVICE_CATEGORIZATION_DIAGRAM`, `SOA_ML_SERVICE_CONTRACT_DIAGRAM`, `SOA_ML_SERVICE_INTERFACE_DIAGRAM`, `SOA_ML_SERVICE_PARTICIPANT_DIAGRAM`, `WSDL_DIAGRAM`, `EJB_DIAGRAM`.

### Database and ORM

`CHEN_NOTATION_ERDUI_MODEL`, `ER_DIAGRAM`, `ORM_DIAGRAM`.

### Business processes and cases

`BUSINESS_PROCESS_DIAGRAM`, `BUSINESS_PROCESS_REENGINEERING`, `CONVERSATION_DIAGRAM`, `CMMN_DIAGRAM`, `EPC_DIAGRAM`, `PROCESS_MAP_DIAGRAM`, `PM_PROCESS_MAP`.

### Data-flow models

`DATA_FLOW_DIAGRAM`, `GANE_SARSON_DFDUI_MODEL`, `YOURDON_AND_COAD_DFDUI_MODEL`, `YOURDON_DE_MARCO_DFDUI_MODEL`.

### Enterprise architecture

`ARCHI_MATE4_DIAGRAM`, `ARCHI_MATE_DIAGRAM`, `ARCHITECTURE_ROADMAP`, `MIGRATION_ROADMAP`, `ZACHMAN_DIAGRAM`, `EA_FORM_DIAGRAM`, `EA_PROCESS_STEP_DIAGRAM`, `OVERVIEW_DIAGRAM`.

### Rules, facts and decisions

`BUSINESS_MOTIVATION_MODEL_DIAGRAM`, `BUSINESS_RULE_EDITOR_DIAGRAM`, `DECISION_MODEL_DIAGRAM`, `DTB_DECISION_TABLE_EDITOR_DIAGRAM`, `FACT_DIAGRAM`, `FACT_TYPE_EDITOR_DIAGRAM`, `TERM_EDITOR_DIAGRAM`.

### Analysis and specification

`ANALYSIS_DIAGRAM`, `ANALYSIS_ITEM_DIAGRAM`, `COMPETITOR_ANALYSIS`, `CRC_CARD_DIAGRAM`, `MATURITY_ANALYSIS`, `ROOT_CAUSE_ANALYSIS`, `TEXTUAL_ANALYSIS`, `USE_CASE_DESCRIPTION_DIAGRAM`.

### Planning and experience

`PERT_CHART`, `BREAKDOWN_STRUCTURE`, `USER_STORY_MAP_DIAGRAM`, `CUSTOMER_JOURNEY_MAP`.

### General visual diagrams

`BRAINSTORM`, `MIND_MAP_DIAGRAM`, `FISHBONE_DIAGRAM`, `ORGANIZATION_CHART`, `FLOWCHART`, `FREEHAND_DIAGRAM`, `FREEHAND_DIAGRAM2`.

### Structured canvases

`PROCESS_CANVAS`, `QUALITY_CONTROL_CANVAS`.

### Data and tabular editors

`CHART_DIAGRAM`, `DATA_STUDIO_SHEET`, `DATA_TABLE_DIAGRAM`, `ETL_TABLE_DIAGRAM`, `GRID_DIAGRAM`, `MATRIX_DIAGRAM`, `TABULAR`.

### UI and wireflow

`USER_INTERFACE_DIAGRAM`, `WEB_DIAGRAM`, `WFI_PAD_DIAGRAM`, `WFI_PHONE_DIAGRAM`, `WF_ANDROID_PHONE_DIAGRAM`, `WF_ANDROID_TABLET_DIAGRAM`, `WF_DESKTOP_DIAGRAM`, `WF_WEB_DIAGRAM`, `WIREFLOW_DIAGRAM`.

### Documents and form editors

`DOCUMENTATION_CABINET`, `REPORT_DIAGRAM`, `FORM`, `VIEW_EDITOR_DIAGRAM`.

### C4 architecture

`C4_MODEL_COMPONENT_DIAGRAM`, `C4_MODEL_CONTAINER_DIAGRAM`, `C4_MODEL_DEPLOYMENT_DIAGRAM`, `C4_MODEL_DYNAMIC_DIAGRAM`, `C4_MODEL_SYSTEM_CONTEXT_DIAGRAM`, `C4_MODEL_SYSTEM_LANDSCAPE_DIAGRAM`.
