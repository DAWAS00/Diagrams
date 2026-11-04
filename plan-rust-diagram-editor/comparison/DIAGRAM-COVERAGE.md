# Diagram coverage: Visual Paradigm inventory versus our unbuilt Rust plan

Date: 2026-11-04. This compares the installed shared VP 18.1 API inventory with the proposed Rust plan in `01-PRODUCT-SCOPE.md` and `05-ROADMAP.md`. **Our application is unbuilt. There are zero implemented editors.** The v0.1 column below describes intended coverage only.

The exhaustive row-level artifact is [diagram-coverage.csv](diagram-coverage.csv): every one of the 101 installed `DIAGRAM_TYPE_*` constants appears exactly once, with family, current status, planned v0.1 coverage, later milestone, semantics, deliberate omissions, qualitative extension cost and CE evidence/uncertainty. [generate_diagram_coverage.py](generate_diagram_coverage.py) checks inventory equality and produces the CSV reproducibly.

## Exact scope difference

| Inventory status | Constants | Meaning |
|---|---:|---|
| Implemented in our application today | 0 | Planning artifacts and HTML concept are not a Rust application |
| Planned partial v0.1 coverage | 1 | `CLASS_DIAGRAM`: a limited class/member/association/generalization subset |
| Absent from v0.1 | 100 | Their notation semantics and editor workflows are deferred |
| M4 explicitly planned adapters | 2 | Flowchart and use-case diagrams |
| M5 named family milestones | 8 | Activity, state, sequence, ER, requirements, block definition, internal block and parametric |
| M6 candidates without committed scope | 90 | Remaining constants; a candidate is not a delivery promise |
| Installed inventory total | 101 | 1 + 2 + 8 + 90; not a count of enabled CE editors |

The 1/101 ratio is an **inventory breadth indicator only**, not a percentage of product parity. Even the class entry is partial and unimplemented. Constants differ greatly in scope, several overlap or represent variants, and some identify structured text/table/document editors rather than ordinary graph canvases.

M5's wording names families rather than an exhaustive standards implementation. Chen ER notation is therefore a separate M6 candidate until explicitly scoped; the M5 ER milestone does not silently promise it. Communication/timing/interaction-overview diagrams are also not silently included in the sequence milestone.

## Family-by-family difference

| Installed analytical family | Constants | Planned v0.1 | Later scope and principal missing work |
|---|---:|---|---|
| UML structural | 7 | Partial class subset only | Object slots, component ports, deployments, package semantics, composites and profiles need typed schemas and specialized tools |
| UML behavioral | 7 | None | Use case M4; activity/state/sequence M5; communication/timing/interaction overview M6 candidates |
| SysML and requirements | 4 | None | M5 family milestones; block/port/constraint/traceability schemas and validation remain to be specified |
| Services and code-oriented models | 7 | None | M6; contracts/interfaces/participants, code/schema links and transformations |
| Database and ORM | 3 | None | ER M5; Chen/ORM M6 candidates; columns/keys/indices/cardinality/mapping semantics |
| Business processes and cases | 7 | None | M6; BPMN/CMMN/EPC and related models with lane/event/subprocess and flow rules |
| Data-flow models | 4 | None | M6; stores/processes/actors, decomposition and notation variants |
| Enterprise architecture | 8 | None | M6; relation taxonomy, viewpoints, layered models, frameworks and roadmaps |
| Rules, facts and decisions | 7 | None | M6; typed rules/facts/decision tables and separately scoped evaluation behavior |
| Analysis and specification | 8 | None | M6; structured analysis records, rich-text references and specialized forms |
| Planning and experience | 4 | None | M6; stories/journey lanes, PERT networks, breakdown trees and any calculations |
| General visual diagrams | 7 | None | Flowchart M4; remaining tree/freeform/branch-aware editors M6 |
| Structured canvases | 2 | None | M6; fixed sections, cards and template-aware export |
| Data and tabular editors | 7 | None | M6; cell/grid/chart/matrix models and editing rather than just node placement |
| UI and wireflow | 9 | None | M6; screens/widgets, device frames, state and navigation/prototype behavior |
| Documents and form editors | 4 | None | M6; rich text, forms, bindings, templates and page composition |
| C4 architecture | 6 | None | M6; hierarchy/boundaries and deployment/dynamic semantics |
| **Total** | **101** | **1 partial planned entry** | **100 excluded from v0.1** |

These 17 families are our analytical grouping from [the research catalog](../../research/visual-paradigm/DIAGRAM-CATALOG.md), not claimed VP menu labels.

## What even the first class editor deliberately loses

The plan includes classes, ordered attributes/operations, associations, generalization, endpoint roles/multiplicity, model reuse, independent views, simple routes and SVG export. That captures a useful structural-modeling workflow, but does not mean complete UML class support.

The installed/observed VP surface is wider: class specification contains visibility and classification flags, documentation, receptions, stereotype/profile behavior and extensive member/property presentation. The plan does not commit to every classifier/relation type, every association-end option, full profile/tagged-value editing, every compartment mode, all routing/layout algorithms or legacy code/database mappings. It also does not claim complete UML conformance. Those losses remain even after v0.1's own acceptance walkthrough passes.

The proposed model/view identity split is a foundation we retain, but v0.1 cannot automatically reproduce VP's master/auxiliary view behavior, every containment mode, clipboard identity rules or cross-project references. Those need explicit specification and fixtures before comparison claims.

## Why extending the palette is not enough

A notation expansion generally needs six connected changes:

1. Semantic schema and stable identity/ownership rules.
2. Creation tools and admissible relationship/containment constraints.
3. Contextual property editing and consistent selection behavior.
4. Rendering, hit testing, labels and notation-specific layout/interaction.
5. Persistence/migrations plus command and undo/redo invariants.
6. Export and acceptance fixtures proving a complete real workflow.

Family cost is qualitative relative to the basic class engine, not an effort/date estimate. “Medium” means considerable reusable canvas work with new shapes/constraints; “High” adds significant typed structures and editor rules; “Very high” may need an additional editing model such as time-ordered sequence interaction, spreadsheets, rich documents, simulation or external transformation tooling. These labels identify extension pressure, not recovered VP implementation complexity.

The Rust plan uses typed enums and ordinary notation modules, not a ready-made dynamic plugin runtime. That protects v0.1 consistency but means new model kinds can require coordinated core/storage/UI/export changes and schema migrations. The architecture should not be advertised as adding any new notation solely through a JSON palette file.

## Evidence versus CE availability

The local UI study exercised basic Class, Sequence and ER workflows and saved/exported research artifacts. Advanced behavior remains unverified. The installed API inventory covers a shared product platform; it does not prove all 101 editors are enabled in Community Edition or run natively/offline.

The inspected [official edition table](https://www.visual-paradigm.com/support/edition-comparison.jsp) documents CE families including UML, SysML, ER and flowcharts. It does not mark BPMN, DFD or C4 toolsets for CE. Other inventory entries may have online dependencies, different entitlements, specialized availability or unclear correspondence to visible creation choices. CSV rows preserve these distinctions instead of turning bundled names into verified functionality.

Use [UI-STUDY.md](../../research/visual-paradigm/UI-STUDY.md), [the API catalog](../../research/visual-paradigm/DIAGRAM-CATALOG.md), and the `ce-evidence` files as the evidence boundary. No new application/configuration/install changes were made for this comparison.

## Decisions this comparison requires later

Treat full-platform parity as a separate multi-epic ambition. Deliver the proposed class workflow first, validate M4's adapter reuse, then scope each M5 family with explicit excluded semantics. Choose among the 90 M6 candidates by real user workflows; do not use the raw constant count as a development checklist or promise. VP interoperability is a separate project even where both applications use SQLite.
