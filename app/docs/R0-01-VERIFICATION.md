# R0-01 — reproducible Rust workspace

Verified on 2026-11-05 in `D:\REA visual\app`, Windows x64 MSVC. **R0-01 complete. M0 remains incomplete.**

## Delivered

- Four Cargo workspace members with explicit one-way dependency boundaries.
- Edition 2024, pinned Rust 1.99.1, rustfmt and Clippy; application packages are non-publishable.
- Cargo.lock resolves thiserror 2.0.21 and its macro dependencies. GUI and SQLite dependencies are deferred until their tasks use them.
- Shared compiler, Rustdoc and Clippy lints, inherited by every member. Safe Rust, documented public library APIs and explicit errors are enforced.
- Core ProjectName validates non-empty/non-whitespace input without silently changing its Unicode spelling.
- Console bootstrap, Windows check script and development instructions. Canvas and storage are documented crate boundaries with no simulated operations.

The exact Rustup toolchain was installed alongside the existing stable channel using `rustup toolchain install 1.99.1 --profile minimal --component clippy,rustfmt`. No user PATH or default toolchain setting was edited. Rustup also performed its own self-update check during installation.

## Results

| Check | Observed result |
|---|---|
| `rustc --version` inside workspace | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| `cargo --version` inside workspace | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| `scripts/check.ps1` | Completed all six commands with exit 0 |
| `cargo fmt --all -- --check` | Passed |
| `cargo check --workspace --all-targets --locked` | All four crates checked |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed, no warnings |
| `cargo test --workspace --locked` | Three integration tests and one executable Rustdoc example passed; no ignored tests |
| `cargo doc --workspace --no-deps --locked` | All four library documentation targets generated, no warnings |
| `cargo build -p diagram-desktop --release --locked` | Windows release executable built and linked |
| `cargo run -p diagram-desktop --locked` | Debug executable printed the bootstrap and exited 0 |
| Direct `target/release/diagram-desktop.exe` launch | Same bootstrap output, exit 0 |
| Cargo metadata / member manifest inspection | Exactly four members, expected dependency direction, all inherit lints and have `publish = false` |
| `cargo tree -p diagram-core --locked` | thiserror + macro dependencies only; no graphics or SQL dependency |

Integration tests cover empty names, Unicode-whitespace-only names and preservation of mixed English/Arabic spelling. This tests the core string value, not native GUI shaping or text input.

Observed bootstrap output:

```text
Diagram Studio - Rust workspace foundation
Example project name: Untitled
Native GUI: next task R0-02. No project file was created.
```

## Remaining gates

R0-02 must add and launch the native eframe shell. R0-03 must compile/link bundled SQLite and perform the temporary database smoke test. R0-04 covers text input, focus, screen readers, scaling and graphics behavior; R0-05 reviews the actual results. None of these is implied by the successful console build.

Project models, commands, history, persistence, scenes and diagram editing remain future implementation. The `.dgram` format is still a proposal, not a file this bootstrap can open or write. Clean-machine distribution has not been tested.

## References

- [Development commands](../README.md)
- [Engineering rules](../../plan-rust-diagram-editor/RUST-ENGINEERING.md)
- [Implementation roadmap](../../plan-rust-diagram-editor/RUST-ROADMAP.md)
- [Cargo workspace reference](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [thiserror documentation](https://docs.rs/thiserror/2.0.21/thiserror/)
