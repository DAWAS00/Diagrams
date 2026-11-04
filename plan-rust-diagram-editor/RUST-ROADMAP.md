# Rust implementation roadmap

Plan date: 2026-11-04. Status updated 2026-11-05: R0-01 is complete; **no application milestone is complete**. The [four-crate workspace](../app/README.md) builds and its console bootstrap runs. [Verification evidence](../app/docs/R0-01-VERIFICATION.md) records the limits. This document preserves M0–M6 and the existing product scope.

## Product path

```mermaid
flowchart LR
    M0[M0 Native prototype gate] --> M1[M1 Model commands and storage]
    M1 --> M2[M2 Complete class workflow]
    M2 --> M3[M3 Reliability usability and release]
    M3 --> V1[v0.1 Basic class editor]
    V1 --> M4[M4 Flowchart and use case]
    M4 --> M5[M5 Specialized editors]
    M5 --> M6[M6 Separately scoped suite capabilities]
```

The first runnable prototype arrives in **M0**. The first useful class-modeling workflow arrives in **M2**. The first release requires **M3**, including failures, recovery, export, native usability and distribution checks. A window that opens is not a release.

Use [ROADMAP.html](ROADMAP.html) to browse stages and task dependencies, [roadmap.json](roadmap.json) as the editable tracking source, and [rust-backlog.csv](rust-backlog.csv) for spreadsheet/task tracking. R0-01 is `done` with evidence; other tasks remain `not_started`. No remote issues are created by this tracker.

## Implementation stack

| Layer | Choice / constraint |
|---|---|
| Language/toolchain | Rust, Windows x64 MSVC first; local Rust/Cargo 1.99.1 rechecked |
| Desktop GUI | Proposed eframe 0.37.0 and aligned egui; M0 must validate graphics, text, DPI and accessibility |
| Domain | Rust enums, typed ID newtypes, validated constructors and owned collections |
| Persistence | rusqlite + bundled SQLite, independent versioned `.dgram` schema |
| Serialization | Serde DTOs at storage/export boundaries, not an untyped live model |
| Errors | thiserror in libraries; optional anyhow only at executable startup/orchestration |
| Concurrency | UI thread owns mutable document; bounded standard channels to worker-owned I/O |
| Checks | rustfmt, Clippy, unit/integration/doc tests and native release smoke tests |

R0-01 pins Rust 1.99.1, rustfmt and Clippy and locks the initial thiserror dependency set. Resolve GUI and SQLite dependencies as their M0 tasks begin. The console build does not replace compiling the GUI or checking the runtime adapter. See [04-TOOLING.md](04-TOOLING.md) for paths and proposed dependencies.

## Rust workspace and delivery boundaries

Source location: `D:\REA visual\app`. The four crates now exist; the module tree below is the delivery target, not a claim that every module is implemented. Keep the plan and VP evidence outside application source.

```text
app/
  Cargo.toml                        virtual workspace, dependencies and inherited lints
  Cargo.lock                        committed application resolution
  rust-toolchain.toml                exact tested compiler, rustfmt and clippy
  crates/
    diagram-core/
      src/{lib.rs,ids.rs,model.rs,views.rs,command.rs,history.rs,validation.rs,error.rs}
      src/notation/class.rs
      tests/{model_reuse.rs,command_sequences.rs}
    diagram-canvas/
      src/{lib.rs,geometry.rs,viewport.rs,hit_test.rs,interaction.rs,scene.rs,svg.rs}
      tests/{coordinate_roundtrip.rs,interaction_cancel.rs,svg_export.rs}
    diagram-storage/
      src/{lib.rs,dto.rs,schema.rs,migrations.rs,load.rs,save.rs,recovery.rs,error.rs}
      tests/{roundtrip.rs,failed_save.rs,migration.rs}
    diagram-desktop/
      src/{main.rs,lib.rs,app.rs,editor_state.rs,worker.rs,diagnostics.rs}
      src/ui/{toolbar.rs,navigator.rs,canvas.rs,inspector.rs,dialogs.rs}
      tests/{basic_workflow.rs,save_acknowledgment.rs}
  fixtures/                         reusable tiny native projects / expected scenes
  assets/                           original icons and approved font resources
  docs/                             run instructions, native checks and release evidence
```

These are starting module responsibilities, not a requirement to create dozens of empty files. Introduce modules when they contain a coherent unit. Integration test files live inside workspace **member crates**; a `tests/` directory at a virtual workspace root is not automatically an executable integration test target.

Dependency rule: desktop → core/canvas/storage; canvas → core; storage → core. Core cannot depend on egui, SQL, OS dialogs or the worker. Rendering and persistence never own a second mutable document.

## M0 — Native Rust prototype and engineering baseline

**Deliver:** a runnable native shell demonstrating the main risks before the product depends on it.

1. Create the workspace and minimal member crates, lock dependencies and pin the tested compiler. Wire inherited lints correctly in each member.
2. Run the eframe shell with toolbar, sidebar, inspector and a simple selectable class view. Use real text widgets for naming.
3. Compile bundled SQLite with existing MSVC and verify a temporary write/read; no project schema commitment yet.
4. Check English, Arabic/mixed text, IME/paste, focus/keyboard, screen-reader access, 100/150/200% scaling and graphics startup.
5. Record the result in ADR-001. Resolve essential failures or reconsider the GUI before building more UI around it.

**Rust emphasis:** ordinary owned state; no global singleton document; no application-wide locks; no custom unsafe rendering. Keep public library surface small and documented from the first commit. Do not introduce Tokio, ECS or dynamic plugins for this prototype.

**Exit:** debug/release startup works on this computer; text/focus/DPI/accessibility limitations are measured and accepted or fixed; a headless library test and docs check run. Do not mark M0 complete on screenshots alone.

## M1 — Domain, commands, history and safe storage

**Deliver:** a headless modeling engine that can save and reopen its own data.

- Typed ElementId/RelationId/DiagramId/ViewId/MemberId, classes and ordered members, view geometry and endpoint models.
- Constructors/checks for finite coordinates, valid dimensions, ownership and relationship admissibility.
- Owned `Command` enum → validated reversible `ChangeSet`; reject edits without partial mutation.
- Delta-based undo/redo and history bounds; separate semantic/view changes from selection or viewport movement.
- Storage DTO validation, SQLite schema v1, migrations, safe Save As and recovery snapshot foundations.
- Immutable snapshot/request/result types with document-session and revision identity.

**Rust emphasis:** `&Document` for reading, one scoped `&mut Document` for command application; explicit owned input for queued jobs. Keep before/after values only for changed records. Prefer enums for known variants and typed `Result` errors; no `Box<dyn Command>` framework is necessary.

**Exit:** reference fixture roundtrips, inverse-patch sequences and invalid-input tests pass; injected save failures preserve previous committed data. A newer unsupported schema is refused without rewriting. An old save result cannot mark a newer/different document saved.

## M2 — The first complete class editor

**Deliver:** the full acceptance scenario through the native UI, not only engine tests.

- Canvas world/screen transforms, hit regions, class compartments, cached measurement and scene generation.
- Explicit interaction enum: idle/select, pan, marquee, move/resize/connect previews and text editing.
- Creation and member editing, roles/multiplicity, association and generalization, anchors and simple routes.
- Model/Diagrams navigation, tabs, selection synchronization, Show in diagram and Duplicate model semantics.
- Escape/capture loss cancels; releasing a drag makes one command; typing edits its own field rather than triggering canvas shortcuts.

**Rust emphasis:** borrow the document to draw; gather input intents; apply commands after borrow scopes end. GUI widgets edit draft buffers, not authoritative records during every render pass. Scene output is derived and can be rebuilt; it is not the canonical project.

**Exit:** Customer/Order/PreferredCustomer, members, associations, inheritance, a second view, shared rename, independent geometry and undo/redo all work. Save/reopen preserves IDs and meaning. Invalid inheritance/self-links do not mutate state.

## M3 — Reliable and usable v0.1

**Deliver:** a small deployable application with declared limitations.

- SVG export from the same scene, escaping and tested fonts/bounds.
- Multi-selection, alignment/basic grouping/styles, bounded undo memory and UI feedback.
- Save/Save As/close/recovery flows, queue limits, error handling and stale-result rejection.
- Keyboard/object-list access, text/DPI tests, native dialog cancellation, release profiling and representative usability tasks.
- Windows distribution smoke test without Rust/MSVC installed; include notices, exact versions, fixtures and known issues.

**Rust emphasis:** preserve error causes internally and present concise recoverable UI messages. Profile release builds; optimize measured hotspots rather than adding clever pointer structures. Normalize fixture IDs/timestamps only where necessary; keep assertions about semantic correctness explicit.

**Exit:** the complete v0.1 checklist in [06-QUALITY.md](06-QUALITY.md) passes. No known data-loss issue remains. Performance/usability results are reported as measurements, not claims based on Rust's reputation.

## M4 — Prove a second and third notation

Implement Flowchart and Use Case adapters against the existing model/scene/command/storage boundaries. Add typed variants, rules, inspectors, creation tools and fixtures. Keep ordinary module/static dispatch while the supported set is known.

**Exit:** these notations work without duplicating save/load, history, navigation or pointer handling. Review the actual common behavior before extracting a reusable notation abstraction; do not design a runtime plugin ABI yet.

## M5 — Specialized editor milestones

Deliver separately: ER; activity/state; requirements/SysML; sequence. The order can respond to actual user needs after v0.1, but sequence must remain its own temporal/activation milestone. Each needs typed schemas, notation-specific interaction/layout, properties, inverse edits, persistence and meaningful native fixtures.

**Rust emphasis:** keep runtime gestures as enums; use small traits or generics only where two real implementations share a contract. Use dynamic dispatch when a genuinely heterogeneous adapter registry needs it, not in every hot primitive operation. Revisit serialization versions deliberately when adding variants.

**Exit per family:** one complete domain workflow and its failure/undo/roundtrip/export tests, plus declared standards subset. A palette button is not a delivered editor.

## M6 — Product proposals, then scoped implementation

Reporting, PNG/PDF, VP XML/native interoperability, advanced layout, other catalog families, code/database tooling, automation/plugins and collaboration are candidates. First write an input/output contract, workflow, effect boundary and fixtures for each accepted epic. M6 does not commit to reproducing all VP functionality.

**Rust emphasis:** do not promise a stable Rust dynamic-library ABI for plugins; assess a versioned message/CLI/Wasm boundary if extension requirements arise. Collaboration needs its own identity/concurrency/storage design; `Arc` or a local SQLite file does not provide it.

## First executable workflow and release gates

**R0-01 is complete.** Next are **R0-02/R0-03**, then the native gate. Task prerequisites and current status are explicit in [roadmap.json](roadmap.json). A task is complete only when its implementation and verification evidence exist; 38 tasks remain not started.

The project-level definition of done combines:

1. Correct externally visible behavior and meaningful failure tests.
2. Documented ownership/error/state invariants and no forbidden dependencies.
3. Formatting, Clippy, unit/integration/doc checks and release build.
4. Native smoke checks for the affected UI/file operations.
5. Updated user/run instructions, task evidence and known limitations.

Read [RUST-ENGINEERING.md](RUST-ENGINEERING.md) for the concrete Rust standards, pointer choices, error taxonomy and check commands. Calendar estimates follow the measured prototype and team capacity; this roadmap intentionally promises dependency order and outcomes rather than unsupported dates.
