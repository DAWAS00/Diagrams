# 06 — Reliability and validation

## What “running well” means

The app starts predictably, preserves data, responds to input, explains failures and supports a complete modeling workflow. It does not mean the window opens once or a static canvas looks correct.

## Meaningful automated coverage

| Layer | Tests that protect behavior |
|---|---|
| Core | Reversible command sequences; validation leaves state untouched; no dangling IDs after deletion; relation type constraints; inheritance-cycle rejection |
| Multi-view | Rename updates shared semantic data; movement/style updates one view; view removal preserves model; model deletion/undo handles dependents |
| Geometry | Transform roundtrips at zoom extremes; finite coordinate guards; hit ordering; edge attachment after move/resize; label bounds |
| Interaction | Press/move/release yields one transaction; Escape/capture loss cancels; text focus suppresses canvas shortcuts |
| Storage | Save/reopen equivalence; transaction rollback; disk-full/access-denied/replacement failures; schema migration and newer-file refusal |
| Background work | Save acknowledgment for R does not clear dirty state at R+1; stale load results rejected; bounded save queue and graceful close |
| Export | Well-formed SVG; escaped text; correct scene bounds; arrowheads and members visible; selection/editor UI omitted |

Use table-driven fixtures and targeted generated command sequences. Compare semantic snapshots with normalized IDs for generated fixtures; do not assert volatile timestamps or accidental rendering implementation details. Add a property-testing dependency only when the first invariant tests identify a useful generated space.

## Native UI checks

- Fresh launch, New/Open/Save As/Close and actual file reopen.
- Mouse and keyboard creation/rename/connect/reuse workflows.
- 100%, 150% and 200% Windows scaling; monitor move; minimum supported window size.
- Keyboard-only navigation and screen-reader object-list/inspector flow.
- English, Arabic, mixed direction, IME composition, clipboard text and multiline member names.
- Pointer leaves window during drag, Escape cancels, text undo remains text undo.
- Unsaved project close, save failure, recovery snapshot and restart recovery.
- Native dialog cancellation leaves app state unchanged.
- Run the release artifact on a Windows machine without the developer toolchain.

M0 verifies the essential framework risks. M3 runs the complete workflow. Framework AccessKit support does not automatically make custom canvas objects accessible; test our mapping and keyboard alternative explicitly.

## Initial performance budgets

Proposed targets to calibrate in M0/M2 on this machine; no benchmark has been run yet:

| Workload | Initial target |
|---|---|
| Cold open to usable empty editor | Under 3 seconds on the recorded reference system |
| Active drag/zoom | Aim for 60 Hz; measure p95 frame time at or below 16.7 ms for the baseline fixture |
| Baseline project | 200 visible nodes, 300 edges; no save/layout work blocking pointer input |
| Medium document | 1,000 nodes, 1,500 edges; viewport culling and cached text/layout; report degradation rather than assuming baseline speed |
| Save/load baseline | Under 1 second on local NTFS, with responsive UI and explicit progress for larger work |
| Idle | Event-driven repaint where possible; no unnecessary continuous high CPU/GPU redraw |

Record CPU/GPU, RAM, build profile, viewport, scaling, graph distribution, fixture seed and measurement procedure. If the targets are missed, profile before adding spatial indexes, worker layout or a new renderer. A custom R-tree or incremental graph router is not an initial dependency.

## Persistence failure policy

No `unwrap`/panic for user-file or routine graphics errors. Show the affected operation/path and actionable retry/recovery options. A failed save does not clear dirty state. The old committed file remains available. Recovery snapshots have timestamps and are offered explicitly; do not overwrite a user's deliberate saved file with a recovery draft.

Use limits on counts, nested structures, text lengths and payload bytes when loading; validate cross-references before publishing a loaded document. Test bad or truncated inputs. Future importers must stage into a new document until validation succeeds.

## Release evidence

The v0.1 handoff should include the exact toolchain/Cargo.lock, build instructions, fixture files, passed automated checks, native smoke-test results, known limitations, performance measurements and distribution/runtime prerequisites. Report intended targets separately from measured outcomes.

No test suite was added or run in this planning pass because application code does not yet exist. The plan/prototype files are checked for internal links and expected content only.
