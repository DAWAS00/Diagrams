# Slice S10: storage core (feat/p5-storage-core)

Base: `main` at `6a81314`. Baseline: 233 tests pass, maintainability 0 FAIL and 7 WARN. Roadmap tasks it contributes to: R1-05, R1-06 and the storage half of R3-03 (see [BUILD-PROGRAM.md](BUILD-PROGRAM.md)). All of it lives in `diagram-storage`; it has no UI and no GUI dependency. The window uses it in S11.

## Scope

A project file is one SQLite database (ADR-002). This slice defines **schema v1** and four operations over an in-memory `diagram_core::model::Document` plus a project name.

| Module | Work |
|---|---|
| `schema.rs` | `SCHEMA_VERSION = 1`, an `application_id` that marks a Diagram Studio file, the tables and constraints, and `user_version` as the schema version. |
| `dto.rs` | Plain data structs (`ProjectDto` and its row structs) that mirror the tables, separate from the in-memory model. `from_document` copies the model out; `into_document` rebuilds and validates it. Text forms of visibility, relation kind and notation live here. |
| `save.rs` | `save(path, name, &Document)`: write a **new temporary file in the same folder**, in one transaction; check it (`PRAGMA integrity_check`, reopen read-only, load it, compare with the document); only then replace the target. Any failure removes the temporary file and leaves the previous file untouched. Failure points can be injected so tests can prove it. |
| `load.rs` | `load(path)`: open **read-only**, check the application ID and version, run the integrity check, read in one read transaction, rebuild the document through the core's own rules (`Name`, `Multiplicity`, `Rect`) and `check_document`. Nothing is created or rewritten. |
| `recovery.rs` | Autosave snapshots in a recovery folder: write one (same atomic save), list them with their modified time, recover one (same checked load), discard one. A newer snapshot replaces the older one atomically. |
| `error.rs` | Typed errors: `SaveError` (cause: could not create or write, SQLite, verification mismatch, could not replace, injected) and `LoadError` (not found, not a Diagram Studio file, newer schema, damaged, invalid content with the core's messages). Each has a message in words. |

## Settled by review (Codex plan round 1)

- **Publication primitive.** `std::fs::rename` over the target (on Windows `MoveFileExW` with replace-existing: one atomic step, the target is never deleted first, so it is the old project or the new one). All connections are closed before the swap. `ReplaceFileW` is not used, because its failure states can leave the original moved aside.
- **Durability.** SQLite writes with `synchronous = FULL` in a rollback journal (`journal_mode = DELETE`, read back and checked); the finished file is then flushed with `sync_all` before the swap. The standard library cannot flush the folder entry, so **no power-loss guarantee is claimed**: the swap is atomic against crashes and errors of the program, not against loss of power. No write-ahead log, so no sidecar files are ever published.
- **Untrusted input.** Loading reads the first 100 bytes itself and refuses a file that is not SQLite, or that is in write-ahead-log mode, *before SQLite opens it* (opening such a file, even read-only, creates `-wal` and `-shm`). It then opens read-only with `trusted_schema = OFF` and `query_only = ON`, and reads at most 1,000,000 rows per table.
- **One read transaction.** Loading begins a read transaction before it reads the version, runs the integrity check and reads every table, so a concurrent writer cannot give a mixed document. A counter above the signed range is damaged.
- **Foreign keys.** On while writing (before the transaction). `integrity_check` does not cover them, so loading does not rely on them: every reference is checked in Rust.
- **Validation.** The core's `check_document` errors, plus checks the core does not make for a file: positions used twice (members, parameters, diagrams, views, connectors), a class shown twice in one diagram, a repeated generalization, roles or multiplicities on a generalization, parameters on an attribute or on a missing member, unknown tags, impossible geometry, IDs used twice. Core *warnings* stay saveable.
- **ID counter.** `next_id` is the last ID handed out (the core increments before it returns one), so it may equal the largest ID and is stored and restored **verbatim**, never recomputed; a counter below the largest ID is damaged. IDs are stored as signed integers with checked conversion.
- **What a save may replace.** Identity is read from the file's own header bytes (no SQLite open): a project of this or an older format (including a damaged one, which saving repairs; even one in write-ahead-log mode), or an empty file. A project from a newer format is refused (`NewerTarget`) and any other file is refused (`ForeignTarget`); S11 decides how the window offers an explicit overwrite.
- **Crash leftovers.** A returned error removes its temporary file. A killed process can leave `.<name>.saving-*`; `remove_orphans(path)` deletes only those for that project and is called when the project is opened. The temporary file is created exclusively.
- **Recovery keys** are chosen by the caller (a stable project identity, not the project name, at most 48 bytes). The file name is a readable prefix plus the whole key in hexadecimal, so different keys never share a snapshot (`project:a` and `project?a` stay separate); `discard(folder, path)` removes only a regular snapshot file directly inside the folder.
- **Name.** `load` returns the project name with the document, and the save's read-back check compares both.

## Rules this must keep

- **No silent rewrite.** A file with a newer schema, a damaged file or a non-project file is refused with a typed error and its bytes are unchanged afterwards. v1 has no older versions to migrate, so there is no migration yet; the version check is where one would go.
- **Previous data survives a failed save.** Proved by injecting a failure at each step (create, write, check, replace) and comparing the target file byte for byte before and after, plus no temporary file left behind.
- **Round trip is exact.** `load(save(d)) == d` for a multi-view project with operations, parameters, roles, multiplicities, a generalization, z-order and an Arabic name, including the ID counter, so IDs are never reused after reopening.
- **Core stays free of SQL; storage stays free of widgets.** `diagram-storage` depends on `diagram-core` and `rusqlite` only (plus `thiserror`).
- **IDs** are stored as integers; a value above `i64::MAX` is refused as damaged.
- **No new roadmap status.** R1-05 and R1-06 stay open until their dependencies and the UI integration (S11) hold; this slice only produces the evidence.

## Tests (written first)

- `tests/schema.rs`: a saved file records the application ID and version 1; a file with a newer `user_version` is refused with `NewerVersion` and is byte-identical afterwards; a plain SQLite file and a text file are `NotAProject`; an empty file is refused.
- `tests/atomic_save.rs`: injected failure at each step returns the matching typed cause, leaves the previous file byte-identical and leaves no temporary file; a first save to a new path leaves nothing behind on failure; a successful save replaces the file and leaves no temporary file; saving over a path whose folder does not exist fails with a clear cause.
- `tests/load.rs`: exact round trip (including the ID counter and Arabic text); a truncated file is `Damaged`; a tampered file (a view that points to a missing element, a blank name, an invalid multiplicity, a `next_id` below the largest ID, a duplicate position) is refused with the reason and nothing is returned; a missing file is `NotFound`.
- `tests/recovery.rs`: the recovered document equals the last snapshot written; a second snapshot replaces the first; listing shows both projects' snapshots newest first; discarding removes only that snapshot; a damaged snapshot is reported and does not hide the others.
- Added after review: a failure at the flush step; a newer-format, foreign, empty and damaged target; leftovers of a killed save and `remove_orphans`; a write-ahead-log file refused with no sidecar created; wrong-column tables; row-level shape checks (positions used twice, class shown twice, repeated generalization, ends on a generalization, parameters on an attribute); the counter kept verbatim after deleting the highest ID; `discard` confined to the folder.
- Adversarial case for review: a save under a reader that holds the file open. On Windows SQLite opens files without delete sharing, so the swap fails; the test accepts a clean success or a typed `Replace` error and checks the old bytes survive and no temporary file remains. A killed saver is simulated by leftover files, not by a child process (see Limits).

## Limits (stated, not hidden)

No child-process kill test and no power-loss test: the first would be flaky in CI and the second cannot be run here. The guarantees rest on the single atomic rename plus the flush, on the injected-failure tests at every step, and on the leftover-file test. A real crash test belongs in the R3-05 profiling and distribution evidence (S13).

## Gates

fmt, clippy `-D warnings`, test and doc from `app/`; maintainability 0 FAIL and WARN not above 7; roadmap `--check` (no status change). No real window is involved; the slice adds no UI.
