//! ER diagram edge-endpoint glyphs (crow's foot notation).
//!
//! Unlike the rest of the renderer, ER endpoint glyphs are **not** SVG
//! `<marker>` elements. They are emitted as plain shape primitives rotated
//! into place using the known orthogonal arrival direction.
//!
//! # Canonical frame
//!
//! Every glyph is designed as if the edge line arrives from `-X` (the left)
//! and the node box sits at `+X` (to the right). The endpoint is at the origin.
//!
//! ```text
//!     line ──────>│ box
//!        (−X)   (0,0)  (+X)
//! ```
//!
//! Convention (standard crow's foot): the **apex** of the fan faces the line
//! (`-X`), the **splay** opens toward the box (`+X`). Ticks and circles sit
//! between the two, aligned on the endpoint's perpendicular axis.
//!
//! A canonical point `(cx, cy)` is rotated into world space using a unit
//! vector that points *from the endpoint into the box*. See [`emit_glyph`].

use trellis_parser::ErCardinality;

/// Length (in pixels) reserved for an ER endpoint glyph along the line axis.
///
/// The routed path's last segment is shortened by this amount before drawing
/// so the line does not protrude through the glyph. A single conservative
/// value covers every cardinality — the widest glyph is `ZeroOrMore` which
/// fits comfortably within 14px.
pub const GLYPH_LENGTH: f64 = 14.0;

/// Half-width (perpendicular to the line) of the widest glyph.
const GLYPH_HALF_WIDTH: f64 = 7.0;

/// Stroke colour for ER glyphs — matches the entity box stroke.
const STROKE: &str = "#336699";
/// Stroke width for ER glyphs.
const STROKE_WIDTH: f64 = 1.5;

/// Which end of an edge a glyph is being drawn for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphEnd {
    /// The `from` (source) end of the edge.
    Start,
    /// The `to` (target) end of the edge.
    End,
}

/// Render an ER cardinality glyph at a given endpoint.
///
/// # Arguments
/// * `card` — the cardinality to draw.
/// * `anchor` — the endpoint position in world coordinates (after path
///   trimming, this is where the drawn line ends).
/// * `into_box` — a unit vector pointing *from the anchor into the node box*.
///   For an orthogonal routed path this is always axis-aligned.
/// * `_which_end` — which edge end this is (reserved for future asymmetric
///   glyphs; currently unused because the canonical frame is symmetric by
///   construction).
///
/// Returns an SVG fragment (without a surrounding `<g>`).
pub fn render_glyph(
    card: ErCardinality,
    anchor: (f64, f64),
    into_box: (f64, f64),
    _which_end: GlyphEnd,
) -> String {
    match card {
        ErCardinality::ExactlyOne => render_exactly_one(anchor, into_box),
        ErCardinality::ZeroOrOne => render_zero_or_one(anchor, into_box),
        ErCardinality::OneOrMore => render_one_or_more(anchor, into_box),
        ErCardinality::ZeroOrMore => render_zero_or_more(anchor, into_box),
    }
}

/// Transform a canonical point `(cx, cy)` to world coordinates.
///
/// Canonical frame: `+X` points into the box, `+Y` points "up" along the
/// perpendicular axis. The returned world point is `anchor + cx * into_box +
/// cy * perp`, where `perp` is `into_box` rotated 90° counter-clockwise.
fn transform(anchor: (f64, f64), into_box: (f64, f64), cx: f64, cy: f64) -> (f64, f64) {
    // Perpendicular (CCW rotation by 90°): (x,y) -> (-y, x)
    let perp = (-into_box.1, into_box.0);
    (
        anchor.0 + cx * into_box.0 + cy * perp.0,
        anchor.1 + cx * into_box.1 + cy * perp.1,
    )
}

/// Emit an SVG `<line>` between two canonical points.
fn line(anchor: (f64, f64), into_box: (f64, f64), c1: (f64, f64), c2: (f64, f64)) -> String {
    let (x1, y1) = transform(anchor, into_box, c1.0, c1.1);
    let (x2, y2) = transform(anchor, into_box, c2.0, c2.1);
    format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"{}\" stroke-width=\"{}\"/>",
        x1, y1, x2, y2, STROKE, STROKE_WIDTH
    )
}

/// Emit an SVG `<circle>` at a canonical point.
fn circle(anchor: (f64, f64), into_box: (f64, f64), c: (f64, f64), r: f64) -> String {
    let (cx, cy) = transform(anchor, into_box, c.0, c.1);
    format!(
        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"white\" \
         stroke=\"{}\" stroke-width=\"{}\"/>",
        cx, cy, r, STROKE, STROKE_WIDTH
    )
}

// ---------------------------------------------------------------------------
// Cardinality drawings (canonical frame: +X into box, origin at endpoint)
// ---------------------------------------------------------------------------

/// `||` — two parallel vertical ticks between the endpoint and the box.
///
/// Canonical:
/// ```text
///   line ──>│ │  box
///          4 8
/// ```
fn render_exactly_one(anchor: (f64, f64), into_box: (f64, f64)) -> String {
    let h = GLYPH_HALF_WIDTH;
    let mut out = String::new();
    out.push_str(&line(anchor, into_box, (4.0, -h), (4.0, h)));
    out.push_str(&line(anchor, into_box, (8.0, -h), (8.0, h)));
    out
}

/// `|o` — circle (zero) on the line side, tick (one) on the box side.
///
/// Canonical:
/// ```text
///   line ──> O │  box
///            4 10
/// ```
fn render_zero_or_one(anchor: (f64, f64), into_box: (f64, f64)) -> String {
    let h = GLYPH_HALF_WIDTH;
    let mut out = String::new();
    out.push_str(&circle(anchor, into_box, (4.0, 0.0), 3.5));
    out.push_str(&line(anchor, into_box, (10.0, -h), (10.0, h)));
    out
}

/// `|{` — tick + crow's-foot fan. Apex of the fan faces the line; the three
/// tines splay toward the box.
///
/// Canonical:
/// ```text
///                ── 12
///           apex ╲
///   line ──>  .────── 12  │  box
///           apex ╱
///                ── 12
///            2      12
/// ```
fn render_one_or_more(anchor: (f64, f64), into_box: (f64, f64)) -> String {
    let h = GLYPH_HALF_WIDTH;
    let apex = (2.0, 0.0);
    let mut out = String::new();
    // Tick (the "one" bar) — sits just past the fan, between foot and box
    out.push_str(&line(anchor, into_box, (12.0, -h), (12.0, h)));
    // Three fan tines radiating from apex toward the box side
    out.push_str(&line(anchor, into_box, apex, (12.0, -h)));
    out.push_str(&line(anchor, into_box, apex, (12.0, 0.0)));
    out.push_str(&line(anchor, into_box, apex, (12.0, h)));
    out
}

/// `o{` — circle + crow's-foot fan (zero-or-many). Circle sits on the line
/// side of the apex.
fn render_zero_or_more(anchor: (f64, f64), into_box: (f64, f64)) -> String {
    let h = GLYPH_HALF_WIDTH;
    // Shift the whole foot box-ward to leave room for the circle on the line side
    let apex = (6.0, 0.0);
    let mut out = String::new();
    out.push_str(&circle(anchor, into_box, (2.5, 0.0), 2.5));
    out.push_str(&line(anchor, into_box, apex, (13.0, -h)));
    out.push_str(&line(anchor, into_box, apex, (13.0, 0.0)));
    out.push_str(&line(anchor, into_box, apex, (13.0, h)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract all `x1` coordinates from a line-only glyph SVG.
    fn line_x1s(svg: &str) -> Vec<f64> {
        let mut out = Vec::new();
        for chunk in svg.split("x1=\"") {
            if let Some(end) = chunk.find('"') {
                if let Ok(v) = chunk[..end].parse::<f64>() {
                    out.push(v);
                }
            }
        }
        out
    }

    #[test]
    fn exactly_one_end_pointing_down() {
        // Endpoint at (100,200), path arriving from above, box below:
        // into_box = (0, +1). Canonical (4, -h) should map to
        // (100 + 0 - (-1) * -h, 200 + 1*4 + 0*-h) = (100 - h, 204)
        // Wait — perp = (-into_box.1, into_box.0) = (-1, 0)
        // (cx,cy)=(4,-h): world = (100 + 4*0 + (-h)*-1, 200 + 4*1 + (-h)*0)
        //               = (100 + h, 204)
        let svg = render_glyph(
            ErCardinality::ExactlyOne,
            (100.0, 200.0),
            (0.0, 1.0),
            GlyphEnd::End,
        );
        // Expect two vertical-ish ticks at world y=204 and y=208 (cx=4 and cx=8)
        assert!(svg.contains("y1=\"204.0\""));
        assert!(svg.contains("y1=\"208.0\""));
    }

    #[test]
    fn one_or_more_apex_faces_line() {
        // into_box = (+1, 0) — box is to the right, line comes from the left.
        // Apex canonical x=2, so world x = 100 + 2 = 102.
        // Fan tine ends canonical x=12, so world x = 100 + 12 = 112.
        // Apex (smaller x) must be closer to the line side (x=100).
        let svg = render_glyph(
            ErCardinality::OneOrMore,
            (100.0, 50.0),
            (1.0, 0.0),
            GlyphEnd::End,
        );
        let xs = line_x1s(&svg);
        // Fan tines all start at apex x=102; tick line has x1=112.
        assert!(xs.iter().any(|x| (*x - 102.0).abs() < 0.01));
        assert!(xs.iter().any(|x| (*x - 112.0).abs() < 0.01));
        // No x1 closer to the line side than the apex — i.e. nothing at x < 102.
        assert!(xs.iter().all(|x| *x >= 101.9));
    }

    #[test]
    fn zero_or_more_circle_on_line_side() {
        // into_box = (+1, 0), anchor (0,0). Circle canonical cx=2.5 → world (2.5, 0).
        // Fan apex canonical x=6 → world (6, 0). Fan tines at x=13.
        // Circle x must be less than apex x (closer to the line).
        let svg = render_glyph(
            ErCardinality::ZeroOrMore,
            (0.0, 0.0),
            (1.0, 0.0),
            GlyphEnd::End,
        );
        assert!(svg.contains("<circle"));
        assert!(svg.contains("cx=\"2.5\""));
    }

    #[test]
    fn zero_or_one_circle_on_line_side() {
        let svg = render_glyph(
            ErCardinality::ZeroOrOne,
            (0.0, 0.0),
            (1.0, 0.0),
            GlyphEnd::End,
        );
        assert!(svg.contains("<circle"));
        // Circle canonical cx=4, tick canonical x=10. Circle must be closer to line.
        assert!(svg.contains("cx=\"4.0\""));
    }

    #[test]
    fn rotation_by_side_upward_path() {
        // Path arriving from below, going up into a box on top.
        // into_box = (0, -1) — perp = (1, 0).
        // Canonical (4, -h) → world (100 + 4*0 + -h*1, 200 + 4*-1 + -h*0)
        //                   = (100 - h, 196)
        let svg = render_glyph(
            ErCardinality::ExactlyOne,
            (100.0, 200.0),
            (0.0, -1.0),
            GlyphEnd::End,
        );
        // Tick 1 (cx=4): world y = 200 - 4 = 196
        // Tick 2 (cx=8): world y = 200 - 8 = 192
        assert!(svg.contains("y1=\"196.0\""));
        assert!(svg.contains("y1=\"192.0\""));
    }
}
