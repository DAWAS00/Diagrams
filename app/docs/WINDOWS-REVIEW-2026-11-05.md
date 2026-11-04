# Windows sync and verification — 2026-11-05

Synced `main` by fast-forward from `1b40ce5` to GitHub commit `17c20ea`. Local Rust skill files, README addition and ignore rules were preserved and restored. This report records the Windows verification against that source baseline; publication includes the evidence and updated task tracker.

## Results on this Windows x64 MSVC computer

| Check | Result |
|---|---|
| `app/scripts/check.ps1` | Passed all six commands: fmt, all-target check, Clippy with warnings denied, tests, Rustdoc and release build |
| Rust tests | 29 integration + 1 doc test passed; 0 failures or ignored tests. Core 9, diagnostics 7, shell 11, SQLite 2. The extra Linux missing-display test is correctly excluded on Windows |
| Python roadmap tests | All 16 passed |
| Five generator scripts | All ran; output matches committed content after CRLF/LF normalization |
| Debug and release `--version` | `Diagram Studio 0.1.0-dev`, `SQLite 3.54.1 (bundled)` |
| Native debug window | Opened through wgpu; toolbar, sidebar, canvas, inspector and status bar rendered |
| Debug interactions | Model-list selection, canvas Order selection, inspector rename to Customer Windows, name reflected in list/canvas/status, first Escape leaves text focus, second Escape deselects, empty-canvas click deselects |
| Mixed text sample | `Order طلب` inserted in the focused Name field; glyphs visible in field/list/canvas and the editable accessibility value preserved the string. This is a sample, not a complete bidi/input validation |
| Native release window | Opened and rendered the prototype successfully; left open for local review |
| Release PE imports (`dumpbin /dependents`) | No sqlite3.dll import. VCRUNTIME140.dll and Windows runtime libraries are imported; clean-machine distribution remains unverified |

No application source fix was needed for these checks. Generated-file differences were line endings rather than content changes. Automated test success covers the implemented prototype, not the future editor.

## Plan status

- **R0-01 done:** workspace and core name value.
- **R0-02 done:** native shell now has Windows debug/release launch evidence plus automated interaction and diagnostic tests.
- **R0-03 done:** bundled SQLite now compiles/links with MSVC and passes the actual temporary-file roundtrip on Windows.
- **R0-04 in progress:** initial English/mixed-text and focus observations exist, but the full matrix below is incomplete.
- **R0-05 not started:** framework gate follows completed R0-04 evidence. **M0 remains open.**

## Next work, in order

1. Finish R0-04: choose/license a dependable font setup; validate Arabic joining, bidi cursor/selection/editing, pure Arabic and mixed names, paste, IME, Tab/arrow navigation, Narrator/NVDA and 100/150/200% scaling. The accessibility tool's reconstructed document text differed from the correct editable value in the mixed-text sample; investigate with real screen readers before claiming accessibility correctness.
2. Apply the planned light default and deliberate text sizes; preserve an explicit dark-theme choice. The current shell follows the OS theme. Review longer labels and smaller windows.
3. Complete R0-05 and record the GUI framework decision in ADR-001 with the full evidence and limitations.
4. Begin M1: typed IDs and model/view records, validation, atomic reversible commands, history, versioned project schema, save/recovery and worker protocol.
5. M2 makes the prototype a real class editor; M3 adds reliability, export and distributable v0.1.

The two preview classes are desktop-owned sample data. Add class, Connect, Undo, Redo, Save and Export are disabled. Model reuse, relationships, member editing, dragging/resizing, project save/load/recovery and SVG are not implemented. A bundled database engine is not yet a project persistence implementation.

Run locally from `app/`: `cargo run -p diagram-desktop --locked`, or open `app/target/release/diagram-desktop.exe`. If Cargo is absent from PATH, use `C:\Users\dawas\.cargo\bin\cargo.exe` as documented in [the workspace README](../README.md).
