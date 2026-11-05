# UI-11 — Notation capability and surface contract

Status: **done** (2026-11-05). Plan: [UI-DESIGN-AND-INTEGRATION.md §7](../../../plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md). Follows [UI-RULES.md](../../../plan-rust-diagram-editor/UI-RULES.md).

## What was built

`crates/diagram-desktop/src/ui/notation.rs` is one **compile-time table** (`NOTATIONS`) describing what each diagram type offers. It has no registration, loading or dynamic dispatch, so it is **not a plugin framework**. The shell reads it:

- The **tool row** shows the active diagram's tools (`notation::tool_commands`). Delivered tools must be real catalog commands, so their labels, shortcuts and disabled reasons come from UI-06.
- The **inspector** shows the active notation's element sections (Model, Members, Appearance, Advanced) and skips any section a notation does not list.
- The **new-project form** offers only `notation::creatable()`, the delivered notations. Planned types never appear, and the old hard-coded type list was removed.
- Every diagram records its notation (`Diagram::notation`).

| Notation | Delivery | Tools | Inspector sections | Relationships | Interaction |
|---|---|---|---|---|---|
| **Class diagram** | **Delivered** | Select, Pan, Class, Association, Note, Arrange | Model, Members, Appearance, Advanced | Association, Generalization | Freeform |
| Flowchart | Planned, M4 | Process, Decision, Start or end, Input or output, Flow | Model, Appearance | Control flow | Freeform |
| Use case diagram | Planned, M4 | Actor, Use case, System boundary, Association | Model, Appearance, Advanced | Association, Include, Extend, Generalization | Freeform |
| Entity-relationship diagram | Planned, M5 | Entity, Relationship | Model, Members (columns), Appearance | One to one, One to many, Many to many | Freeform |
| Sequence diagram | Planned, M5 | Lifeline, Message, Reply, Fragment | Model, Appearance | Synchronous, asynchronous message, Reply | **Temporal** |

**Sequence requirements stay explicit** (`SEQUENCE_REQUIREMENTS`), so the freeform class canvas is never forced onto it:

1. Messages are ordered top to bottom; moving a message changes its order, not a free position.
2. Lifelines stay aligned in one row and extend down the whole diagram.
3. Activation bars follow the messages that start and end them.
4. Creation and destruction messages start or end a lifeline.
5. Combined fragments (alt, loop, opt) enclose a contiguous range of messages.

## Verification

| Check | Result |
|---|---|
| `tests/notation.rs` (8 tests) | Pass: only Class is creatable and every planned notation names M4 or M5; lookup returns each entry; delivered notations use catalog commands and planned ones only names; Class tools equal the tool-row placement; Class inspector sections, relationships and freeform interaction; Sequence alone is temporal, with its requirements kept; the shell shows the Class tools and no planned tool; the new-project form offers only creatable notations |
| `tests/welcome.rs` | The form's type list comes from the notation table |
| All workspace tests | 155 integration + 1 doc test pass; `cargo fmt`, `clippy -D warnings` and `cargo doc` clean |
| Real window (Linux) | New project "Orders" opens with the Class tools in the tool row |

## Not covered here
- Building the planned editors is M4 and M5. Each one adds its row to this table and its tools to the command catalog when its milestone starts.
