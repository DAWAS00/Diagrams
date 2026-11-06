//! B1 / UI-07: what is painted directly on the canvas surface (the
//! relationship line and its multiplicities) stays readable in both themes,
//! selected or not.

mod common;
use common::{LARGE, harness_with};
use diagram_desktop::shell::ShellState;
use diagram_desktop::ui::read_model;
use diagram_desktop::ui::theme::{self, DARK, LIGHT, Palette};
use eframe::egui::epaint::{ClippedShape, Shape, TextShape};
use eframe::egui::{Color32, Pos2, Rect, ThemePreference};

/// How far a connector end may be from its multiplicity label.
const NEAR_LABEL: f32 = 16.0;

/// The document ink the light canvas used before B1; the light theme must
/// look the same after it.
const LIGHT_INK: Color32 = Color32::from_rgb(0x22, 0x31, 0x47);

/// Every shape in a frame, with nested lists opened.
fn flatten(shapes: &[ClippedShape]) -> Vec<Shape> {
    fn walk(shape: &Shape, out: &mut Vec<Shape>) {
        match shape {
            Shape::Vec(inner) => inner.iter().for_each(|shape| walk(shape, out)),
            other => out.push(other.clone()),
        }
    }
    let mut out = Vec::new();
    for clipped in shapes {
        walk(&clipped.shape, &mut out);
    }
    out
}

fn text_rect(text: &TextShape) -> Rect {
    text.galley.rect.translate(text.pos.to_vec2())
}

/// What the sample relationship looks like in one painted frame.
struct Connector {
    /// Colours of the `1` and `*` multiplicity labels.
    labels: Vec<Color32>,
    /// Colour of the line segment whose two ends sit next to those labels.
    line: Option<Color32>,
}

fn painted(preference: ThemePreference, selected: bool) -> Connector {
    let mut state = ShellState::default();
    state.theme = preference;
    if selected {
        let relation = read_model::relation_rows(state.document())
            .first()
            .map(|row| row.id);
        assert!(relation.is_some(), "the sample has a relationship");
        if let Some(relation) = relation {
            assert!(state.select_relation(relation));
        }
    }
    let harness = harness_with(LARGE, state);
    let shapes = flatten(&harness.output().shapes);

    // "1" and "*" also appear elsewhere (for example in the inspector's
    // fields), so the connector is the line whose two ends sit next to one
    // "1" label and one "*" label.
    let labels = |wanted: &str| -> Vec<(Color32, Rect)> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) if text.galley.text() == wanted => {
                    Some((text.fallback_color, text_rect(text)))
                }
                _ => None,
            })
            .collect()
    };
    let near = |rect: Rect, point: Pos2| rect.distance_to_pos(point) <= NEAR_LABEL;
    for source in labels("1") {
        for target in labels("*") {
            let line = shapes.iter().find_map(|shape| match shape {
                Shape::LineSegment {
                    points: [a, b],
                    stroke,
                } if (near(source.1, *a) && near(target.1, *b))
                    || (near(source.1, *b) && near(target.1, *a)) =>
                {
                    Some(stroke.color)
                }
                _ => None,
            });
            if line.is_some() {
                return Connector {
                    labels: vec![source.0, target.0],
                    line,
                };
            }
        }
    }
    Connector {
        labels: Vec::new(),
        line: None,
    }
}

fn check(preference: ThemePreference, palette: &Palette, selected: bool) -> Color32 {
    let connector = painted(preference, selected);
    let case = format!("{preference:?}, selected: {selected}");

    assert_eq!(
        connector.labels.len(),
        2,
        "{case}: both multiplicities are painted"
    );
    for color in &connector.labels {
        let ratio = theme::contrast_ratio(*color, palette.canvas);
        assert!(
            ratio >= theme::TEXT_CONTRAST,
            "{case}: multiplicity {color:?} on canvas {:?} is {ratio:.2}:1",
            palette.canvas
        );
    }
    let Some(line) = connector.line else {
        assert!(
            connector.line.is_some(),
            "{case}: no connector between the labels"
        );
        return Color32::TRANSPARENT;
    };
    let ratio = theme::contrast_ratio(line, palette.canvas);
    assert!(
        ratio >= theme::NON_TEXT_CONTRAST,
        "{case}: connector {line:?} on canvas is {ratio:.2}:1"
    );
    assert_eq!(
        connector.labels[0], line,
        "{case}: line and labels share one ink"
    );
    line
}

#[test]
fn relationships_should_be_readable_on_the_dark_canvas() {
    check(ThemePreference::Dark, &DARK, false);
    check(ThemePreference::Dark, &DARK, true);
}

#[test]
fn relationships_should_be_readable_on_the_light_canvas_and_look_as_before() {
    let ink = check(ThemePreference::Light, &LIGHT, false);
    assert_eq!(ink, LIGHT_INK, "the light theme keeps its document ink");
    check(ThemePreference::Light, &LIGHT, true);
}
