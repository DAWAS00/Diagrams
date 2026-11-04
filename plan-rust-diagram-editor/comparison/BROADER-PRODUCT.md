# Broader product comparison: Visual Paradigm and the Rust editor plan

Date: 2026-11-04. This comparison extends beyond diagram creation. It compares a shipped product family with an **unbuilt plan**, not two working applications.

## Evidence labels and scope

- **Local**: previously recorded installation/API or UI evidence under `research/visual-paradigm`. Seeing a menu is not evidence that its operation succeeds in CE.
- **Documented**: official vendor documentation, not locally exercised functionality. Some guides describe older interfaces or paid/service-dependent capabilities.
- **Planned**: proposed Rust editor behavior in this planning folder. No runtime result, usability improvement, performance advantage, or data-safety guarantee is established yet.
- **Gap**: the capability is absent from v0.1, only broadly deferred, or unspecified in the plan. It is not automatically a requirement to implement it.

The current vendor comparison lists code engineering, database engineering, Doc. Composer and project-web publishing in Standard/Professional/Enterprise, with CE blank. Some collaboration rows do include CE but require a collaboration service; other revision/branching tools have narrower availability. Do not infer entitlements from shared JARs or ribbon visibility. [Current edition comparison](https://www.visual-paradigm.com/support/edition-comparison.jsp).

The Rust plan deliberately starts offline, single-user and Windows-first. Its v0.1 is a limited class-modeling workflow with native storage and SVG export. M4–M5 add notation families; M6 only considers broader product epics. None of these milestones is implemented.

## Comparison matrix

| Product area | VP evidence and boundary | Rust plan status | Gap / decision |
|---|---|---|---|
| Reporting | Documented template-driven generation from model data; paid-edition scope | Later M6 reports; no report engine design | Separate report/query/template domain, not an SVG export option |
| Web publishing | Documented interactive model/diagram website generation | No publisher in v0.1 | Need navigation links, document assembly, asset/reference policy and output validation |
| Code engineering | Documented generation, reversal and synchronization | Explicitly later | Define one language/operation before estimating; generation is not round-trip |
| Database engineering | Documented ER-to-DDL/live database generation and schema reversal | ER editor M5; engineering later | Diagramming, SQL generation and applying database changes are separate deliverables |
| Plugins/Open API | Installed public Java API and documented plugin integration | Internal notation modules; no dynamic plugin platform | No user plugin ABI, compatibility contract, permissions or packaging specified |
| Navigation | Local Project Browser and ribbon; documented multi-perspective browsing | Planned Diagrams/Model sidebar, search and tabs | Thumbnails, supplementary files, recent-team views, referenced-project browsing not in basic scope |
| Collaboration | Documented repository commit/update/conflict workflow | Offline single-user; later epic | Identity/version/merge/server/account design entirely open |
| Online services | Documented desktop/web product connections | Explicitly no basic account/service dependency | Product boundary differs intentionally; no online-feature parity |
| Localization | Documented switchable UI languages and font/date settings | English/Arabic text and IME tests planned | Editing multilingual content is not translated UI; localization architecture unspecified |
| Accessibility | Shortcut/options documentation exists; conformance not established here | Explicit keyboard/object-list/screen-reader tests at M0/M3 | Potential design priority only; no evidence either application is more accessible |
| Integration/exchange | Local scripts/IDE adapters/API; some capabilities edition-specific | Native `.dgram`; SVG; later VP exchange | File compatibility, IDE integration and external automation absent from v0.1 |

## Reporting and publishing are separate products within the suite

**Documented:** Doc. Composer can combine model data and templates into documents, including diagrams and model details; the FAQ identifies HTML, PDF and Word output. This is structured reporting from the repository, not merely saving a picture. [Official FAQ, documentation section](https://www.visual-paradigm.com/support/faq.jsp).

The Build-from-Scratch workflow chooses a model/diagram, drags an applicable element template into a document, and exports the result. Custom templates change the selected fields and layout. [Doc. Composer walkthrough](https://knowhow.visual-paradigm.com/reporting/using-doc-composer-build-from-scratch-mode/).

**Documented:** Project Publisher writes a browsable web representation of diagrams and model details. Its options include output location, reference-file copying and browser viewing. This is generated web content; it should not be confused with uploading a project to a cloud collaboration server. [Project Publisher](https://circle.visual-paradigm.com/docs/document-production/project-publisher/publishing-project-using-project-publisher/).

**Planned/gap:** Our shared scene and SVG export could provide report images, but they do not supply document queries, templating, pagination, cross-references, linked HTML navigation, refresh semantics or asset handling. M6 names reports without specifying these systems. If reporting is desired, first propose a simple class/member/relationship HTML report generated from an immutable project snapshot; defer custom template scripting and Word integration until separately scoped. This is a recommendation, not implemented behavior.

## Code and database engineering

**Documented:** VP distinguishes forward generation, reverse engineering from source, and round-trip synchronization of artifacts that changed independently. Supporting an importer plus an exporter alone is not evidence of semantic synchronization. [Code-engineering guide](https://www.visual-paradigm.com/guide/development/practical-guide-for-code-engineering/).

The REST workflow generates provider sample code and optional API documentation from modeled resources and configurable templates. That is another specialized model/export path, not a property every class diagram automatically provides. [REST generation](https://circle.visual-paradigm.com/docs/code-engineering/rest-api-design-and-generation/generating-rest-api-as-service-provider/).

**Documented:** Database generation can write DDL or execute changes against a configured database. These have different effects and operational requirements. [Database generation](https://circle.visual-paradigm.com/docs/database-design-engineering/database-management-guide/how-to-generate-database-from-erd/). Database reversal selects source/schema/tables, previews extracted entities and can create models with or without arranging diagrams. [Database reversal](https://circle.visual-paradigm.com/docs/database-design-engineering/database-management-guide/how-to-reverse-engineer-erd-from-database/).

**Local boundary:** The installation contains code/DB/ORM support libraries and scripts, and the CE UI exposes relevant menus. Our prior work did not successfully run code generation, database connection/reversal, or live database updates. Their existence in the commercial suite must not be attributed as tested CE behavior.

**Planned/gap:** v0.1 has class properties and free-form type text, not language parsers or compilable-code guarantees. ER's M5 specification has typed columns/keys/cardinality but no DBMS adapters, catalog discovery, SQL dialect handling, migration/diff model or connection-secret storage. These need separate designs. The safest first engineering milestone would be a file-only output with preview and reproducible fixtures, followed later by import/synchronization and finally live-system actions, if requested.

## Extensibility and automation

**Local:** `openapi.jar` supplies public project/model/diagram/view interfaces. The recorded API signatures show application managers and separate semantic/visual object types. Local installation also has batch entrypoints and IDE adapters. These observations establish API artifacts; they do not verify plugin lifecycle or all operations in CE.

**Documented:** VP plugins can add menu/context actions and programmatically work with models and diagrams through the public Java API. The vendor describes plugin controllers, context and model factories. [Implementing a plugin](https://circle.visual-paradigm.com/docs/open-api/implementing-plug-in/implementing-a-visual-paradigm-plug-in/).

**Planned/gap:** `02-ARCHITECTURE.md` explicitly uses ordinary Rust notation modules initially. Its four crates are internal boundaries, not a public SDK. No stable plugin ABI, capability isolation, extension versioning, installed-plugin registry or automation command interface exists. Keep those claims absent from v0.1. A future typed batch import/export CLI may be simpler to support than arbitrary in-process plugins; that is an option to evaluate, not part of the agreed plan.

## Project organization and navigation

**Local:** The UI study records Project Browser tabs, diagram tabs, project navigation and the ribbon's search/jump/bookmark surfaces. Only the recorded actions should be treated as exercised. See [UI-STUDY.md](../../research/visual-paradigm/UI-STUDY.md).

**Documented:** Project Browser offers diagram thumbnails, model structure, supplemental files, recent diagrams and search across diagrams/models/views. It also describes browsing referenced projects. [Project Browser](https://circle.visual-paradigm.com/docs/visual-paradigm-getting-started/user-interface/project-browser/).

**Planned/gap:** The Rust sidebar deliberately reduces navigation to Diagrams/Model tabs, search and open tabs. That covers basic local work but not a project-file library, cross-project reference resolution, thumbnail browsing, bookmarks, named logical views or team history. Initial search needs an explicit scope: model name, diagram name and preferably member name; whether it indexes descriptions and relationship roles is not yet specified. These omissions simplify v0.1 while reducing suite breadth. They do not prove easier use.

## Collaboration and service dependence

**Documented:** Teamwork Client connects to VP Online or a self-hosted Teamwork Server and manages collaborative projects and revision operations. [Teamwork Client](https://circle.visual-paradigm.com/docs/team-collaboration/introduction/working-with-the-teamwork-client-window/). Commit uploads local modifications, previews changes and resolves conflicting property changes before publication; partial commits are documented. [Commit and conflicts](https://circle.visual-paradigm.com/docs/team-collaboration/team-collaboration/committing-changes-to-repository/).

**Documented product distinction:** VP Desktop and VP Online are different clients, and some desktop features depend on the online platform. The vendor's older dependency page has an edition table that differs from the current comparison; use it for the architectural dependency distinction rather than assuming today's CE entitlements. [Desktop/web dependence](https://www.visual-paradigm.com/features/vp-online/desktop-accessibility/).

**Planned/gap:** A local `.dgram` file, recovery snapshots and undo are not collaboration or version control. Stable UUIDs and atomic commands are helpful prerequisites but do not solve multi-user concurrency, merge semantics, authentication, repository storage, offline synchronization, access control or audit trails. No collaboration protocol/server/merge implementation is planned for v0.1. Do not describe this as real-time collaborative parity. If collaboration becomes a requirement, specify file handoff, repository commits or simultaneous editing as distinct possible products first.

## Language, text and accessibility

**Documented:** VP supports selecting installed UI-language resources and restarting to apply them. [Language switching](https://circle.visual-paradigm.com/docs/visual-paradigm-getting-started/configuration/how-to-switch-visual-paradigms-language/). Appearance options document user-language, font and date/time-format settings. [Appearance options](https://circle.visual-paradigm.com/docs/application-options/general/appearance/).

The vendor page titled “Accessibility of the web-based features” concerns availability through desktop/web clients, **not** disability-accessibility certification. A bounded search of official VP pages for screen-reader/accessibility statements did not establish a desktop conformance claim. That is an evidence gap, not proof of absent support. We have not tested VP screen-reader behavior or its handling of all scripts.

**Planned/gap:** Rust M0/M3 explicitly gate English/Arabic/mixed-direction text, IME, focus, keyboard navigation, UI scaling and a canvas-object-list alternative. Those are validation tasks, not passed results. The plan does not yet specify translated menus, localized formatting, RTL workspace layout, translation files or a locale-switch mechanism. Likewise, standard egui widget accessibility does not automatically describe custom canvas objects. Do not promise accessibility superiority until both relevant workflows are tested with assistive technology.

## Broader-product decisions before expanding scope

1. Keep the v0.1 acceptance contract focused on the class-modeling vertical slice and reliable local files. Adding a reporting or cloud menu without its data workflow is not progress toward parity.
2. Maintain three comparison columns: installed CE behavior, documented commercial/online capability, and unbuilt Rust proposal. Never combine them into one VP feature count.
3. Treat each broader epic as a separate product proposal with input/output schema, UI, dependencies, edition boundary, error cases and fixtures.
4. For suite-wide ambition, add explicit roadmap decisions for reporting, file-only generation, import/reversal, plugin/CLI automation, cross-project references, collaboration, localization and assistive-technology support. Currently most are only deferred labels or unspecified.
5. Measure usability and performance on the same tasks before claiming our compact workspace is faster, easier, lighter or more reliable. No comparative benchmark exists.

## Inputs reviewed

Local plan: `README.md`, `01-PRODUCT-SCOPE.md`, `02-ARCHITECTURE.md`, `03-GUI-AND-WORKFLOWS.md`, `05-ROADMAP.md`, `06-QUALITY.md`. Local research: `UI-STUDY.md`, initial `README.md`, installed API evidence and the documentation workflow reference. Official pages above were read on 2026-11-04. No application, configuration, installation, source workspace or project file was changed in this comparison pass.
