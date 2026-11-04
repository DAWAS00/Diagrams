# Diagram Studio — application workspace

R0-01 creates the buildable Rust foundation. The executable currently prints a console bootstrap message. The native GUI is R0-02; project persistence is not implemented yet.

## Structure

```text
app/
  Cargo.toml                    Shared metadata, dependency paths and inherited lints
  Cargo.lock                    Reproducible dependency resolution
  rust-toolchain.toml            Rust 1.99.1, Clippy and rustfmt
  crates/
    diagram-core/               Validated project name; future models/commands/history
    diagram-canvas/             Future geometry, interactions, scenes and SVG
    diagram-storage/            Future SQLite, migrations and recovery
    diagram-desktop/            Executable and future native UI/worker orchestration
  scripts/check.ps1             Repeatable Windows quality checks
  docs/                         Implementation evidence and development guidance
```

Dependency direction: desktop → core/canvas/storage; canvas → core; storage → core. Core has no GUI, SQLite or OS integration dependency. Canvas and storage currently define their crate boundaries without placeholder operations. Add modules when real behavior needs them.

## Prerequisites and run

Windows x64 with Rustup and Visual Studio C++ Build Tools / Windows SDK. Rustup reads the pinned toolchain from this directory; a machine may download it on first use. Compilation for other platforms has not been verified.

From `app/`:

```powershell
cargo run -p diagram-desktop --locked
```

If Rust is not on PATH on this computer:

```powershell
Set-Location 'D:\REA visual\app'
& 'C:\Users\dawas\.cargo\bin\cargo.exe' run -p diagram-desktop --locked
```

The bootstrap uses a validated project name and exits. It does not open a window or save anything.

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

Tests: `diagram-core/tests/project_name.rs` covers name validation, display, equality and the error message; `diagram-desktop/tests/bootstrap.rs` covers the bootstrap text, writer failures and the real executable. The roadmap validator has Python tests: from the repository root run `python -m unittest discover -s plan-rust-diagram-editor`.

All member crates inherit the workspace lints. Public library APIs require documentation; owned project names are validated once and borrowed as `&str`. Production unwrap/expect/panic and unsafe code are forbidden by the current lint policy. Build output stays in the ignored `app/target/` directory. Add dependencies when a task first uses them, and review/commit the resulting lockfile changes.

Next: [R0-02 native shell](../plan-rust-diagram-editor/RUST-ROADMAP.md). The GUI and database prerequisites remain unverified until their own tasks pass.
