# Validation gaps and evidence ledger

## Evidence ledger

| Evidence | Confirmed boundary |
|---|---|
| `research/visual-paradigm/ce-evidence/target.json` | CE Start Menu target identity |
| `summary.json`, `archives.json`, public API listings | Installation composition and exposed representation, not executed internal algorithms |
| `ui/` captures and `UI-STUDY.md` | Three native editor workflows and broader UI entrypoints |
| `ui-project-inspection.json` | Historical saved-fixture identity/SQLite observations at acquisition time |
| `xml-export-inspection.json` | Three exported diagram types, nine original visual references and semantic/presentation distinctions |
| `rea-sqlite-evidence.json` | REA's bounded read-only schema/DIAGRAM inspection of the earlier saved fixture |
| `comparison-project-inspection.json` | Four-diagram copied-view fixture, shared model ID, SQLite quick check |
| Official links in BROADER-PRODUCT.md | Documented suite behavior, with edition/service/version caveats |
| `plan-rust-diagram-editor/*.md` | Proposed Rust behavior only; no working application |

Paths in the table are under `D:\REA visual\` or the specified `ce-evidence` directory. Preserve evidence with its digest and acquisition context. The original disposable project changed across resumed sessions; a prior digest remains historical evidence, not an assertion of its current contents.

## Answered at the product-planning level

- Product scope and the distinction between CE, shared platform and broader paid/online suite.
- All 101 installed diagram constants classified once into 17 analytical families.
- Proposed v0.1 versus later-family scope and the main losses users would experience.
- Visible desktop workspace, creation wizard, property dialogs and common editing flow.
- Semantic model versus view storage, relationship endpoints versus visual routing, and copied-view shared identity.
- Independent Rust implementation boundaries and why technology choice alone proves no product advantage.

## Partially answered

- The basic class/sequence/ER editor behavior; advanced relations, layout, clipboard and specialized editing remain incomplete.
- Save/export behavior; successful native saves and selected XML export do not prove failure recovery or import roundtrips.
- Public extensibility; APIs and docs exist, but plugins were not installed/executed here.
- Edition availability; current vendor tables and local menus differ in what they establish.

## Unresolved tests with architectural impact

| Test | Why it matters | Status |
|---|---|---|
| Rename shared model from either view and verify both | Update propagation and inspector scope | Shared IDs confirmed; rename propagation not executed in this pass |
| Move/style auxiliary view, compare other view | Independent appearance versus shared semantic state | Separate views/geometry represented; independence needs a controlled paired edit |
| Master-view reparent versus auxiliary drag | Package ownership policy differs in proposed app | Documented VP distinction, local experiment pending |
| Copy Model/Paste Model across projects | Duplication and reference rules | Menu/docs only |
| View removal versus semantic deletion and undo | Dependency cleanup and user expectation | Destructive workflow not executed |
| Class members, generalization and cycle behavior | Exact supported UML subset | Partial API/dialog evidence; complete workflow pending |
| Resize/anchors/ports/member endpoints and routing | Connector engine complexity | Partial API and movement proof |
| Sequence insertion/reordering/fragments/returns | Temporal/activation constraints | Basic message/activation/numbering only |
| ER columns/PK/FK and many-to-many generation | Semantic schema transformations | Dialog/API/docs; full mutation workflow pending |
| Activity/state/use-case/SysML and other families | Notation-specific semantics | Inventory/docs; editors not individually exercised |
| VP XML reimport and `.vpp` migrations | Compatibility and unsupported-property preservation | Export only; complete inner payload grammar not recovered |
| Crash/disk-full/failed-save recovery | Persistence guarantees | Neither comparative VP test nor Rust implementation exists |
| Text/IME/RTL/DPI/screen reader | GUI feasibility and usability | Rust gate defined; paired tests not performed |
| Large-project responsiveness and memory | Optimization requirements | No comparative benchmark |
| Team repository merge and offline synchronization | Collaboration architecture | Documented, no service/account testing |

## Completion language

Use **product-wide gap analysis completed** for this deliverable. Do not say full reverse engineering, native decompilation completed, all 101 editors tested, full CE parity, or Rust application verified.

No Ghidra-backed analysis session was opened; therefore there is no native analysis process to close from this pass. REA's connected server reports Ghidra missing/unconfigured. Fixing it belongs to the separately reviewed setup scope, not this feature comparison.

## Recommended next evidence pass

After building the M0 Rust prototype, compare actual text/DPI/focus behavior and the class vertical-slice tasks. Before adding a family, capture one complete native VP example with its properties, relationships, undo and save/export evidence, then turn it into independent acceptance fixtures. This produces useful requirements without attempting to decompile the launcher as a substitute for product behavior.
