use std::collections::HashSet;

use crate::config::CrossingStyle;
use crate::grid::Grid;
use crate::render::c4_shapes::c4_edge_markers;
use crate::render::class_shapes::class_edge_markers;
use crate::render::er_glyphs::{render_glyph, GlyphEnd, GLYPH_LENGTH};
use crate::render::segments::{build_edge_segments, segments_to_svg_path};
use crate::routing::astar::GridPoint;
use trellis_parser::{ArrowHead, Edge, EdgeStyle};

/// A 2D point in world (pixel) coordinates.
#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

/// Return a direction tuple (dx_sign, dy_sign) where each is -1, 0, or 1.
/// Uses an epsilon to avoid float noise.
fn direction_sign(from: &Point, to: &Point) -> (i32, i32) {
    let eps = 0.001;
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let sx = if dx.abs() < eps {
        0
    } else if dx > 0.0 {
        1
    } else {
        -1
    };
    let sy = if dy.abs() < eps {
        0
    } else if dy > 0.0 {
        1
    } else {
        -1
    };
    (sx, sy)
}

/// Simplify a path by removing collinear points.
/// Only keeps bend points (where direction changes) plus start and end.
fn simplify_path(points: &[Point]) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut simplified = vec![points[0]];

    for i in 1..points.len() - 1 {
        let prev = &points[i - 1];
        let curr = &points[i];
        let next = &points[i + 1];

        // Check if direction changes using a small epsilon for float comparison
        let dir1 = direction_sign(prev, curr);
        let dir2 = direction_sign(curr, next);

        if dir1 != dir2 {
            simplified.push(*curr);
        }
    }

    simplified.push(*points.last().unwrap());
    simplified
}

/// Generate an SVG path data string with rounded corners (quadratic Bezier).
fn generate_rounded_polyline(points: &[Point], corner_radius: f64) -> String {
    if points.is_empty() {
        return String::new();
    }
    if points.len() == 1 {
        return format!("M {:.1} {:.1}", points[0].x, points[0].y);
    }
    if points.len() == 2 {
        return format!(
            "M {:.1} {:.1} L {:.1} {:.1}",
            points[0].x, points[0].y, points[1].x, points[1].y
        );
    }

    let mut d = format!("M {:.1} {:.1}", points[0].x, points[0].y);

    for i in 1..points.len() - 1 {
        let prev = &points[i - 1];
        let curr = &points[i];
        let next = &points[i + 1];

        let dist_prev = distance(prev, curr);
        let dist_next = distance(curr, next);
        let r = corner_radius.min(dist_prev / 2.0).min(dist_next / 2.0);

        if r < 0.5 {
            // Too small for rounding, just line to the point
            d.push_str(&format!(" L {:.1} {:.1}", curr.x, curr.y));
            continue;
        }

        // Point before the corner
        let before = move_towards(curr, prev, r);
        // Point after the corner
        let after = move_towards(curr, next, r);

        d.push_str(&format!(" L {:.1} {:.1}", before.x, before.y));
        d.push_str(&format!(
            " Q {:.1} {:.1} {:.1} {:.1}",
            curr.x, curr.y, after.x, after.y
        ));
    }

    let last = points.last().unwrap();
    d.push_str(&format!(" L {:.1} {:.1}", last.x, last.y));

    d
}

/// Calculate distance between two points.
fn distance(a: &Point, b: &Point) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// Move from `from` towards `to` by `dist` pixels.
fn move_towards(from: &Point, to: &Point, dist: f64) -> Point {
    let d = distance(from, to);
    if d < 0.001 {
        return *from;
    }
    let ratio = dist / d;
    Point {
        x: from.x + (to.x - from.x) * ratio,
        y: from.y + (to.y - from.y) * ratio,
    }
}

/// Determine the SVG stroke style attributes for an edge.
fn stroke_attrs(style: EdgeStyle) -> String {
    match style {
        EdgeStyle::Solid => "stroke-width=\"1.5\"".to_string(),
        EdgeStyle::Dotted => "stroke-width=\"1.5\" stroke-dasharray=\"5,5\"".to_string(),
        EdgeStyle::Thick => "stroke-width=\"3\"".to_string(),
    }
}

/// Choose a marker ID based on the arrow head type.
fn marker_attr(arrow_head: ArrowHead) -> &'static str {
    match arrow_head {
        ArrowHead::Arrow => " marker-end=\"url(#arrowhead)\"",
        ArrowHead::None => "",
    }
}

/// Unit vector pointing from `from` toward `to`. Returns `(0,0)` for coincident points.
fn unit_vector(from: &Point, to: &Point) -> (f64, f64) {
    let d = distance(from, to);
    if d < 0.001 {
        return (0.0, 0.0);
    }
    ((to.x - from.x) / d, (to.y - from.y) / d)
}

/// Shorten a polyline from the start and/or end by `trim_start` / `trim_end` pixels.
///
/// Trimming walks inward segment-by-segment, consuming full segments when the
/// trim distance exceeds the segment length. The resulting polyline always
/// preserves the interior bends of the original.
fn trim_polyline(points: &[Point], trim_start: f64, trim_end: f64) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let mut pts = points.to_vec();

    // Trim from the start
    let mut remaining = trim_start;
    while remaining > 0.0 && pts.len() >= 2 {
        let seg_len = distance(&pts[0], &pts[1]);
        if seg_len > remaining + 0.001 {
            pts[0] = move_towards(&pts[0], &pts[1], remaining);
            break;
        }
        remaining -= seg_len;
        pts.remove(0);
    }

    // Trim from the end
    let mut remaining = trim_end;
    while remaining > 0.0 && pts.len() >= 2 {
        let last = pts.len() - 1;
        let seg_len = distance(&pts[last], &pts[last - 1]);
        if seg_len > remaining + 0.001 {
            pts[last] = move_towards(&pts[last], &pts[last - 1], remaining);
            break;
        }
        remaining -= seg_len;
        pts.pop();
    }

    pts
}

/// Render ER cardinality glyphs at either end of an edge and return
/// `(svg_fragment, trim_start, trim_end)`.
///
/// `trim_start` / `trim_end` are the amounts the path line should be shortened
/// on each end so the stroke does not protrude through the glyph.
fn render_er_glyphs(edge: &Edge, points: &[Point]) -> (String, f64, f64) {
    if points.len() < 2 {
        return (String::new(), 0.0, 0.0);
    }

    let mut svg = String::new();
    let mut trim_start = 0.0;
    let mut trim_end = 0.0;

    if let Some(card) = edge.er_source_card {
        // At the start, the "into box" direction is opposite the outgoing tangent.
        let tangent = unit_vector(&points[0], &points[1]);
        let into_box = (-tangent.0, -tangent.1);
        let anchor_world = (points[0].x, points[0].y);
        // The glyph occupies GLYPH_LENGTH pixels along the line. We draw the
        // glyph with its canonical origin at the path's first drawn point —
        // i.e., after trimming — so the anchor is GLYPH_LENGTH into the path.
        let anchor = (
            anchor_world.0 + tangent.0 * GLYPH_LENGTH,
            anchor_world.1 + tangent.1 * GLYPH_LENGTH,
        );
        svg.push_str(&render_glyph(card, anchor, into_box, GlyphEnd::Start));
        trim_start = GLYPH_LENGTH;
    }

    if let Some(card) = edge.er_target_card {
        let last = points.len() - 1;
        let tangent = unit_vector(&points[last - 1], &points[last]);
        let into_box = tangent;
        let anchor_world = (points[last].x, points[last].y);
        let anchor = (
            anchor_world.0 - tangent.0 * GLYPH_LENGTH,
            anchor_world.1 - tangent.1 * GLYPH_LENGTH,
        );
        svg.push_str(&render_glyph(card, anchor, into_box, GlyphEnd::End));
        trim_end = GLYPH_LENGTH;
    }

    (svg, trim_start, trim_end)
}

/// Render a single routed edge as an SVG path element.
///
/// `crossing_set` contains the grid coordinates where this edge crosses another.
/// `crossing_style` controls decoration at those cells; use `CrossingStyle::None`
/// to pass crossings straight through without any special rendering.
pub fn render_edge(
    edge: &Edge,
    grid_points: &[GridPoint],
    grid: &Grid,
    corner_radius: f64,
    crossing_set: &HashSet<(i64, i64)>,
    crossing_style: CrossingStyle,
) -> String {
    if grid_points.len() < 2 {
        return String::new();
    }

    let stroke = stroke_attrs(edge.style);

    let is_er = edge.er_source_card.is_some() || edge.er_target_card.is_some();

    // ER edges use a separate trimming pipeline for crow's-foot glyphs.
    // Hop arc rendering is not yet applied to ER edges (Phase 4 polish).
    if is_er {
        let world_points: Vec<Point> = grid_points
            .iter()
            .map(|gp| {
                let (x, y) = grid.grid_to_world(gp.row as usize, gp.col as usize);
                Point { x, y }
            })
            .collect();
        let simplified = simplify_path(&world_points);
        let (glyph_svg, trim_start, trim_end) = render_er_glyphs(edge, &simplified);
        let trimmed = trim_polyline(&simplified, trim_start, trim_end);
        let path_data = generate_rounded_polyline(&trimmed, corner_radius);
        return format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"#555\" {}/>{}",
            path_data, stroke, glyph_svg
        );
    }

    // Standard edges: use the segment pipeline which integrates crossing decorations.
    let segments = build_edge_segments(grid_points, grid, crossing_set, corner_radius, crossing_style);
    let path_data = segments_to_svg_path(&segments);

    let (marker_start, marker_end) = if edge.class_edge_type.is_some() {
        class_edge_markers(edge)
    } else if edge.c4_rel_type.is_some() {
        c4_edge_markers(edge)
    } else {
        (String::new(), marker_attr(edge.arrow_head).to_string())
    };

    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"#555\" {}{}{}/>",
        path_data, stroke, marker_start, marker_end
    )
}


/// Render a fallback straight-line edge when A* routing failed.
pub fn render_fallback_edge(edge: &Edge, from_x: f64, from_y: f64, to_x: f64, to_y: f64) -> String {
    let stroke = stroke_attrs(edge.style);
    let marker = marker_attr(edge.arrow_head);
    format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"#ccc\" {} stroke-dasharray=\"3,3\"{}/>",
        from_x, from_y, to_x, to_y, stroke, marker
    )
}

/// Generate the SVG `<defs>` block containing arrow marker definitions.
pub fn arrow_marker_defs() -> &'static str {
    "<defs>\
     <marker id=\"arrowhead\" viewBox=\"0 0 10 10\" refX=\"10\" refY=\"5\" \
     markerWidth=\"4\" markerHeight=\"4\" orient=\"auto-start-reverse\">\
     <path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#666\"/>\
     </marker>\
     </defs>"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_collinear() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 10.0 },
            Point { x: 0.0, y: 20.0 },
            Point { x: 0.0, y: 30.0 },
        ];
        let simplified = simplify_path(&points);
        assert_eq!(simplified.len(), 2); // only start and end
    }

    #[test]
    fn test_simplify_with_bend() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 10.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 10.0, y: 20.0 },
        ];
        let simplified = simplify_path(&points);
        assert_eq!(simplified.len(), 4); // all points are significant
    }

    #[test]
    fn test_simplify_preserves_bends() {
        // L-shape: down then right
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 5.0 },
            Point { x: 0.0, y: 10.0 }, // collinear, should be removed
            Point { x: 5.0, y: 10.0 }, // bend here, keep (0,10)
            Point { x: 10.0, y: 10.0 },
        ];
        let simplified = simplify_path(&points);
        // Start, bend at (0,10), end
        assert_eq!(simplified.len(), 3);
    }

    #[test]
    fn test_rounded_polyline_straight() {
        let points = vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }];
        let path = generate_rounded_polyline(&points, 5.0);
        assert!(path.starts_with("M 0.0 0.0"));
        assert!(path.contains("L 100.0 0.0"));
        assert!(!path.contains("Q")); // no curves for straight line
    }

    #[test]
    fn test_rounded_polyline_with_corner() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 50.0 },
            Point { x: 50.0, y: 50.0 },
        ];
        let path = generate_rounded_polyline(&points, 5.0);
        assert!(path.contains("Q")); // should have a quadratic bezier curve
    }

    #[test]
    fn test_distance() {
        let a = Point { x: 0.0, y: 0.0 };
        let b = Point { x: 3.0, y: 4.0 };
        assert!((distance(&a, &b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_stroke_attrs() {
        assert!(!stroke_attrs(EdgeStyle::Solid).contains("dasharray"));
        assert!(stroke_attrs(EdgeStyle::Dotted).contains("dasharray"));
        assert!(stroke_attrs(EdgeStyle::Thick).contains("stroke-width=\"3\""));
    }
}
