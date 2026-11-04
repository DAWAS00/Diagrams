# 04 — Toolchain and dependencies

## Readiness checked on 2026-11-04

| Item | Local observation | Next development step |
|---|---|---|
| OS | Local Windows x64 | First supported build/test target |
| Rust | `rustc 1.99.1`, stable x86_64-pc-windows-msvc | Propose pinning 1.99.1 after M0 build verification |
| Cargo | `cargo 1.99.1` | Available at the path below |
| PATH | Rust tools not resolved by the current shell | Use absolute path initially; choose any user PATH update explicitly when implementing |
| MSVC | VS 2022 Build Tools, toolset 14.44.35207 | Validate native linking and bundled SQLite compilation in M0 |
| SDK | Windows SDK library directory 10.0.26100.0 exists | Validate headers/libraries through an actual build |
| Git | Installed | Initialize/use a repository when application implementation starts |

Installed Rust executables:

```text
C:\Users\dawas\.cargo\bin\rustup.exe
C:\Users\dawas\.cargo\bin\cargo.exe
C:\Users\dawas\.cargo\bin\rustc.exe
```

Build Tools: `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`.

No SDK, compiler, Rust, database or GUI package was installed in this planning pass. Discovery is not a successful GUI build; M0 establishes that result.

## Proposed dependency inventory

| Dependency | Use | Constraint |
|---|---|---|
| `eframe` / its `egui` re-export | Native window/event loop, controls and canvas drawing | Candidate 0.37.0; checked MSRV 1.95, below local Rust 1.99.1 |
| `rusqlite` with `bundled` | SQLite project persistence | Compile bundled SQLite with existing MSVC; no end-user database server |
| `serde`, `serde_json` | Typed snapshot/export metadata, bounded property serialization | Explicit versioned types; no unchecked deserialization directly into live state |
| `uuid` | Stable element/diagram/view/member IDs | Wrap in distinct newtypes; generate IDs outside deterministic fixture comparisons |
| `thiserror` | Typed domain/storage errors | Convert into clear user-facing messages at the desktop boundary |
| `rfd` | Native open/save dialogs | Follow Windows UI-thread ownership and cancellation semantics |
| `tracing`, `tracing-subscriber` | Startup and save/render diagnostics | Bounded files; no project bodies or secrets in logs |
| `std` threads / channels | Background persistence and export | Bounded request queue; avoid adding a runtime for offline work |
| Rust SVG writer module | Small declared export subset from the scene | Escape text/attributes; validate output and font behavior |

Use one eframe-aligned egui version; prefer the re-export to accidental mismatches. Resolve exact versions of other crates during M0 from published releases, review feature/MSRV compatibility, then commit Cargo.lock. Do not use wildcard dependency versions or Git main for the application.

Initial graphics choice: eframe's wgpu backend on Windows. M0 tests initialization on the actual adapter and clear failure diagnostics. Investigate the glow backend only if the tested target needs it; do not promise automatic GPU fallback before verifying the selected features/backends. No custom shader engine is needed for the basic editor.

Fonts/icons: original vector icons and a redistributable font with English/Arabic coverage as required by the text-input gate. Review the font license and preserve notices. Font shaping, mixed direction and fallback rendering are validation tasks; a font file by itself does not solve them.

## Build workflow after the workspace exists

```powershell
& 'C:\Users\dawas\.cargo\bin\cargo.exe' fmt --all -- --check
& 'C:\Users\dawas\.cargo\bin\cargo.exe' clippy --workspace --all-targets --locked -- -D warnings
& 'C:\Users\dawas\.cargo\bin\cargo.exe' test --workspace --locked
& 'C:\Users\dawas\.cargo\bin\cargo.exe' run -p diagram-desktop --locked
& 'C:\Users\dawas\.cargo\bin\cargo.exe' build -p diagram-desktop --release --locked
```

These are planned commands, not executed validation results. Cargo may download declared dependencies during implementation; build artifacts can be large and will remain under the future application's target directory.

Use a clean Windows machine without developer tools to validate distribution later. Begin with an unpacked release build and included notices; choose an installer only after startup, runtime-library and graphics requirements are known. No end-user Rust/Node/Java installation should be required, subject to that packaging test.

## Official references reviewed

- [eframe 0.37 documentation](https://docs.rs/eframe/0.37.0/eframe/) — native framework and backend features.
- [eframe release metadata](https://crates.io/crates/eframe/0.37.0) — version/MSRV/license baseline.
- [egui repository](https://github.com/emilk/egui) — immediate-mode approach and optional AccessKit support.
- [Rust installation](https://rust-lang.org/tools/install/) — Windows toolchain requirements.
- [rusqlite](https://github.com/rusqlite/rusqlite) — Rust SQLite bindings and bundled build option.
- [Serde](https://serde.rs/), [UUID](https://docs.rs/uuid/latest/uuid/), [thiserror](https://docs.rs/thiserror/latest/thiserror/), [tracing](https://docs.rs/tracing/latest/tracing/) — proposed support libraries.
- [rfd documentation](https://docs.rs/rfd/latest/rfd/) — native dialogs.

Library releases change. Recheck the resolved version set when implementation begins; this document records a reviewed baseline rather than a promise about future latest releases.
