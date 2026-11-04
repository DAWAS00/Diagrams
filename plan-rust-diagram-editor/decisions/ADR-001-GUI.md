# ADR-001 — Rust desktop GUI

Status: proposed; conditional on M0 validation. Date: 2026-11-04.

## Context

The user requires Rust and wants a diagram editor easier to use than the studied application. We need custom canvas input plus ordinary text fields, trees, inspectors, dialogs and keyboard access. Windows offline desktop is the initial target.

## Alternatives

| Option | Why consider it | Project tradeoff |
|---|---|---|
| egui + eframe | Rust-first immediate-mode controls and drawing, native framework | Custom canvas semantics, polished styling, shaping/accessibility and state discipline need explicit work |
| Iced | Rust GUI with an Elm-inspired message/state approach | Viable alternative; we would build custom diagram interactions and renderer adapters around its UI model |
| Slint | Declarative UI with Rust integration | Attractive for a designed shell; evaluate custom large-canvas workflow and selected distribution terms if reconsidered |
| Tauri | Desktop shell with Rust application logic and a web frontend | Adds webview/frontend and IPC boundaries; useful if HTML/CSS UI becomes a product requirement |

These are project judgments, not objective performance rankings. [egui](https://github.com/emilk/egui), [eframe](https://docs.rs/eframe/0.37.0/eframe/), [Iced](https://github.com/iced-rs/iced), [Slint](https://slint.dev/), [Tauri](https://tauri.app/start/).

## Proposed decision

Use eframe 0.37.0 and its egui re-export for M0. Build the core and scene independently of the GUI. Prefer a simple fixed/resizable shell over arbitrary docking and floating windows.

## Rationale and accepted tradeoffs

The custom canvas and direct Rust integration fit the first vertical slice. A single application language simplifies ownership and debugging. We accept responsibility for deliberate visual design, custom hit testing, canvas accessibility, text metrics and strict separation between ephemeral widget state and document commands. Immediate-mode rendering must not rerun persistence or semantic mutations every frame.

## Gate / revisit trigger

Before committing, test English/Arabic/mixed text, IME, screen-reader/keyboard access, Windows scaling, pointer capture and real adapter startup. If essential requirements fail and bounded mitigation is not credible, compare Iced/Slint with the same prototype and revise this ADR. Do not hide those risks behind a visually attractive mockup.

The HTML concept in this plan is solely a review artifact. Selecting it does not imply selecting Tauri or a web frontend.
