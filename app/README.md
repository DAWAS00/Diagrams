# Diagram Studio — application workspace

R0-01 created the buildable Rust foundation. R0-02 adds a native eframe/egui shell prototype (toolbar, sidebar, canvas with two selectable preview classes, inspector with a real name field). R0-03 links bundled SQLite. Project persistence and real diagram editing are not implemented yet. Both tasks passed Windows verification on 2026-11-05; see [docs/R0-02-R0-03-VERIFICATION.md](docs/R0-02-R0-03-VERIFICATION.md).

## Structure

```text
app/
  Cargo.toml                    Shared metadata, dependency paths and inherited lints
  Cargo.lock                    Reproducible dependency resolution
  rust-toolchain.toml            Rust 1.99.1, Clippy and rustfmt
  crates/
    diagram-core/               Validated project name; future models/commands/history
    diagram-canvas/             Future geometry, interactions, scenes and SVG
    diagram-storage/            Bundled SQLite (R0-03); future schema, migrations and recovery
    diagram-desktop/            Native eframe shell prototype (R0-02) and executable
  scripts/check.ps1             Repeatable Windows quality checks
  docs/                         Implementation evidence and development guidance
```

Dependency direction: desktop → core/canvas/storage; canvas → core; storage → core. Core has no GUI, SQLite or OS integration dependency. Canvas and storage currently define their crate boundaries without placeholder operations. Add modules when real behavior needs them.

## Prerequisites and run

Windows x64 with Rustup and Visual Studio C++ Build Tools / Windows SDK (also used to compile the bundled SQLite C source). Rustup reads the pinned toolchain from this directory; a machine may download it on first use. Linux x64 builds and tests were also checked (needs a C compiler, and at run time libxkbcommon-x11 plus a Vulkan or GL driver).

From `app/`:

```powershell
cargo run -p diagram-desktop --locked
```

If Rust is not on PATH on this computer:

```powershell
Set-Location 'D:\REA visual\app'
& 'C:\Users\dawas\.cargo\bin\cargo.exe' run -p diagram-desktop --locked
```

This opens the native window. Nothing is saved. `cargo run -p diagram-desktop --locked -- --version` prints the application and bundled SQLite versions without opening a window. If the window cannot start, the program prints the cause and a hint, then exits with code 1.

UI progress, evidence and next steps: [docs/ui/README.md](docs/ui/README.md).

## Checks

From any directory on Windows:

```powershell
& 'D:\REA visual\app\scripts\check.ps1'
```

The script resolves Cargo from PATH or the user's standard Rustup installation, changes into this workspace, and stops on the first failed command. It checks formatting, all targets, Clippy, tests (including doc tests), generated API documentation and the release executable. It does not change the user's PATH or default toolchain.

Manual equivalent, from `app/`:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
cargo build -p diagram-desktop --release --locked
```

Tests: `diagram-core/tests/project_name.rs` covers name validation, display, equality and the error message; `diagram-desktop/tests/shell.rs` drives the shell headlessly through its accessibility tree (egui_kittest): layout, disabled actions, canvas and list selection, Escape, renaming and the blank-name warning; `diagram-desktop/tests/theme.rs` checks every design-token contrast pair and live theme switching (UI-01); `diagram-desktop/tests/diagnostics.rs` covers `--version`, usage errors, window settings and launch-failure messages; `diagram-storage/tests/sqlite_smoke.rs` writes, closes, reopens, reads and deletes a temporary database. The roadmap validator has Python tests: from the repository root run `python -m unittest discover -s plan-rust-diagram-editor`.

All member crates inherit the workspace lints. Public library APIs require documentation; owned project names are validated once and borrowed as `&str`. Production unwrap/expect/panic and unsafe code are forbidden by the current lint policy. Build output stays in the ignored `app/target/` directory. Add dependencies when a task first uses them, and review/commit the resulting lockfile changes.

Next: record the Windows results for R0-02/R0-03, then [R0-04 text, focus and accessibility](../plan-rust-diagram-editor/RUST-ROADMAP.md).
