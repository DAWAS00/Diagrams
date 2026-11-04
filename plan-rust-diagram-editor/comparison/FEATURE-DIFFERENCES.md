# Feature and behavior differences

Compare three different kinds of evidence: **L** = local installation/API/UI/file observation; **D** = official documentation; **P** = proposed Rust behavior. A visible menu is only UI evidence. No P entry is implemented or validated yet.

## Core product and workflow matrix

| Area | Visual Paradigm evidence | Proposed Rust v0.1 | Decision / practical difference |
|---|---|---|---|
| Product breadth | L: 101 shared API constants and many suite entrypoints; enabled CE subset differs | P: partial class workflow | **Defer** most breadth; do not advertise suite parity |
| Start a diagram | L: type search → template/blank → name/location/description | P: choose type/name and enter editor; metadata later | **Simplify** default path while keeping metadata editable |
| Templates | L: preview catalog loads asynchronously | P: blank initial workflow | **Defer** template catalog; fewer starter examples |
| Online/AI creation | L: prompts and web-category cards; no request executed | P: offline creation | **Separate/defer** service-backed workflows |
| Primary commands | L: multiple ribbon groups | P: compact project/diagram/view command surface | **Redesign** command organization; advanced suite actions absent |
| Project navigation | L: navigator tree, model explorer, thumbnail browser and recent/files views | P: Diagrams/Model sidebar, tabs and search | **Simplify** basic navigation; less rich browsing |
| Class placement | L: click tool then canvas, or drag from palette | P: labeled tool, drag and contextual add | **Keep** direct manipulation with clearer discovery |
| Naming | L: immediate inline name editing | P: inline naming with explicit commit/cancel | **Keep/refine**, not an entirely new capability |
| Class members | L: class/attribute specification dialogs and typed API members | P: ordered attributes/operations in inspector | **Redesign** UI; preserve typed ordered members |
| Detailed UML | L: flags, receptions, stereotypes/profiles and extensive properties | P: limited declared class subset | **Defer** semantics outside the defined subset |
| Associations | L: created connection; semantic AssociationEnd records separate from view routes | P: roles/multiplicities plus basic association | **Keep subset**; not every navigability/aggregation/qualifier option |
| Generalization | L: palette/API support; creation not exercised | P: inheritance relation, self/cycle checks | **Plan** and validate its exact subset independently |
| Shared identity | L: Copy to Diagram creates a second appearance referencing the same Class ID | P: Show in diagram | **Keep**, use more explicit wording |
| Model duplication | D: duplicate/copy-model distinctions; menu observed | P: Duplicate model with new IDs | **Specify/test** identity rules, not just appearance |
| Master/auxiliary views | D: containment behavior can differ by view role | P: simpler explicit semantic reparent command | **Redesign**; do not silently infer semantic ownership from dragging |
| Semantic versus visual change | L: separate model and visual fields | P: Model versus Appearance inspector sections | **Keep/make explicit** property scope |
| Node movement | L: connector follows a moved class | P: transient preview, commit one transaction | **Keep**, internal implementation is our own |
| Undo | L: class movement undo restored placement/route | P: reversible atomic commands and bounded history | **Keep/validate** compound actions and text-focus behavior |
| Resize and precise placement | L/API: geometry and handles; complex behavior not fully tested | P: resize, snapping, keyboard movement | **Keep subset**, measure coordinate/focus edge cases |
| Selection | L: selected handles/context actions; API supports members | P: single/multi-selection plus object-list access | **Keep core**; member-level targets require explicit design |
| Connection affordances | L: palette tools and contextual handles | P: visible valid-target previews and large hit areas | **Refine** discoverability; UX benefit unmeasured |
| Connector styles | L/API: route points, pins, jumps, multiple styles, labels and member endpoints | P: basic straight/orthogonal routes and anchors | **Defer advanced** curves, crossings, edge/member targets and rich labels |
| Auto-layout | L/API: several layout families and option factories | P: initial arrangement/alignment; richer layout later | **Narrow** automatic layout capability |
| Formatting | L: styles/presentation controls; stored per-view values | P: small style set, font/line/fill basics | **Keep subset**, less presentation configurability |
| Layers/grouping | L/API/menu: supported surfaces, not fully exercised | P: basic grouping in quality milestone; layers not core v0.1 contract | **Clarify/defer**, distinguish semantic ownership |
| Clipboard exchange | L: image JPG/EMF, Simple/Traditional XML and internal copy menu | P: explicit reuse/duplication first | **Defer formats**; loss for office/document workflows |
| View removal / model deletion | D/menu: distinct possible semantics; destructive workflow untested | P: Remove view default; Delete model everywhere separately | **Define/test**, not a claim about VP defaults |
| Search | L/D: browser search across project perspectives | P: sidebar name search | **Narrow** search scope; specify members/docs/roles later |
| References/navigation | L/API/D: subdiagrams, references, jump/bookmarks, referenced projects | P: within-project IDs and tabs | **Defer** cross-project reference/reuse infrastructure |
| Project file | L: native `.vpp` is SQLite in two inspected fixtures | P: independently versioned `.dgram` SQLite | **Change format**; shared database technology does not create compatibility |
| XML exchange | L: selected diagrams exported as Simple XML plus data.zip | P: no VP XML importer in v0.1 | **Defer** VP interchange and roundtrip guarantees |
| Image/document export | L: export entrypoints; D: multiple image/PDF formats | P: SVG first | **Narrow** sharing formats; consider PNG next based on user need |
| Save state | L: modified title marker and save status | P: visible Saving/Saved/error state | **Refine** feedback; no claim of stronger safety yet |
| Recovery/migrations | VP behavior not fully investigated | P: recovery snapshots, transactional migration and errors | **Plan/verify**, never assert VP lacks safeguards |
| Runtime stack | L: native Windows launchers, bundled JVM and Java platform | P: Rust/eframe application | **Replace implementation** without inferring performance benefits |
| Accessibility | Full VP conformance not established | P: keyboard/object-list/screen-reader gates | **Test**, do not claim superiority |
| Multilingual content | Not comprehensively tested in VP | P: English/Arabic/mixed text and IME gate | **Test** shaping/input; localization is a separate gap |
| Multiple platforms | D: broader VP platform support | P: Windows-first | **Narrow** initial support; portability is only an architectural aim |
| Extensions | L/D: public Java plugin APIs and automation scripts | P: internal Rust notation modules | **Defer** public SDK, plugin lifecycle and automation compatibility |
| Reports/engineering/services | D: broad suite, edition/service boundaries apply | P: later or unspecified | **Defer** as separately scoped systems |

## Diagram families: the largest functional difference

The complete 101-row mapping is in [DIAGRAM-COVERAGE.md](DIAGRAM-COVERAGE.md) and [diagram-coverage.csv](diagram-coverage.csv). Only part of Class Diagram is intended for v0.1. Flowchart and Use Case are M4; ER, activity/state, sequence and selected requirements/SysML are later family milestones. Most remaining types are M6 candidates, not promises.

This is not 1% parity. A class editor, decision table, sequence editor, report composer and wireframe tool have very different costs. Counting constants cannot measure implementation effort or completeness.

## Where the UX is deliberately different

### Creation and editing

VP's observed wizard offers useful templates and metadata before editing. Our default path postpones optional information. VP already supports inline names and contextual creation; our proposal mainly reorganizes common controls and reduces modal member editing. The measurable hypothesis is fewer interruptions for novice tasks, not that every VP workflow is inefficient.

### Identity and ownership

Use **Show in diagram** for another appearance and **Duplicate model** for another semantic object. Keep clear Model/Appearance sections. Choose explicit package reparenting rather than encoding master-view ownership behavior implicitly in a visual drag. This is a real behavior difference; document it and make migration/interchange adapters respect it later.

### Advanced properties

Hide optional fields behind expandable sections, but preserve every property we declare supported. If the first-release schema lacks profiles, tagged values, receptions or advanced association ends, label that a missing capability, not just cleaner UI. Unrepresented imported fields must not be silently discarded by a future importer.

## Architecture differences and non-differences

**Different:** Rust types and ownership, command patches, renderer-neutral scenes, four internal crates, independent storage schema, native deployment and our chosen worker/revision policy.

**Intentionally similar:** a structured project, typed semantic entities, separate diagram views, relationship endpoints and visual anchors, geometry, presentation, versioned persistence and extensible notation boundaries.

**Unknown:** VP's exact transaction/history implementation, locking/recovery guarantees, renderer caches, routing algorithms, text-shaping pipeline and performance bottlenecks. Public signatures and serialized options do not establish these algorithms. Do not claim the Rust design solves a demonstrated VP implementation defect without additional evidence.

## Decisions prompted by the comparison

1. Keep basic semantic fidelity ahead of visual polish: rename, typed members, relationships, reuse, undo and save/reopen must work together.
2. Maintain a strict v0.1 list; do not show unsupported notation or suite buttons as though features exist.
3. Consider PNG export early after SVG because office/report sharing may matter more than another rare notation. This is a recommendation, not a scope change.
4. Define search indexing, package ownership and history/copy semantics before implementation ambiguity creates accidental behavior.
5. Keep text shaping, DPI and keyboard accessibility as the GUI selection gate. A native Rust UI is not automatically more usable.
6. Measure the same representative tasks in both applications before claiming faster, lighter, easier or safer.
7. Separate local CE proof from commercial-suite documentation in every later feature proposal.

See [VALIDATION-GAPS.md](VALIDATION-GAPS.md) for the remaining evidence work and [BROADER-PRODUCT.md](BROADER-PRODUCT.md) for sourced suite-level details.
