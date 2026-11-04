# Rust engineering rules for the roadmap

Applies to future application code. Based on the invoked [rust-best-practices skill](/C:/Users/dawas/.agents/skills/rust-best-practices/SKILL.md) and its chapters; selections below are project-specific policy rather than a copy of every example.

## 1. Ownership and memory

| State/data | Owner | Access policy |
|---|---|---|
| Authoritative Document and command history | Desktop editor session | Read by borrow; mutate only through validated commands |
| Selection, tool, hover, drafts, viewport | UI editor state | Ordinary mutable state on UI thread; does not create semantic history by itself |
| Geometry/scene/text cache | Canvas module state | Derived from document/style/font inputs, invalidatable |
| Save/load/export request | Transferred to worker | Owned request with bounded immutable snapshot and typed identity tokens |
| SQLite connection | Worker thread | Created/used/dropped there; never shared with widgets |
| Worker result | Returned to UI | Owned result; checked for correct session/request before publication |

Use `&str`, `&Path`, `&[T]`, `&T` for read-only APIs and `&mut T` for exclusive mutation. An API that retains/sends data should accept ownership explicitly. Small plain IDs/coordinates may be Copy; large documents, strings and vectors are not copied on every frame.

Store graph edges as typed IDs into owned maps, not a graph of `Rc<RefCell<Node>>` objects. Use deterministic ordering for saved data and fixtures; a HashMap's incidental iteration order is not document order. Member/Z-order vectors are explicit where order matters.

Cloning a snapshot at a save boundary is a justified owned copy; cloning the whole document every frame or per pointer move is not. Start with bounded/coalesced snapshots and measure peak memory. Consider structural sharing only if profiling justifies its complexity. Borrowed data cannot escape the frame into a `'static` worker request.

Use `Arc<T>` only for genuine shared immutable ownership across threads; `Arc::clone` clones the handle, not the payload. It does not make arbitrary `T` thread-safe. Auto-trait bounds depend on the contained types, and the compiler must enforce Send/Sync; do not use blanket pointer tables as guarantees. [Rust Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html).

No application-wide `Arc<Mutex<Document>>`, unsafe Send/Sync implementations, or global mutable state. If a small shared flag is needed, use the smallest appropriate primitive and document why. Default core/canvas code stays safe Rust; any platform FFI later needs a narrow reviewed adapter and explicit safety invariants.

Skill references: Chapter 1 ownership/idioms, Chapter 3 clone/allocation decisions, Chapter 9 pointer ownership.

## 2. Types, lifecycle and state transitions

- Distinct ElementId, RelationId, DiagramId, ViewId and MemberId newtypes.
- Checked geometry constructors reject NaN/infinity, invalid dimensions and out-of-policy coordinates.
- `ElementKind`, `RelationKind` and `Command` are enums for the supported compiled-in set.
- Runtime canvas state is an enum, not a collection of mutually inconsistent booleans.
- Storage parses into an untrusted DTO, validates it, then constructs a ValidatedDocument/LoadOutcome with private invariants. A simple validated wrapper is preferable to a large generic typestate hierarchy.
- Use a ReadyProject/validated snapshot boundary if it prevents publishing unchecked data. Add typestate only when it removes a meaningful invalid operation; do not apply PhantomData to ordinary selection gestures.

Use concrete types first. Generics are useful for real strategies/test seams with a small contract; trait objects are appropriate for genuinely heterogeneous runtime behavior. Do not box every command, node, style or renderer merely to imitate an object hierarchy. Extract shared decisions after actual reuse, not guessed future plugin requirements.

Skill references: Chapters 6 and 7; Chapter 1 extraction guidance.

## 3. Command pipeline and dirty-state identity

Input → owned intent → validation/preconditions → reversible ChangeSet → one commit → invalidation/events. Keep I/O out of command application. Resolve invalid IDs/targets before mutation, and make failure leave state untouched. Store deltas for affected records, not whole-project copies per history entry.

Distinguish:

- **SessionId:** identifies the currently open document lifetime.
- **Revision:** monotonic edit sequence for snapshots and stale-result checks.
- **StateId:** identifies a history state; undo can return to the saved StateId even though Revision increases.
- **SaveRequestId:** identifies one specific destination/request/result.

Dirty state compares current and saved state identity, not just a boolean or a timestamp. After save(snapshot R), acknowledge that snapshot's state only. If a new edit or another document is active, an older result must not make it appear saved. Initial Save As updates the active path only after successful publication. Coalesce pending autosaves, but preserve explicit-save/close ordering.

Treat scene rebuilds and cached text metrics as derived effects; they must not create semantic commands. Pointer previews are temporary and disappear on cancellation, capture loss or tool change.

## 4. Errors and recovery

| Error type | Examples | UI response |
|---|---|---|
| DomainError | Missing model, incompatible endpoints, cycle, invalid member/geometry | Explain next to the action; leave document unchanged |
| StorageError | Unsupported schema, invalid references, unavailable path, permission/disk/transaction/publication failure | Keep dirty state, show persistent retry/Save As/recovery choices |
| ExportError | Invalid target, unsupported scene operation, write failure | Preserve project; identify export limitation/destination |
| WorkerError | Closed channel, canceled optional job, stale response, unavailable worker | Distinguish routine cancellation from real failure; prevent false success |
| StartupError | Graphics/window/backend initialization | Useful startup diagnostic; no silent loop or pretend successful editor |

Library crates return typed `Result<T, E>` using thiserror, preserve sources and use `?`. Keep anyhow optional at the executable edge for context; do not erase domain error variants that callers need to handle. Log a failure at its handling boundary instead of logging the same propagated error at every layer.

No production `unwrap()`/`expect()`, `todo!()`/`unimplemented!()` in reachable shipped paths, or panic for malformed user files. Tests may unwrap known setup results. Cancellation of a file picker is a normal outcome, not an error. Avoid defaults that hide invalid loaded IDs or corrupted fields.

Skill reference: Chapter 4.

## 5. Workspace checks and lint policy

Define shared lints at workspace level and opt member packages into inheritance using `[lints] workspace = true`; workspace definitions alone do not apply them to every member. [Cargo workspace lints](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table).

Use conservative compiler/Clippy correctness/style/performance checks; explicitly inspect redundant_clone, needless_collect, clone_on_copy and large_enum_variant. Do not turn on every pedantic/nursery lint without triage. Prefer local `#[expect(..., reason = "...")]` when the warning is genuinely justified and the pinned toolchain supports it; do not silence a whole module to clear a build.

At library roots require public documentation with `#![deny(missing_docs)]`, including crate/module docs and meaningful public errors/invariants. Keep API surface small instead of exposing internal structs to avoid design work. Enforce no production unwrap/expect at appropriate source boundaries; allow explicit test setup rather than weakening production rules globally.

Checks run from `app/`, where Cargo.lock now exists. [scripts/check.ps1](../app/scripts/check.ps1) provides the repeatable Windows gate; [R0-01 evidence](../app/docs/R0-01-VERIFICATION.md) records the current passed results.

```powershell
$cargoExe = 'C:\Users\dawas\.cargo\bin\cargo.exe'
& $cargoExe fmt --all -- --check
& $cargoExe check --workspace --all-targets --locked
& $cargoExe clippy --workspace --all-targets --all-features --locked -- -D warnings
& $cargoExe test --workspace --locked
& $cargoExe test --workspace --doc --locked
& $cargoExe doc --workspace --no-deps --locked
& $cargoExe build -p diagram-desktop --release --locked
```

The explicit doc-test command makes the gate clear; `cargo test` normally includes documentation tests, so there is no need to repeat it after an unchanged successful run merely for more output. If a future runner such as nextest omits doc tests, keep the doc-test command separately. [Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html).

`--all-features` is appropriate while features are additive/compatible. If backend combinations become mutually exclusive, document/test supported feature combinations instead of assuming all-features validates them. Review lockfile changes; `--locked` failure is not an instruction to update every dependency blindly. [Clippy usage](https://doc.rust-lang.org/clippy/usage.html).

Use stable rustfmt options; do not add nightly-only formatting requirements to the stable toolchain casually. The R0-01 workspace passes these checks with Rust 1.99.1. GUI, SQLite, file handling and editor behavior require their own later verification.

Skill reference: Chapter 2.

## 6. Tests, benchmarks and docs

Name tests for a behavior and condition, for example `rename_should_preserve_view_geometry_when_model_is_shared`, `save_ack_should_keep_newer_edits_dirty`, and `connect_should_reject_inheritance_cycle_without_changes`.

One behavior per focused unit test; use multiple assertions when they establish one meaningful invariant/roundtrip. Share fixture setup where useful; keep actions and expected outcomes visible. Do not split every field into a ceremonial test or create helpers that merely restate the implementation.

Core tests cover commands/inverses/reference constraints. Canvas tests cover transforms/hit targets/cancellation. Storage integration tests cover real temporary databases, fault injection, migration and publication. Desktop headless integrations cover session/request handling; native smoke tests cover real graphics/text/input. Snapshot small normalized scene/SVG structures, not huge volatile projects or the critical graph invariants alone. Add insta only if it improves review over plain golden fixtures.

Benchmark optimized release builds with recorded hardware, scaling, fixture and workload. Profile the p95 frame work, text/route generation, allocation and I/O boundaries before introducing indexes, arenas or concurrent layout. Do not use microbenchmark results to claim whole-program superiority.

Use `///`/`//!` for what public APIs do, errors and examples. Comments explain non-obvious reasoning, platform restrictions and measured optimizations. Future TODOs must reference an actual tracked task; local R*-IDs may link to this roadmap until remote issues are explicitly created. No source-hosting messages or issues are created automatically.

Skill references: Chapters 3, 5 and 8. The local skill is in `C:\Users\dawas\.agents\skills\rust-best-practices`; reviewed chapters 01–09 informed this document. Illustrative skill examples are not copied as authoritative thread-safety or API rules where compiler/reference contracts are more precise.
