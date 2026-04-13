use std::collections::HashSet;

use crate::config::CrossingStyle;
use crate::grid::Grid;
use crate::routing::astar::GridPoint;

/// A 2D point in world (pixel) coordinates used during segment construction.
#[derive(Debug, Clone, Copy)]
pub struct RenderPoint {
    pub x: f64,
    pub y: f64,
}

impl RenderPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A typed segment in an edge path.
///
/// Segments form a connected sequence: each segment's implied end point is the
/// start of the next. The first segment must be `MoveTo`.
#[derive(Debug, Clone)]
pub enum EdgeSegment {
    /// Start of the path — emits `M x y`.
    MoveTo(RenderPoint),
    /// Straight line to point — emits `L x y`.
    LineTo(RenderPoint),
    /// Rounded bend: line to `entry`, then quadratic Bezier to `exit` through
    /// `control`. Emits `L entry_x entry_y Q ctrl_x ctrl_y exit_x exit_y`.
    Bend {
        entry: RenderPoint,
        control: RenderPoint,
        exit: RenderPoint,
    },
    /// Crossing hop arc with flat feet: `_͡_` shape.
    ///
    /// Path flows: `L entry  L foot_in  A r r 0 0 sweep foot_out  L exit`
    ///
    /// `entry`/`exit` are half a cell from the crossing center (same as the
    /// old arc endpoints).  `foot_in`/`foot_out` are `HOP_FOOT_LEN` pixels
    /// closer to the center, so the arc proper is smaller and the flat feet
    /// visually "ground" the bridge.
    Hop {
        entry: RenderPoint,
        foot_in: RenderPoint,
        foot_out: RenderPoint,
        exit: RenderPoint,
        radius: f64,
        sweep: u8,
    },
    /// Rectangular bridge: `_|‾|_` shape.
    ///
    /// Path flows: `L entry  L foot_in  L corner1  L corner2  L foot_out  L exit`
    ///
    /// `entry`/`exit` are half a cell from the crossing center.
    /// `foot_in`/`foot_out` are `HOP_FOOT_LEN` pixels closer to the center —
    /// the same flat feet as the arc, so the rectangle starts/ends with 2 px
    /// of straight line before rising.  `corner1`/`corner2` are the top corners
    /// of the bridge, offset perpendicularly by `height` from `foot_in`/`foot_out`.
    HopRect {
        entry: RenderPoint,
        foot_in: RenderPoint,
        corner1: RenderPoint,
        corner2: RenderPoint,
        foot_out: RenderPoint,
        exit: RenderPoint,
    },
    /// Gap/skip: `-| |-` shape — stroke breaks at the crossing.
    ///
    /// Path flows: `L entry  L foot_in  M foot_out  L exit`
    ///
    /// `entry`/`exit` are half a cell from the crossing center.
    /// `foot_in`/`foot_out` are `HOP_FOOT_LEN` pixels closer to the center —
    /// the same flat feet as Arc/Rectangular, so there are 2 px of visible stub
    /// on each side before the gap starts.
    HopSkip {
        entry: RenderPoint,
        foot_in: RenderPoint,
        foot_out: RenderPoint,
        exit: RenderPoint,
    },
}

/// Length of the flat foot segments on each side of a hop arc (pixels).
/// Total hop span = `cell_size`; arc radius = `cell_size/2 - HOP_FOOT_LEN`.
const HOP_FOOT_LEN: f64 = 2.0;

// ─── Geometry helpers ────────────────────────────────────────────────────────

fn distance(a: RenderPoint, b: RenderPoint) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

fn move_towards(from: RenderPoint, to: RenderPoint, dist: f64) -> RenderPoint {
    let d = distance(from, to);
    if d < 0.001 {
        return from;
    }
    let ratio = dist / d;
    RenderPoint {
        x: from.x + (to.x - from.x) * ratio,
        y: from.y + (to.y - from.y) * ratio,
    }
}

fn direction_sign(from: RenderPoint, to: RenderPoint) -> (i32, i32) {
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

/// Determine the SVG arc sweep flag for a hop at `curr`, coming from `prev`
/// and continuing to `next`.
///
/// Convention: arcs always bulge "upward-or-left" — away from the positive
/// screen-coordinate direction of travel. This produces a consistent bridge
/// look regardless of edge orientation.
fn hop_sweep(prev: RenderPoint, _curr: RenderPoint, next: RenderPoint) -> u8 {
    let dx = next.x - prev.x;
    let dy = next.y - prev.y;
    if dx.abs() >= dy.abs() {
        // Horizontal travel: going right (dx>0) arcs upward → sweep=0 (CCW in y-down SVG)
        //                    going left  (dx<0) arcs downward → sweep=1
        if dx >= 0.0 {
            0
        } else {
            1
        }
    } else {
        // Vertical travel: going down (dy>0) arcs rightward → sweep=1
        //                  going up   (dy<0) arcs leftward  → sweep=0
        if dy >= 0.0 {
            1
        } else {
            0
        }
    }
}

// ─── Path simplification ─────────────────────────────────────────────────────

/// Simplify a path keeping only direction-change points plus start and end.
/// Crossing points are always preserved even when collinear, so hop arcs are
/// never accidentally dropped by simplification.
fn simplify_path_with_grid(
    points: &[(RenderPoint, (i64, i64))],
    crossing_set: &HashSet<(i64, i64)>,
) -> Vec<(RenderPoint, (i64, i64))> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut simplified = vec![points[0]];

    for i in 1..points.len() - 1 {
        let (prev_pt, _) = points[i - 1];
        let (curr_pt, curr_gc) = points[i];
        let (next_pt, _) = points[i + 1];

        let dir1 = direction_sign(prev_pt, curr_pt);
        let dir2 = direction_sign(curr_pt, next_pt);

        // Keep if direction changes OR if this cell has a crossing.
        if dir1 != dir2 || crossing_set.contains(&curr_gc) {
            simplified.push((curr_pt, curr_gc));
        }
    }

    simplified.push(*points.last().unwrap());
    simplified
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Build a sequence of typed edge segments from grid-coordinate path points.
///
/// Converts grid coordinates to world coordinates, simplifies collinear points
/// (preserving crossing cells), then classifies each point as a `LineTo`,
/// `Bend`, or hop segment. The `crossing_set` contains the grid coordinates of
/// cells where this edge crosses another; those cells are decorated according to
/// `crossing_style`.  Pass `CrossingStyle::None` (or an empty set) for plain
/// line + bend segments.
pub fn build_edge_segments(
    grid_points: &[GridPoint],
    grid: &Grid,
    crossing_set: &HashSet<(i64, i64)>,
    corner_radius: f64,
    crossing_style: CrossingStyle,
) -> Vec<EdgeSegment> {
    if grid_points.is_empty() {
        return vec![];
    }

    // Convert to world coords, keeping the grid coords for crossing detection.
    let world_with_gc: Vec<(RenderPoint, (i64, i64))> = grid_points
        .iter()
        .map(|gp| {
            let (x, y) = grid.grid_to_world(gp.row as usize, gp.col as usize);
            (RenderPoint::new(x, y), (gp.row, gp.col))
        })
        .collect();

    if world_with_gc.len() == 1 {
        return vec![EdgeSegment::MoveTo(world_with_gc[0].0)];
    }

    // Simplify: remove collinear points (but keep crossings).
    let simplified = simplify_path_with_grid(&world_with_gc, crossing_set);

    if simplified.len() == 1 {
        return vec![EdgeSegment::MoveTo(simplified[0].0)];
    }

    let cell_size = grid.cell_size as f64;
    let n = simplified.len();
    let mut segments: Vec<EdgeSegment> = Vec::with_capacity(n + 4);

    segments.push(EdgeSegment::MoveTo(simplified[0].0));

    // Process each interior + final point.
    let mut i = 1usize;
    while i < n {
        let (curr_pt, curr_gc) = simplified[i];

        if i == n - 1 {
            // Final point — always a straight line to it.
            segments.push(EdgeSegment::LineTo(curr_pt));
            i += 1;
            continue;
        }

        // Interior point.
        let (prev_pt, _) = simplified[i - 1];
        let (next_pt, _) = simplified[i + 1];

        if crossing_set.contains(&curr_gc) && crossing_style != CrossingStyle::None {
            let half_cell = cell_size / 2.0;
            let entry = move_towards(curr_pt, prev_pt, half_cell);
            let exit_pt = move_towards(curr_pt, next_pt, half_cell);

            match crossing_style {
                CrossingStyle::Arc => {
                    // `_͡_` shape: flat feet + semicircular arc.
                    let foot_in = move_towards(entry, curr_pt, HOP_FOOT_LEN);
                    let foot_out = move_towards(exit_pt, curr_pt, HOP_FOOT_LEN);
                    let radius = (half_cell - HOP_FOOT_LEN).max(1.0);
                    let sweep = hop_sweep(prev_pt, curr_pt, next_pt);
                    segments.push(EdgeSegment::Hop {
                        entry,
                        foot_in,
                        foot_out,
                        exit: exit_pt,
                        radius,
                        sweep,
                    });
                }
                CrossingStyle::Rectangular => {
                    // `_|‾|_` shape: 2 px flat feet then a perpendicular square bump.
                    //
                    // foot_in/foot_out are HOP_FOOT_LEN pixels closer to the center
                    // than entry/exit — same flat-foot pattern as the arc.
                    // corner1/corner2 are offset perpendicularly from foot_in/foot_out.
                    //
                    // Perpendicular unit vector (same side as the arc bump):
                    //   for travel direction (ux, uy), perp = (uy, -ux) when
                    //   sweep==0 and (-uy, ux) when sweep==1.
                    let foot_in = move_towards(entry, curr_pt, HOP_FOOT_LEN);
                    let foot_out = move_towards(exit_pt, curr_pt, HOP_FOOT_LEN);
                    let sweep = hop_sweep(prev_pt, curr_pt, next_pt);
                    let travel_dx = next_pt.x - prev_pt.x;
                    let travel_dy = next_pt.y - prev_pt.y;
                    let len = (travel_dx * travel_dx + travel_dy * travel_dy)
                        .sqrt()
                        .max(0.001);
                    let (ux, uy) = (travel_dx / len, travel_dy / len);
                    let (px, py) = if sweep == 0 { (uy, -ux) } else { (-uy, ux) };
                    let height = (half_cell - HOP_FOOT_LEN).max(1.0);
                    let corner1 = RenderPoint {
                        x: foot_in.x + px * height,
                        y: foot_in.y + py * height,
                    };
                    let corner2 = RenderPoint {
                        x: foot_out.x + px * height,
                        y: foot_out.y + py * height,
                    };
                    segments.push(EdgeSegment::HopRect {
                        entry,
                        foot_in,
                        corner1,
                        corner2,
                        foot_out,
                        exit: exit_pt,
                    });
                }
                CrossingStyle::Skip => {
                    // `-| |-` shape: 2 px stub → gap → 2 px stub.
                    let foot_in = move_towards(entry, curr_pt, HOP_FOOT_LEN);
                    let foot_out = move_towards(exit_pt, curr_pt, HOP_FOOT_LEN);
                    segments.push(EdgeSegment::HopSkip {
                        entry,
                        foot_in,
                        foot_out,
                        exit: exit_pt,
                    });
                }
                CrossingStyle::None => unreachable!(),
            }
            // After any hop the SVG pen is at `exit_pt`. The next iteration
            // uses simplified[i] as "previous" for corner-radius maths —
            // slight approximation for back-to-back hops but visually negligible.
        } else {
            // Bend or straight segment.
            let dist_prev = distance(prev_pt, curr_pt);
            let dist_next = distance(curr_pt, next_pt);
            let r = corner_radius.min(dist_prev / 2.0).min(dist_next / 2.0);

            if r < 0.5 {
                segments.push(EdgeSegment::LineTo(curr_pt));
            } else {
                let entry = move_towards(curr_pt, prev_pt, r);
                let exit_pt = move_towards(curr_pt, next_pt, r);
                segments.push(EdgeSegment::Bend {
                    entry,
                    control: curr_pt,
                    exit: exit_pt,
                });
            }
        }

        i += 1;
    }

    segments
}

/// Render a sequence of edge segments into an SVG path `d` attribute string.
pub fn segments_to_svg_path(segments: &[EdgeSegment]) -> String {
    let mut d = String::new();
    for seg in segments {
        match seg {
            EdgeSegment::MoveTo(p) => {
                d.push_str(&format!("M {:.1} {:.1}", p.x, p.y));
            }
            EdgeSegment::LineTo(p) => {
                d.push_str(&format!(" L {:.1} {:.1}", p.x, p.y));
            }
            EdgeSegment::Bend {
                entry,
                control,
                exit,
            } => {
                d.push_str(&format!(
                    " L {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}",
                    entry.x, entry.y, control.x, control.y, exit.x, exit.y
                ));
            }
            EdgeSegment::Hop {
                entry,
                foot_in,
                foot_out,
                exit,
                radius,
                sweep,
            } => {
                // _͡_ : flat foot → arc → flat foot
                d.push_str(&format!(
                    " L {:.1} {:.1} L {:.1} {:.1} A {:.1} {:.1} 0 0 {} {:.1} {:.1} L {:.1} {:.1}",
                    entry.x,
                    entry.y,
                    foot_in.x,
                    foot_in.y,
                    radius,
                    radius,
                    sweep,
                    foot_out.x,
                    foot_out.y,
                    exit.x,
                    exit.y,
                ));
            }
            EdgeSegment::HopRect {
                entry,
                foot_in,
                corner1,
                corner2,
                foot_out,
                exit,
            } => {
                // _|‾|_ : flat foot → up → across → down → flat foot
                d.push_str(&format!(
                    " L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1}",
                    entry.x, entry.y,
                    foot_in.x, foot_in.y,
                    corner1.x, corner1.y,
                    corner2.x, corner2.y,
                    foot_out.x, foot_out.y,
                    exit.x, exit.y,
                ));
            }
            EdgeSegment::HopSkip {
                entry,
                foot_in,
                foot_out,
                exit,
            } => {
                // -| |- : stub → gap → stub
                d.push_str(&format!(
                    " L {:.1} {:.1} L {:.1} {:.1} M {:.1} {:.1} L {:.1} {:.1}",
                    entry.x, entry.y, foot_in.x, foot_in.y, foot_out.x, foot_out.y, exit.x, exit.y,
                ));
            }
        }
    }
    d
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CrossingStyle;
    use crate::grid::Grid;
    use crate::routing::astar::GridPoint;

    fn make_grid() -> Grid {
        Grid::new(20, 20, 10, 0, 0)
    }

    fn gp(row: i64, col: i64) -> GridPoint {
        GridPoint { row, col }
    }

    #[test]
    fn test_straight_line_two_points() {
        let grid = make_grid();
        let pts = vec![gp(0, 0), gp(0, 5)];
        let segs = build_edge_segments(&pts, &grid, &HashSet::new(), 5.0, CrossingStyle::None);
        let d = segments_to_svg_path(&segs);
        assert!(d.starts_with("M 0.0 0.0"));
        assert!(d.contains("L 50.0 0.0"));
        assert!(!d.contains('Q'));
        assert!(!d.contains('A'));
    }

    #[test]
    fn test_bend_produces_quadratic_bezier() {
        let grid = make_grid();
        // L-shape: down then right
        let pts = vec![gp(0, 0), gp(3, 0), gp(3, 5)];
        let segs = build_edge_segments(&pts, &grid, &HashSet::new(), 5.0, CrossingStyle::None);
        let d = segments_to_svg_path(&segs);
        assert!(d.contains('Q'), "expected quadratic bezier in: {}", d);
    }

    #[test]
    fn test_collinear_points_simplified() {
        let grid = make_grid();
        // All horizontal — should simplify to just start + end
        let pts = vec![gp(0, 0), gp(0, 1), gp(0, 2), gp(0, 3)];
        let segs = build_edge_segments(&pts, &grid, &HashSet::new(), 5.0, CrossingStyle::None);
        // MoveTo + LineTo only, no Bend
        assert!(segs.iter().all(|s| !matches!(s, EdgeSegment::Bend { .. })));
    }

    #[test]
    fn test_crossing_point_preserved_through_simplification() {
        let grid = make_grid();
        // Horizontal path; crossing at collinear interior point (1,2)
        let pts = vec![gp(1, 0), gp(1, 1), gp(1, 2), gp(1, 3)];
        let mut crossing_set = HashSet::new();
        crossing_set.insert((1i64, 2i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::Arc);
        let d = segments_to_svg_path(&segs);
        // Should contain an arc command for the hop
        assert!(d.contains('A'), "expected arc hop in: {}", d);
    }

    #[test]
    fn test_hop_arc_fixed_diameter_regardless_of_segment_length() {
        // Long horizontal segment (0..10 cols) with crossing at col 5.
        // The arc must always span exactly one cell (entry and exit each
        // half a cell away from the crossing), even though the simplified
        // prev/next points are far apart.
        let grid = make_grid(); // cell_size = 10
        let pts: Vec<GridPoint> = (0..=10).map(|c| gp(0, c)).collect();
        let mut crossing_set = HashSet::new();
        crossing_set.insert((0i64, 5i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::Arc);

        let hop = segs.iter().find_map(|s| {
            if let EdgeSegment::Hop { entry, exit, .. } = s {
                Some((*entry, *exit))
            } else {
                None
            }
        });
        let (entry, exit) = hop.expect("expected a Hop segment");

        // Crossing is at world x = 5 * 10 = 50.
        // Entry should be at x = 50 - 5 = 45, exit at x = 50 + 5 = 55.
        assert!((entry.x - 45.0).abs() < 0.1, "entry.x = {}", entry.x);
        assert!((exit.x - 55.0).abs() < 0.1, "exit.x = {}", exit.x);
        assert!((entry.y).abs() < 0.1);
        assert!((exit.y).abs() < 0.1);
    }

    #[test]
    fn test_hop_arc_radius_is_half_cell_minus_foot() {
        // cell_size=10 → half_cell=5, HOP_FOOT_LEN=2 → radius should be 3
        let grid = make_grid();
        let pts = vec![gp(0, 0), gp(0, 2), gp(0, 4)];
        let mut crossing_set = HashSet::new();
        crossing_set.insert((0i64, 2i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::Arc);
        let expected_radius = grid.cell_size as f64 / 2.0 - HOP_FOOT_LEN;
        let has_hop = segs.iter().any(|s| {
            matches!(s, EdgeSegment::Hop { radius, .. } if (*radius - expected_radius).abs() < 0.01)
        });
        assert!(has_hop);
    }

    #[test]
    fn test_segments_to_svg_path_formats_correctly() {
        let segs = vec![
            EdgeSegment::MoveTo(RenderPoint::new(0.0, 0.0)),
            EdgeSegment::LineTo(RenderPoint::new(10.0, 0.0)),
            EdgeSegment::Bend {
                entry: RenderPoint::new(15.0, 0.0),
                control: RenderPoint::new(20.0, 0.0),
                exit: RenderPoint::new(20.0, 5.0),
            },
            EdgeSegment::Hop {
                entry: RenderPoint::new(25.0, 5.0),
                foot_in: RenderPoint::new(27.0, 5.0),
                foot_out: RenderPoint::new(33.0, 5.0),
                exit: RenderPoint::new(35.0, 5.0),
                radius: 3.0,
                sweep: 0,
            },
        ];
        let d = segments_to_svg_path(&segs);
        assert!(d.starts_with("M 0.0 0.0"));
        assert!(d.contains("L 10.0 0.0"));
        assert!(d.contains("Q 20.0 0.0 20.0 5.0"));
        // Flat feet flank the arc
        assert!(
            d.contains("L 25.0 5.0 L 27.0 5.0"),
            "entry foot missing: {d}"
        );
        assert!(d.contains("A 3.0 3.0 0 0 0 33.0 5.0"), "arc missing: {d}");
        assert!(d.contains("L 35.0 5.0"), "exit foot missing: {d}");
    }

    #[test]
    fn test_hop_shape_has_feet_and_smaller_arc() {
        // cell_size=10 → half_cell=5, foot=2 → radius=3, arc span=6px, feet=2px each
        let grid = make_grid();
        let pts: Vec<GridPoint> = (0..=6).map(|c| gp(0, c)).collect();
        let mut crossing_set = HashSet::new();
        crossing_set.insert((0i64, 3i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::Arc);
        let hop = segs.iter().find_map(|s| match s {
            EdgeSegment::Hop {
                entry,
                foot_in,
                foot_out,
                exit,
                radius,
                ..
            } => Some((*entry, *foot_in, *foot_out, *exit, *radius)),
            _ => None,
        });
        let (entry, foot_in, foot_out, exit, radius) = hop.expect("Hop segment expected");

        // Crossing at col 3 → world x=30. half_cell=5 → entry at x=25, exit at x=35.
        assert!((entry.x - 25.0).abs() < 0.1, "entry.x={}", entry.x);
        assert!((exit.x - 35.0).abs() < 0.1, "exit.x={}", exit.x);
        // Feet 2px toward center
        assert!((foot_in.x - 27.0).abs() < 0.1, "foot_in.x={}", foot_in.x);
        assert!((foot_out.x - 33.0).abs() < 0.1, "foot_out.x={}", foot_out.x);
        // Arc radius = half_cell - foot_len = 5 - 2 = 3
        assert!((radius - 3.0).abs() < 0.1, "radius={}", radius);
    }

    #[test]
    fn test_hop_rectangular_has_feet_and_corners() {
        // cell_size=10, crossing at col 3 (world x=30), horizontal travel rightward.
        // half_cell=5, foot=2 → entry at x=25, foot_in at x=27, foot_out at x=33, exit at x=35.
        // height=3 → corners at y=-3 (upward bump in y-down SVG).
        let grid = make_grid();
        let pts: Vec<GridPoint> = (0..=6).map(|c| gp(0, c)).collect();
        let mut crossing_set = HashSet::new();
        crossing_set.insert((0i64, 3i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::Rectangular);
        let d = segments_to_svg_path(&segs);

        // No arc, no pen lift
        assert!(!d.contains('A'), "no arc expected: {d}");
        let m_count = d.chars().filter(|&c| c == 'M').count();
        assert_eq!(m_count, 1, "no pen lift expected: {d}");

        let hop = segs.iter().find_map(|s| match s {
            EdgeSegment::HopRect {
                entry,
                foot_in,
                corner1,
                corner2,
                foot_out,
                exit,
            } => Some((*entry, *foot_in, *corner1, *corner2, *foot_out, *exit)),
            _ => None,
        });
        let (entry, foot_in, corner1, corner2, foot_out, exit) =
            hop.expect("HopRect segment expected");

        // Flat feet: same positions as arc
        assert!((entry.x - 25.0).abs() < 0.1, "entry.x={}", entry.x);
        assert!((foot_in.x - 27.0).abs() < 0.1, "foot_in.x={}", foot_in.x);
        assert!((foot_out.x - 33.0).abs() < 0.1, "foot_out.x={}", foot_out.x);
        assert!((exit.x - 35.0).abs() < 0.1, "exit.x={}", exit.x);
        // All on the baseline y=0
        assert!(entry.y.abs() < 0.1);
        assert!(foot_in.y.abs() < 0.1);
        assert!(foot_out.y.abs() < 0.1);
        assert!(exit.y.abs() < 0.1);
        // Corners offset perpendicularly (upward = negative y in SVG)
        let height = grid.cell_size as f64 / 2.0 - HOP_FOOT_LEN; // 3.0
        assert!((corner1.x - 27.0).abs() < 0.1, "corner1.x={}", corner1.x);
        assert!(
            (corner1.y - (-height)).abs() < 0.1,
            "corner1.y={}",
            corner1.y
        );
        assert!((corner2.x - 33.0).abs() < 0.1, "corner2.x={}", corner2.x);
        assert!(
            (corner2.y - (-height)).abs() < 0.1,
            "corner2.y={}",
            corner2.y
        );
    }

    #[test]
    fn test_hop_skip_breaks_stroke_with_feet() {
        // cell_size=10, crossing at col 3 (world x=30), horizontal travel rightward.
        // half_cell=5, foot=2 → entry=25, foot_in=27, gap, foot_out=33, exit=35.
        let grid = make_grid();
        let pts: Vec<GridPoint> = (0..=6).map(|c| gp(0, c)).collect();
        let mut crossing_set = HashSet::new();
        crossing_set.insert((0i64, 3i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::Skip);
        let d = segments_to_svg_path(&segs);

        // Pen lift present, no arc
        let m_count = d.chars().filter(|&c| c == 'M').count();
        assert!(m_count >= 2, "expected pen lift for skip gap: {d}");
        assert!(!d.contains('A'), "no arc expected: {d}");

        let skip = segs.iter().find_map(|s| match s {
            EdgeSegment::HopSkip {
                entry,
                foot_in,
                foot_out,
                exit,
            } => Some((*entry, *foot_in, *foot_out, *exit)),
            _ => None,
        });
        let (entry, foot_in, foot_out, exit) = skip.expect("HopSkip segment expected");

        // Same foot positions as Arc/Rect
        assert!((entry.x - 25.0).abs() < 0.1, "entry.x={}", entry.x);
        assert!((foot_in.x - 27.0).abs() < 0.1, "foot_in.x={}", foot_in.x);
        assert!((foot_out.x - 33.0).abs() < 0.1, "foot_out.x={}", foot_out.x);
        assert!((exit.x - 35.0).abs() < 0.1, "exit.x={}", exit.x);
        // All on baseline
        assert!(entry.y.abs() < 0.1);
        assert!(foot_in.y.abs() < 0.1);
        assert!(foot_out.y.abs() < 0.1);
        assert!(exit.y.abs() < 0.1);
    }

    #[test]
    fn test_crossing_style_none_ignores_crossing_set() {
        let grid = make_grid();
        let pts: Vec<GridPoint> = (0..=6).map(|c| gp(0, c)).collect();
        let mut crossing_set = HashSet::new();
        crossing_set.insert((0i64, 3i64));

        let segs = build_edge_segments(&pts, &grid, &crossing_set, 5.0, CrossingStyle::None);
        let d = segments_to_svg_path(&segs);

        // No special hop decorations — just a straight line
        assert!(!d.contains('A'), "no arc: {d}");
        let m_count = d.chars().filter(|&c| c == 'M').count();
        assert_eq!(m_count, 1, "no pen lift: {d}");
        let hop = segs.iter().any(|s| {
            matches!(
                s,
                EdgeSegment::Hop { .. } | EdgeSegment::HopRect { .. } | EdgeSegment::HopSkip { .. }
            )
        });
        assert!(!hop, "no hop segments expected");
    }
}
