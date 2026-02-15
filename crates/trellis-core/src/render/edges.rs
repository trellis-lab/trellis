use crate::grid::Grid;
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

/// Render a single routed edge as an SVG path element.
pub fn render_edge(
    edge: &Edge,
    grid_points: &[GridPoint],
    grid: &Grid,
    corner_radius: f64,
) -> String {
    if grid_points.len() < 2 {
        return String::new();
    }

    // Convert grid coordinates to world (pixel) coordinates
    let world_points: Vec<Point> = grid_points
        .iter()
        .map(|gp| {
            let (x, y) = grid.grid_to_world(gp.row as usize, gp.col as usize);
            Point { x, y }
        })
        .collect();

    // Simplify: remove collinear intermediate points
    let simplified = simplify_path(&world_points);

    // Generate SVG path with rounded corners
    let path_data = generate_rounded_polyline(&simplified, corner_radius);

    let stroke = stroke_attrs(edge.style);
    let marker = marker_attr(edge.arrow_head);

    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"#666\" {}{}/>",
        path_data, stroke, marker
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
