# ADR-003 — UI decisions D1 to D6 for the class workflow

Status: accepted. Date: 2026-11-06.

## Context

The [ideas plan](../../app/docs/next-steps/IDEAS-PLAN.md) section 5 lists six decisions that later phases of the [execution plan](../../app/docs/next-steps/EXECUTION-PLAN.md) need before they write code: how a generalization is drawn, what happens when a typed class name already exists, which relations v0.1 ships, where a new member appears, which persistence unit to build, and where task status lives. The user accepted every recommendation in that table on 2026-11-06.

Facts in the code that these decisions rest on, checked at commit `96b1fae`:

- `diagram-core` treats a generalization's `source` as the child and its `target` as the parent (`Relation` in `crates/diagram-core/src/model.rs`, `Command::Connect` in `command.rs`).
- `RelationKind` (`crates/diagram-core/src/notation.rs`) has two variants, `Association` and `Generalization`, and is not `#[non_exhaustive]`.
- Core has no rule about duplicate class names. Members belong to the shared model element, not to a view.
- ADR-002 proposes one SQLite file per project; schema v1 (R1-05) is not started.
- Several documents carried hand-copied task counts that no longer matched `roadmap.json`.

## Alternatives

| ID | Question | Options considered | Chosen |
|---|---|---|---|
| D1 | Generalization direction in the UI | drag child to parent; drag parent to child | child to parent |
| D2 | Typing a class name that already exists | allow silently; block; offer a choice | offer a choice in words |
| D3 | Relation kinds in v0.1 | association and generalization; also dependency | association and generalization |
| D4 | Adding a member to a class shown in several diagrams | show it everywhere; ask every time | show it everywhere, no prompt |
| D5 | Persistence unit | one SQLite file per project (ADR-002); other formats in ADR-002 | keep ADR-002 |
| D6 | Status text in documents | keep hand-written summaries; generate them from `roadmap.json` | generate them |

## Decision

**D1 — Generalization is drawn from child to parent.** The user starts the connector on the child and ends it on the parent, matching core's `source` = child, `target` = parent, so the UI never swaps ends. The inspector names both ends in words: "Parent: X, child: Y". A cycle or a duplicate generalization is refused with a written error (Phase 3, `tests/inspector.rs`).

**D2 — An existing name is reported in words, and the user chooses.** When the typed name of a new class matches an existing class, the app says so and offers "Show the existing class here" or "Keep a new class". It never switches the new view to the existing model element without asking, and keeping a new class stays allowed. This is a UI rule; core keeps accepting duplicate names (Phase 2, `ui/dialogs.rs`, `tests/dialogs.rs`).

**D3 — v0.1 ships association and generalization only.** No dependency, realization, aggregation or composition kinds are added. `RelationKind` stays a plain enum that new variants can join; code that matches on it must keep handling every variant explicitly. Revisit after the class vertical slice (R2-08).

**D4 — A new member appears in every diagram that shows its class, with no prompt.** The model is shared; each view shows the element's members. Undo removes the member everywhere at once.

**D5 — Keep ADR-002: one SQLite file per project.** Schema v1 and its DTOs (R1-05) are defined before any UI save code (Phase 5 starts with the schema). ADR-002 keeps its own status and is marked accepted when the R1-05/R1-06 evidence lands; this ADR confirms its direction.

**D6 — `roadmap.json` is the only status source.** Task status is edited only there. `build_rust_roadmap.py` regenerates every status summary in the documents, between `<!-- roadmap:NAME:begin -->` and `<!-- roadmap:NAME:end -->` markers, alongside `ROADMAP.html` and `rust-backlog.csv`. `python build_rust_roadmap.py --check` and the Python tests fail when a committed summary differs from `roadmap.json`. Hand-written counts outside the markers are removed; dated reports and logs keep their historical numbers.

## Consequences

- Phase 3 builds the generalization tool, labels and errors around D1; no direction setting is needed.
- Phase 2 adds a duplicate-name dialog (D2) with keyboard access and focus return under the UI rules. Core stays free of naming policy.
- Adding a relation kind later (D3) touches core, the notation table and the inspector, and needs its own decision; the extension point is the enum, not a plugin mechanism.
- D4 needs no prompt dialog. Tests check that a member added in one diagram shows in another.
- D5 orders Phase 5: schema and DTOs first, then save and load, then the worker and the UI.
- D6 removes the drift that made several documents disagree. A status change is one edit to `roadmap.json` plus one generator run; a forgotten run fails `--check` and the tests.

## Revisit trigger

- D1 or D2: hands-on tests show users drawing generalizations the other way or confused by the name choice.
- D3: after R2-08, or when a v0.1 user workflow cannot be modeled without another relation kind.
- D4: views ever need to hide members per diagram.
- D5: the triggers in ADR-002.
- D6: another tool becomes the task tracker; then the generator reads from it instead.
