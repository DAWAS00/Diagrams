# 01 — Product scope

## Product objective

Help a student or developer express a design quickly, keep its structured data consistent, and confidently save or share it. The first release is a small useful modeling application, not a broad collection of partially implemented diagram types.

## Basic release: v0.1

| Capability | Concrete behavior |
|---|---|
| Project lifecycle | New, open, save, Save As, close, recent projects and unsaved-change handling |
| Navigation | Separate Diagrams and Model tabs in the project sidebar; search; open-diagram tabs |
| Class notation | Classes, ordered attributes/operations, association and generalization; semantic endpoint roles/multiplicity |
| Canvas | Select, multiselect, place, drag, resize, pan, cursor-centered zoom, fit, labels and basic grid snapping |
| Connections | Port/edge hit targets, attached endpoints, straight and simple orthogonal routes, editable relationship properties |
| Editing | Inline naming, contextual properties, Escape cancel, keyboard movement, undo/redo, duplicate model versus create another view |
| Consistency | Same class can appear in two diagrams; rename propagates; positions/styles remain independent |
| Appearance | Readable theme, original icons, a small style set, consistent focus/selection and DPI behavior |
| Persistence | Versioned native file, transactional saves, save errors that preserve previous data, recovery snapshots |
| Export | SVG of diagram content without selection handles; validated XML; declared font behavior |

Association ends store role/multiplicity separately from visual anchors. Generalization points child → parent, rejects self-links and inheritance cycles. This is a deliberately limited UML subset, with explicit validation instead of a claim of full UML conformance.

Do not make copy/paste a prerequisite for basic creation. The first identity-aware reuse action is **Show in diagram**; **Duplicate model** creates new semantic IDs. Rich clipboard interoperability follows after those semantics are stable.

## First acceptance walkthrough

1. Launch the app and choose **New project → Class diagram** on one screen.
2. Add Customer and Order; rename them inline.
3. Add `id: String` to Customer and `total: Decimal` to Order.
4. Connect Customer to Order with an association and set multiplicity `1` / `0..*`.
5. Add PreferredCustomer with a generalization to Customer.
6. Create a second class diagram and use **Show in diagram** for Customer.
7. Rename Customer to Client. Both views display Client.
8. Move/style one Client view; the other keeps its position/style.
9. Undo and redo the compound edits; no dangling references appear.
10. Save, close and reopen. IDs, member ordering, relations, anchors and geometry match.
11. Export SVG. Labels, arrowheads and compartments are readable, and editor controls are absent.

This same scenario becomes an automated engine/storage fixture and a native UI smoke test. A second scenario covers cancellation and failed saving.

## Explicitly later

Flowcharts and use cases follow the basic release. ER needs columns, database types and key rules; sequence needs temporal ordering and activation layout. Those are separate editor milestones. BPMN, SysML/requirements beyond the first adapters, C4/enterprise methods, document composition, collaboration, AI, code generation, database reverse engineering and VP interchange are later epics.

Online/web catalogs observed in VP do not automatically become requirements. There is no account or service dependency for our basic application.

## Product usability goals

These are targets to measure, not claims already achieved:

- First-time user completes two connected named classes in under two minutes after launch.
- Common create/rename/connect/save steps require no modal property dialog.
- A user can determine what changed, whether it is saved and where it is stored.
- Advanced properties are available in the inspector without hiding the canvas.
- Invalid actions explain the reason next to the relevant object and leave the document unchanged.
- A keyboard user can reach the model tree, canvas object list and inspector, rename an item and save.

During usability evaluation, ask at least three representative users to complete the same tasks and record time, hesitation, mistakes and recovery. If representative users are unavailable, mark the goal unvalidated rather than claiming the UI is easier.
