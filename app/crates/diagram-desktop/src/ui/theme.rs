//! Design tokens, text roles and their mapping to egui styles (UI-01, UI-02).
//!
//! Every chrome color comes from a [`Palette`]; widgets never use raw RGB
//! values. Diagram objects on the canvas keep their own document colors so a
//! theme change never recolors saved diagrams or exported files.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Stroke, Style, TextStyle, Theme, ThemePreference, Vec2,
};

/// Minimum contrast for normal text (WCAG 2.2, 1.4.3).
pub const TEXT_CONTRAST: f32 = 4.5;
/// Minimum contrast for essential control boundaries and focus indicators (WCAG 2.2, 1.4.11).
pub const NON_TEXT_CONTRAST: f32 = 3.0;

/// Opacity egui applies to disabled widgets (set explicitly in [`style`]).
pub const DISABLED_ALPHA: f32 = 0.5;

/// Height of buttons and single-line fields, in logical points.
pub const CONTROL_HEIGHT: f32 = 34.0;
/// Minimum height of clickable list rows (plan: hit targets of at least 32).
pub const ROW_HEIGHT: f32 = 32.0;
/// Corner radius of controls.
pub const CONTROL_RADIUS: u8 = 6;
/// Corner radius of dialogs and windows.
pub const DIALOG_RADIUS: u8 = 8;
/// Base spacing unit.
pub const SPACING: f32 = 8.0;
/// Gap inside composite controls.
pub const GAP: f32 = 4.0;

/// Name of the egui text style used for [`TextRole::PageTitle`].
pub const PAGE_TITLE_STYLE: &str = "page-title";

/// Text roles from the UI plan. Every text size in the shell comes from one
/// of these, through egui text styles, so sizes are changed in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextRole {
    /// Page titles such as the welcome screen heading (24).
    PageTitle,
    /// Panel and section titles (16).
    PanelTitle,
    /// Body text, messages and list content (15).
    Body,
    /// Field labels, button and status text (14).
    Label,
    /// Member lines, types and other code-like text (14, monospace).
    Code,
}

impl TextRole {
    /// Every role, from largest to smallest.
    pub const ALL: [Self; 5] = [
        Self::PageTitle,
        Self::PanelTitle,
        Self::Body,
        Self::Label,
        Self::Code,
    ];

    /// Size in logical points.
    pub const fn size(self) -> f32 {
        match self {
            Self::PageTitle => 24.0,
            Self::PanelTitle => 16.0,
            Self::Body => 15.0,
            Self::Label | Self::Code => 14.0,
        }
    }

    /// The egui text style that carries this role.
    pub fn text_style(self) -> TextStyle {
        match self {
            Self::PageTitle => TextStyle::Name(PAGE_TITLE_STYLE.into()),
            Self::PanelTitle => TextStyle::Heading,
            Self::Body => TextStyle::Body,
            Self::Label => TextStyle::Button,
            Self::Code => TextStyle::Monospace,
        }
    }

    /// Font for painting this role directly, for example on the canvas.
    pub fn font_id(self) -> FontId {
        match self {
            Self::Code => FontId::monospace(self.size()),
            _ => FontId::proportional(self.size()),
        }
    }
}

/// Semantic color tokens for one theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Application background behind panels (header, status bar, canvas surround).
    pub workspace: Color32,
    /// Panels, fields and dialogs.
    pub panel: Color32,
    /// Diagram canvas background.
    pub canvas: Color32,
    /// Primary text.
    pub text: Color32,
    /// Secondary text such as section labels and hints.
    pub text_secondary: Color32,
    /// Accent for links, focus and editor selection overlays.
    pub accent: Color32,
    /// Text drawn on an accent fill.
    pub on_accent: Color32,
    /// Decorative separators between panels. Not used for control boundaries.
    pub separator: Color32,
    /// Boundary of buttons, fields and other controls.
    pub control_border: Color32,
    /// Background of a hovered control.
    pub hover_fill: Color32,
    /// Background of selected rows, pressed controls and selected text.
    pub selection_fill: Color32,
    /// Text and focus stroke on [`Self::selection_fill`].
    pub selection_text: Color32,
    /// Warning text.
    pub warning: Color32,
    /// Error and danger text.
    pub danger: Color32,
}

const fn rgb(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

/// Light theme tokens (the default theme).
pub const LIGHT: Palette = Palette {
    workspace: rgb(0xF4F6FA),
    panel: rgb(0xFFFFFF),
    canvas: rgb(0xFCFCFD),
    text: rgb(0x182438),
    text_secondary: rgb(0x526078),
    accent: rgb(0x245CC5),
    on_accent: rgb(0xFFFFFF),
    separator: rgb(0xD9E0EB),
    control_border: rgb(0x76839A),
    hover_fill: rgb(0xEAEFF6),
    selection_fill: rgb(0xDCE7FA),
    selection_text: rgb(0x1E4FAD),
    warning: rgb(0x8A4B00),
    danger: rgb(0xB42318),
};

/// Dark theme tokens.
pub const DARK: Palette = Palette {
    workspace: rgb(0x141A24),
    panel: rgb(0x1D2633),
    canvas: rgb(0x202B39),
    text: rgb(0xEDF2F8),
    text_secondary: rgb(0xB6C2D3),
    accent: rgb(0x91B8FF),
    on_accent: rgb(0x141A24),
    separator: rgb(0x465469),
    control_border: rgb(0x6E7F98),
    hover_fill: rgb(0x273345),
    selection_fill: rgb(0x25395C),
    selection_text: rgb(0x91B8FF),
    warning: rgb(0xFFD08A),
    danger: rgb(0xFFB4AD),
};

/// Default fill of a class box in a new diagram. This is a *document* color:
/// it belongs to the diagram, does not follow the editor theme, and reaches
/// the board through the presentation's document style.
pub const DOCUMENT_DEFAULT_FILL: Color32 = rgb(0xFAFCFF);
/// Default line and text color of diagram objects (a document color).
pub const DOCUMENT_DEFAULT_INK: Color32 = rgb(0x223147);

/// Tokens for `theme`.
pub fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::Light => &LIGHT,
        Theme::Dark => &DARK,
    }
}

/// WCAG contrast ratio between two opaque colors, from 1.0 to 21.0.
pub fn contrast_ratio(first: Color32, second: Color32) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    fn luminance(color: Color32) -> f32 {
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }
    let (first, second) = (luminance(first), luminance(second));
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

/// One foreground/background combination the shell draws, with its required minimum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastPair {
    /// Human-readable description, used in test failures and evidence.
    pub name: &'static str,
    /// Foreground color.
    pub foreground: Color32,
    /// Background color.
    pub background: Color32,
    /// Required minimum ratio.
    pub minimum: f32,
}

impl ContrastPair {
    /// Measured ratio of this pair.
    pub fn ratio(&self) -> f32 {
        contrast_ratio(self.foreground, self.background)
    }
}

/// Every token pair the shell relies on. Decorative separators and disabled
/// controls are excluded, as WCAG exempts them.
pub fn contrast_pairs(palette: &Palette) -> Vec<ContrastPair> {
    let p = palette;
    let pair = |name, foreground, background, minimum| ContrastPair {
        name,
        foreground,
        background,
        minimum,
    };
    vec![
        pair("text on workspace", p.text, p.workspace, TEXT_CONTRAST),
        pair("text on panel", p.text, p.panel, TEXT_CONTRAST),
        pair("text on canvas", p.text, p.canvas, TEXT_CONTRAST),
        pair("text on hover", p.text, p.hover_fill, TEXT_CONTRAST),
        pair("text on selection", p.text, p.selection_fill, TEXT_CONTRAST),
        pair(
            "secondary text on workspace",
            p.text_secondary,
            p.workspace,
            TEXT_CONTRAST,
        ),
        pair(
            "secondary text on panel",
            p.text_secondary,
            p.panel,
            TEXT_CONTRAST,
        ),
        pair(
            "secondary text on hover",
            p.text_secondary,
            p.hover_fill,
            TEXT_CONTRAST,
        ),
        pair("accent on panel", p.accent, p.panel, TEXT_CONTRAST),
        pair("accent on workspace", p.accent, p.workspace, TEXT_CONTRAST),
        pair(
            "accent overlay on canvas",
            p.accent,
            p.canvas,
            NON_TEXT_CONTRAST,
        ),
        pair("text on accent", p.on_accent, p.accent, TEXT_CONTRAST),
        pair(
            "selected text",
            p.selection_text,
            p.selection_fill,
            TEXT_CONTRAST,
        ),
        pair(
            "focus ring on panel",
            p.selection_text,
            p.panel,
            NON_TEXT_CONTRAST,
        ),
        pair("warning on panel", p.warning, p.panel, TEXT_CONTRAST),
        pair(
            "warning on workspace",
            p.warning,
            p.workspace,
            TEXT_CONTRAST,
        ),
        pair("danger on panel", p.danger, p.panel, TEXT_CONTRAST),
        pair("danger on workspace", p.danger, p.workspace, TEXT_CONTRAST),
        pair(
            "control border on panel",
            p.control_border,
            p.panel,
            NON_TEXT_CONTRAST,
        ),
        pair(
            "control border on workspace",
            p.control_border,
            p.workspace,
            NON_TEXT_CONTRAST,
        ),
        pair(
            "control border on hover",
            p.control_border,
            p.hover_fill,
            NON_TEXT_CONTRAST,
        ),
    ]
}

/// `color` as it appears on `background` when a widget is disabled.
pub fn disabled_color(color: Color32, background: Color32) -> Color32 {
    let mix = |foreground: u8, back: u8| {
        let value =
            f32::from(foreground) * DISABLED_ALPHA + f32::from(back) * (1.0 - DISABLED_ALPHA);
        value.round().clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(
        mix(color.r(), background.r()),
        mix(color.g(), background.g()),
        mix(color.b(), background.b()),
    )
}

/// Contrast of interaction states: focus, error and disabled. WCAG exempts
/// disabled controls; this project still keeps disabled text at 3:1 so it
/// stays readable.
pub fn state_contrast_pairs(palette: &Palette) -> Vec<ContrastPair> {
    let p = palette;
    let pair = |name, foreground, background, minimum| ContrastPair {
        name,
        foreground,
        background,
        minimum,
    };
    vec![
        pair(
            "focus ring on panel",
            p.selection_text,
            p.panel,
            NON_TEXT_CONTRAST,
        ),
        pair(
            "focus ring on workspace",
            p.selection_text,
            p.workspace,
            NON_TEXT_CONTRAST,
        ),
        pair(
            "error border on panel",
            p.danger,
            p.panel,
            NON_TEXT_CONTRAST,
        ),
        pair("error text on panel", p.danger, p.panel, TEXT_CONTRAST),
        pair(
            "disabled text on a button",
            disabled_color(p.text, p.workspace),
            p.workspace,
            NON_TEXT_CONTRAST,
        ),
        pair(
            "disabled text on panel",
            disabled_color(p.text, p.panel),
            p.panel,
            NON_TEXT_CONTRAST,
        ),
    ]
}

/// Complete egui style for `theme`: colors, text roles, spacing and corner radii.
pub fn style(theme: Theme) -> Style {
    let p = palette(theme);
    let mut style = Style::default();
    let mut visuals = match theme {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    };

    visuals.override_text_color = None;
    visuals.weak_text_color = Some(p.text_secondary);
    visuals.hyperlink_color = p.accent;
    visuals.faint_bg_color = p.workspace;
    visuals.extreme_bg_color = p.panel;
    visuals.text_edit_bg_color = Some(p.panel);
    visuals.code_bg_color = p.workspace;
    visuals.warn_fg_color = p.warning;
    visuals.error_fg_color = p.danger;
    visuals.panel_fill = p.panel;
    visuals.window_fill = p.panel;
    visuals.window_stroke = Stroke::new(1.0, p.separator);
    visuals.window_corner_radius = CornerRadius::same(DIALOG_RADIUS);
    visuals.menu_corner_radius = CornerRadius::same(CONTROL_RADIUS);
    visuals.disabled_alpha = DISABLED_ALPHA;
    visuals.selection.bg_fill = p.selection_fill;
    visuals.selection.stroke = Stroke::new(1.5, p.selection_text);

    let radius = CornerRadius::same(CONTROL_RADIUS);
    let widgets = &mut visuals.widgets;
    for (state, fill, border, width) in [
        (&mut widgets.noninteractive, p.panel, p.separator, 1.0),
        (&mut widgets.inactive, p.workspace, p.control_border, 1.0),
        (&mut widgets.hovered, p.hover_fill, p.accent, 1.0),
        (&mut widgets.active, p.selection_fill, p.accent, 1.5),
        (&mut widgets.open, p.selection_fill, p.accent, 1.5),
    ] {
        state.weak_bg_fill = fill;
        state.bg_fill = if fill == p.workspace { p.panel } else { fill };
        state.bg_stroke = Stroke::new(width, border);
        state.fg_stroke = Stroke::new(1.0, p.text);
        state.corner_radius = radius;
    }

    style.visuals = visuals;
    style.text_styles = TextRole::ALL
        .into_iter()
        .map(|role| (role.text_style(), role.font_id()))
        .chain([(TextStyle::Small, FontId::proportional(12.0))])
        .collect();
    style.spacing.item_spacing = Vec2::new(SPACING, GAP);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    style.spacing.interact_size.y = CONTROL_HEIGHT;
    style
}

/// Installs the light and dark styles and selects `preference`. Safe to call
/// again: a changed preference takes effect without restarting.
pub fn apply(ctx: &egui::Context, preference: ThemePreference) {
    for theme in [Theme::Light, Theme::Dark] {
        ctx.set_style_of(theme, style(theme));
    }
    ctx.set_theme(preference);
}
