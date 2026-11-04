//! UI-01: design tokens meet contrast targets and the theme switches live.

use diagram_desktop::shell::{self, ShellState};
use diagram_desktop::ui::theme::{self, DARK, LIGHT, Palette};
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Color32, Theme, ThemePreference, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn harness() -> Harness<'static, ShellState> {
    Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, ShellState::default())
}

fn failing_pairs(palette: &Palette) -> Vec<String> {
    theme::contrast_pairs(palette)
        .into_iter()
        .filter(|pair| pair.ratio() < pair.minimum)
        .map(|pair| format!("{}: {:.2} < {}", pair.name, pair.ratio(), pair.minimum))
        .collect()
}

#[test]
fn contrast_ratio_should_match_the_wcag_extremes() {
    let ratio = theme::contrast_ratio(Color32::BLACK, Color32::WHITE);

    assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
    assert!((theme::contrast_ratio(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.01);
}

#[test]
fn light_tokens_should_meet_every_contrast_target() {
    assert_eq!(failing_pairs(&LIGHT), Vec::<String>::new());
}

#[test]
fn dark_tokens_should_meet_every_contrast_target() {
    assert_eq!(failing_pairs(&DARK), Vec::<String>::new());
}

#[test]
fn control_border_should_be_stronger_than_the_decorative_separator() {
    for palette in [&LIGHT, &DARK] {
        let background = palette.panel;
        assert!(theme::contrast_ratio(palette.separator, background) < theme::NON_TEXT_CONTRAST);
        assert!(
            theme::contrast_ratio(palette.control_border, background) >= theme::NON_TEXT_CONTRAST
        );
    }
}

#[test]
fn styles_should_use_the_tokens() {
    for (theme_choice, palette) in [(Theme::Light, &LIGHT), (Theme::Dark, &DARK)] {
        let style = theme::style(theme_choice);

        assert_eq!(style.visuals.panel_fill, palette.panel);
        assert_eq!(
            style.visuals.widgets.inactive.bg_stroke.color,
            palette.control_border
        );
        assert_eq!(style.visuals.selection.bg_fill, palette.selection_fill);
        assert_eq!(style.visuals.warn_fg_color, palette.warning);
        assert_eq!(style.visuals.dark_mode, theme_choice == Theme::Dark);
        assert_eq!(style.spacing.interact_size.y, theme::CONTROL_HEIGHT);
    }
}

#[test]
fn shell_should_start_in_the_light_theme() {
    let harness = harness();

    assert_eq!(harness.ctx.theme(), Theme::Light);
    assert_eq!(harness.ctx.global_style().visuals.panel_fill, LIGHT.panel);
    assert_eq!(
        harness.get_by_label("Light").accesskit_node().toggled(),
        Some(Toggled::True)
    );
}

#[test]
fn theme_choice_should_switch_to_dark_and_back_without_restart() {
    let mut harness = harness();

    harness.get_by_label("Dark").click();
    harness.run();

    assert_eq!(harness.state().theme, ThemePreference::Dark);
    assert_eq!(harness.ctx.theme(), Theme::Dark);
    assert_eq!(harness.ctx.global_style().visuals.panel_fill, DARK.panel);

    harness.get_by_label("Light").click();
    harness.run();

    assert_eq!(harness.ctx.theme(), Theme::Light);
    assert_eq!(harness.ctx.global_style().visuals.panel_fill, LIGHT.panel);
}

#[test]
fn system_choice_should_follow_the_operating_system() {
    let mut harness = harness();

    harness.get_by_label("System").click();
    harness.run();

    assert_eq!(harness.state().theme, ThemePreference::System);
    assert_eq!(
        harness.ctx.options(|options| options.theme_preference),
        ThemePreference::System
    );
}
