# Project instructions

Diagram Studio: a native Rust/egui diagram editor. The application workspace is `app/`, and the plan and tracker are in `plan-rust-diagram-editor/`.

## Rules for any UI change (mandatory)

Read and follow **[plan-rust-diagram-editor/UI-RULES.md](plan-rust-diagram-editor/UI-RULES.md)** before changing the UI. In short:

- Every control has a visible label and an accessible name, is at least 32 points high, and fits without overlap at 800×500, 1280×800 and 1440×900.
- Disabled controls say why. Errors and warnings are words. Fields keep focus when an error appears.
- Everything works from the keyboard. Escape closes text field → dialog → drawer → selection, and focus returns to the opener.
- Use only the design tokens and text roles in `ui/theme.rs`, and the shortcuts in `ui/commands.rs`.
- Add headless tests and add the screen to `tests/ui_audit.rs`. Every bug fix gets a test proven to fail without the fix. Check the real window and save screenshots.
- Before committing, run fmt, clippy `-D warnings`, test and doc from `app/`, and the tracker validator if a status changed. Mark a task `done` only when its acceptance text and the UI rules hold.

Shared UI progress page: `app/docs/ui/README.md`. Plan and task definitions: `plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md` and `roadmap.json`.
