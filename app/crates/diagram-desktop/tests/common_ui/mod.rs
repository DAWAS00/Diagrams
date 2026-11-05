//! Shared helpers for Agent A's UI tests (TASKS-AND-TESTS.md §2):
//!
//! - harnesses that draw one view from a presentation fixture at 800×500,
//!   1280×800 or 1440×900 in either theme;
//! - an **intent recorder**: the harness state collects every `UiIntent`
//!   the view pushed;
//! - find-by-accessible-name helpers;
//! - the usability audit's layout and Tab-order checks (UI-RULES §2.2).
//!
//! Each test binary uses a different subset, so unused items are allowed.
#![allow(dead_code)]

use std::collections::HashSet;

use diagram_desktop::intent::UiIntent;
use diagram_desktop::ui::theme;
use eframe::egui::accesskit::{Action, Rect, Role, Toggled};
use eframe::egui::{self, Key, ThemePreference, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// The three supported window sizes (UI-RULES 1.2).
pub const SMALL: (f32, f32) = (800.0, 500.0);
/// Default window size.
pub const LARGE: (f32, f32) = (1280.0, 800.0);
/// Large desktop size.
pub const WIDE: (f32, f32) = (1440.0, 900.0);
/// Every supported size.
pub const SIZES: [(f32, f32); 3] = [SMALL, LARGE, WIDE];
/// Both themes.
pub const THEMES: [ThemePreference; 2] = [ThemePreference::Light, ThemePreference::Dark];

/// Draws one view: the same shape as every A view function.
pub type Draw<V> = fn(&mut egui::Ui, &V, &mut Vec<UiIntent>);

/// Harness state: the view being drawn and the intents it pushed.
pub struct Recorder<V> {
    /// The presentation fixture drawn every frame. Tests may replace it to
    /// simulate B's next presentation.
    pub view: V,
    /// Every intent pushed so far, in order.
    pub out: Vec<UiIntent>,
    draw: Draw<V>,
    theme: ThemePreference,
    applied: bool,
}

impl<V> Recorder<V> {
    /// Takes the intents recorded so far.
    pub fn take(&mut self) -> Vec<UiIntent> {
        std::mem::take(&mut self.out)
    }
}

/// A harness drawing `view` with `draw` at `size` in `theme`.
pub fn harness<V: 'static>(
    size: (f32, f32),
    theme: ThemePreference,
    view: V,
    draw: Draw<V>,
) -> Harness<'static, Recorder<V>> {
    let recorder = Recorder {
        view,
        out: Vec::new(),
        draw,
        theme,
        applied: false,
    };
    let mut harness = Harness::builder()
        .with_size(Vec2::new(size.0, size.1))
        .build_ui_state(
            |ui, state: &mut Recorder<V>| {
                if !state.applied {
                    theme::apply(ui.ctx(), state.theme);
                    ui.set_style(ui.ctx().global_style());
                    state.applied = true;
                }
                (state.draw)(ui, &state.view, &mut state.out);
            },
            recorder,
        );
    harness.run();
    harness
}

/// [`harness`] at 1280×800 in the light theme.
pub fn light<V: 'static>(view: V, draw: Draw<V>) -> Harness<'static, Recorder<V>> {
    harness(LARGE, ThemePreference::Light, view, draw)
}

/// Runs a frame and returns the intents it produced.
pub fn intents_after<V>(harness: &mut Harness<'_, Recorder<V>>) -> Vec<UiIntent> {
    harness.run();
    harness.state_mut().take()
}

/// Clicks the control named `name` and returns the intents that produced.
pub fn click<V>(harness: &mut Harness<'_, Recorder<V>>, name: &str) -> Vec<UiIntent> {
    harness.state_mut().take();
    harness.get_by_label(name).click();
    intents_after(harness)
}

/// Focuses the control named `name`, presses Enter and returns the intents.
pub fn press_enter_on<V>(harness: &mut Harness<'_, Recorder<V>>, name: &str) -> Vec<UiIntent> {
    harness.get_by_label(name).focus();
    harness.run();
    harness.state_mut().take();
    harness.key_press(Key::Enter);
    intents_after(harness)
}

/// Bounding box of the control named `name`.
pub fn bounds<S>(harness: &Harness<'_, S>, name: &str) -> Rect {
    harness
        .get_by_label(name)
        .accesskit_node()
        .bounding_box()
        .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0))
}

/// Whether the control named `name` is drawn pressed or checked.
pub fn pressed<S>(harness: &Harness<'_, S>, name: &str) -> bool {
    harness.get_by_label(name).accesskit_node().toggled() == Some(Toggled::True)
}

/// Whether the control named `name` is disabled.
pub fn is_disabled<S>(harness: &Harness<'_, S>, name: &str) -> bool {
    harness.get_by_label(name).accesskit_node().is_disabled()
}

/// Whether the control named `name` has keyboard focus.
pub fn is_focused<S>(harness: &Harness<'_, S>, name: &str) -> bool {
    harness.get_by_label(name).is_focused()
}

/// Whether any node's label or value contains `text`.
pub fn shows_text<S>(harness: &Harness<'_, S>, text: &str) -> bool {
    harness
        .query_all_by(|node| {
            let access = node.data();
            access.label().is_some_and(|label| label.contains(text))
                || access.value().is_some_and(|value| value.contains(text))
        })
        .next()
        .is_some()
}

/// One operable control as the accessibility tree reports it.
#[derive(Debug, Clone)]
pub struct Control {
    /// Accessible name (label, or value for fields without one).
    pub name: String,
    /// Role.
    pub role: Role,
    /// Screen rectangle.
    pub rect: Rect,
    /// Enabled.
    pub enabled: bool,
}

/// Roles a person operates. Plain text labels (selectable text) and egui's
/// own scroll bars are not controls in this sense.
pub fn is_control(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::Tab
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::ComboBox
            | Role::Link
            | Role::ListBoxOption
            | Role::Slider
            | Role::SpinButton
            | Role::Canvas
    )
}

/// Every operable control on screen.
pub fn controls<S>(harness: &Harness<'_, S>) -> Vec<Control> {
    harness
        .query_all_by(|node| {
            let data = node.data();
            !node.is_hidden()
                && is_control(node.role())
                && (data.supports_action(Action::Click) || data.supports_action(Action::Focus))
        })
        .map(|node| {
            let access = node.accesskit_node();
            Control {
                name: access
                    .label()
                    .filter(|label| !label.trim().is_empty())
                    .or_else(|| access.value())
                    .unwrap_or_default(),
                role: access.role(),
                rect: access
                    .bounding_box()
                    .unwrap_or_else(|| Rect::new(-1.0, -1.0, -1.0, -1.0)),
                enabled: !access.is_disabled(),
            }
        })
        .collect()
}

/// The rectangle touches the window.
pub fn visible(rect: Rect, (width, height): (f32, f32)) -> bool {
    rect.x1 > 0.0 && rect.y1 > 0.0 && rect.x0 < f64::from(width) && rect.y0 < f64::from(height)
}

/// `outer` contains `inner`, within half a point.
pub fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x0 <= inner.x0 + 0.5
        && outer.y0 <= inner.y0 + 0.5
        && outer.x1 + 0.5 >= inner.x1
        && outer.y1 + 0.5 >= inner.y1
}

/// The rectangles overlap by more than half a point.
pub fn overlap(first: Rect, second: Rect) -> bool {
    first.x0 + 0.5 < second.x1
        && second.x0 + 0.5 < first.x1
        && first.y0 + 0.5 < second.y1
        && second.y0 + 0.5 < first.y1
}

/// Plan: "hit targets at least 32 units".
pub const MIN_HIT_HEIGHT: f64 = 32.0;

/// Static checks of one screen state: every control on screen has a name,
/// lies inside the window, is at least the hit height, and (unless
/// `floating`, for dialogs and menus that cover other controls by design)
/// does not partly overlap another control.
pub fn audit_layout<S>(
    scene: &str,
    harness: &Harness<'_, S>,
    size: (f32, f32),
    floating: bool,
) -> Vec<String> {
    let mut findings = Vec::new();
    let all = controls(harness);
    let shown: Vec<&Control> = all.iter().filter(|c| visible(c.rect, size)).collect();
    for control in &shown {
        let what = format!("{scene}: {:?} \"{}\"", control.role, control.name);
        if control.name.trim().is_empty() {
            findings.push(format!("{what} has no accessible name"));
        }
        let rect = control.rect;
        if rect.x0 < -0.5
            || rect.y0 < -0.5
            || rect.x1 > f64::from(size.0) + 0.5
            || rect.y1 > f64::from(size.1) + 0.5
        {
            findings.push(format!("{what} extends outside the window: {rect:?}"));
        }
        if rect.height() < MIN_HIT_HEIGHT - 0.5 {
            findings.push(format!(
                "{what} is {:.0} high, below the {MIN_HIT_HEIGHT} hit target",
                rect.height()
            ));
        }
    }
    if !floating {
        for (index, first) in shown.iter().enumerate() {
            for second in &shown[index + 1..] {
                if overlap(first.rect, second.rect)
                    && !contains(first.rect, second.rect)
                    && !contains(second.rect, first.rect)
                {
                    findings.push(format!(
                        "{scene}: \"{}\" overlaps \"{}\"",
                        first.name, second.name
                    ));
                }
            }
        }
    }
    findings
}

/// Presses Tab through the screen and reports enabled controls never focused.
/// `pointer_only` lists controls with a documented keyboard alternative.
pub fn audit_tab_order<S>(
    scene: &str,
    harness: &mut Harness<'_, S>,
    size: (f32, f32),
    pointer_only: &[&str],
) -> Vec<String> {
    let expected: HashSet<String> = controls(harness)
        .into_iter()
        .filter(|c| c.enabled && visible(c.rect, size) && c.role != Role::Canvas)
        .map(|c| c.name)
        .filter(|name| !pointer_only.iter().any(|allowed| name.starts_with(allowed)))
        .collect();
    let mut reached = HashSet::new();
    for _ in 0..(expected.len() * 2 + 4) {
        harness.key_press(Key::Tab);
        harness.run();
        if let Some(node) = harness.query_by(|node| node.is_focused()) {
            let access = node.accesskit_node();
            if let Some(name) = access.label().or_else(|| access.value()) {
                reached.insert(name);
            }
        }
    }
    let mut missing: Vec<_> = expected.difference(&reached).cloned().collect();
    missing.sort();
    missing
        .into_iter()
        .map(|name| format!("{scene}: \"{name}\" cannot be reached with Tab"))
        .collect()
}

/// Fails with every finding.
pub fn report(findings: Vec<String>) {
    assert!(
        findings.is_empty(),
        "{} usability finding(s):\n{}",
        findings.len(),
        findings.join("\n")
    );
}

/// Audits `view` drawn by `draw` at every supported size in both themes:
/// names, hit height, inside the window, no overlap, and Tab reaches every
/// enabled control except those starting with a `pointer_only` prefix.
pub fn audit_view_everywhere<V: Clone + 'static>(
    scene: &str,
    view: &V,
    draw: Draw<V>,
    pointer_only: &[&str],
) {
    let mut findings = Vec::new();
    for size in SIZES {
        for theme in THEMES {
            let scene = format!("{scene} at {}x{} {theme:?}", size.0, size.1);
            let mut harness = harness(size, theme, view.clone(), draw);
            findings.extend(audit_layout(&scene, &harness, size, false));
            findings.extend(audit_tab_order(&scene, &mut harness, size, pointer_only));
        }
    }
    report(findings);
}

/// Draws a view in a top row of exactly `height`, as the shell will.
pub fn in_top_row(ui: &mut egui::Ui, height: f32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Panel::top("row-under-test")
        .exact_size(height)
        .show(ui, add);
}

/// Draws a view in a side panel `width` wide, as the shell will.
pub fn in_side_panel(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Panel::left("panel-under-test")
        .exact_size(width)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, add);
        });
}
