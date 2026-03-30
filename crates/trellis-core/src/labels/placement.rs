use std::collections::HashMap;

use crate::grid::Grid;
use crate::labels::collision::collides;
use crate::labels::slide::slide_label_along_segment;
use crate::routing::astar::RoutedPath;
use trellis_parser::{DiagramType, Graph};

/// Padding around label text in pixels.
const LABEL_PADDING: f64 = 4.0;
/// Average character width at font size 12.
const LABEL_CHAR_WIDTH: f64 = 7.0;
/// Line height at font size 12.
const LABEL_LINE_HEIGHT: f64 = 14.0;

/// A placed label with its position and text.
#[derive(Debug, Clone)]
pub struct LabelPlacement {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub side: LabelSide,
}

/// Which side of the segment the label is placed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelSide {
    Above,
    Below,
    Left,
    Right,
}

/// A bounding box for collision detection.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// A segment of a simplified edge path (two consecutive points in world coordinates).
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Direction of a segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentDirection {
    Horizontal,
    Vertical,
}

impl Segment {
    pub fn length(&self) -> f64 {
        ((self.x2 - self.x1).powi(2) + (self.y2 - self.y1).powi(2)).sqrt()
    }

    pub fn midpoint(&self) -> (f64, f64) {
        ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }

    pub fn direction(&self) -> SegmentDirection {
        if (self.y2 - self.y1).abs() < 0.01 {
            SegmentDirection::Horizontal
        } else {
            SegmentDirection::Vertical
        }
    }
}

/// Estimate label text width in pixels.
fn measure_label_width(text: &str) -> f64 {
    let max_line_len = text.lines().map(|l| l.len()).max().unwrap_or(0);
    max_line_len as f64 * LABEL_CHAR_WIDTH
}

/// Estimate label text height in pixels.
fn measure_label_height(text: &str) -> f64 {
    let lines = text.lines().count().max(1);
    lines as f64 * LABEL_LINE_HEIGHT
}

/// Convert grid points of a routed path to simplified world-coordinate segments.
/// Removes collinear intermediate points, keeping only bend points.
fn path_to_segments(path: &RoutedPath, grid: &Grid) -> Vec<Segment> {
    if path.points.len() < 2 {
        return vec![];
    }

    // Convert to world coordinates
    let world_points: Vec<(f64, f64)> = path
        .points
        .iter()
        .map(|gp| grid.grid_to_world(gp.row as usize, gp.col as usize))
        .collect();

    // Simplify: remove collinear points
    let mut simplified = vec![world_points[0]];
    for i in 1..world_points.len() - 1 {
        let (px, py) = world_points[i - 1];
        let (cx, cy) = world_points[i];
        let (nx, ny) = world_points[i + 1];

        let dx1 = (cx - px).signum() as i32;
        let dy1 = (cy - py).signum() as i32;
        let dx2 = (nx - cx).signum() as i32;
        let dy2 = (ny - cy).signum() as i32;

        if dx1 != dx2 || dy1 != dy2 {
            simplified.push(world_points[i]);
        }
    }
    simplified.push(*world_points.last().unwrap());

    // Build segments
    simplified
        .windows(2)
        .map(|w| Segment {
            x1: w[0].0,
            y1: w[0].1,
            x2: w[1].0,
            y2: w[1].1,
        })
        .collect()
}

/// Select the best segment for label placement.
/// Preference: middle segment > longest segment.
fn select_best_segment(segments: &[Segment], label_width: f64) -> usize {
    if segments.is_empty() {
        return 0;
    }

    let middle_idx = segments.len() / 2;
    if segments[middle_idx].length() >= label_width {
        return middle_idx;
    }

    // Find the longest segment
    segments
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.length().partial_cmp(&b.length()).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(middle_idx)
}

/// Generate candidate positions for a label on a given segment.
fn generate_candidates(
    segment: &Segment,
    label_width: f64,
    label_height: f64,
) -> Vec<(f64, f64, LabelSide)> {
    let (mx, my) = segment.midpoint();
    let mut candidates = Vec::new();

    match segment.direction() {
        SegmentDirection::Horizontal => {
            // Above
            candidates.push((
                mx - label_width / 2.0,
                my - label_height - LABEL_PADDING,
                LabelSide::Above,
            ));
            // Below
            candidates.push((
                mx - label_width / 2.0,
                my + LABEL_PADDING,
                LabelSide::Below,
            ));
        }
        SegmentDirection::Vertical => {
            // Left
            candidates.push((
                mx - label_width - LABEL_PADDING,
                my - label_height / 2.0,
                LabelSide::Left,
            ));
            // Right
            candidates.push((
                mx + LABEL_PADDING,
                my - label_height / 2.0,
                LabelSide::Right,
            ));
        }
    }

    candidates
}

/// Place labels on all edges that have labels.
pub fn place_all_labels(
    graph: &Graph,
    routing_result: &HashMap<usize, RoutedPath>,
    grid: &Grid,
) -> Vec<LabelPlacement> {
    let mut placements = Vec::new();
    let mut placed_bboxes: Vec<BoundingBox> = Vec::new();

    // Collect all segments from all routed paths for collision detection
    let all_segments: Vec<Vec<Segment>> = routing_result
        .values()
        .map(|path| path_to_segments(path, grid))
        .collect();

    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        let base_label = match &edge.label {
            Some(l) if !l.is_empty() => l.as_str(),
            _ => continue,
        };

        // For C4 edges: append technology as a second label line "[Tech]".
        let display_text: String =
            if graph.diagram_type == DiagramType::C4Diagram {
                if let Some(tech) = &edge.c4_technology {
                    format!("{}\n[{}]", base_label, tech)
                } else {
                    base_label.to_string()
                }
            } else {
                base_label.to_string()
            };

        let path = match routing_result.get(&edge_idx) {
            Some(p) => p,
            None => continue,
        };

        let segments = path_to_segments(path, grid);
        if segments.is_empty() {
            continue;
        }

        let label = &display_text;
        let label_width = measure_label_width(label);
        let label_height = measure_label_height(label);

        let best_seg_idx = select_best_segment(&segments, label_width);
        let segment = &segments[best_seg_idx];

        let candidates = generate_candidates(segment, label_width, label_height);

        let bbox_width = label_width + 2.0 * LABEL_PADDING;
        let bbox_height = label_height + 2.0 * LABEL_PADDING;

        let mut placed = false;

        // Try each candidate position
        for (cx, cy, side) in &candidates {
            let bbox = BoundingBox {
                x: *cx,
                y: *cy,
                width: bbox_width,
                height: bbox_height,
            };

            if !collides(&bbox, &graph.nodes, &all_segments, &placed_bboxes) {
                placed_bboxes.push(bbox);
                placements.push(LabelPlacement {
                    text: label.clone(),
                    x: *cx,
                    y: *cy,
                    width: label_width,
                    height: label_height,
                    side: *side,
                });
                placed = true;
                break;
            }
        }

        // If no candidate worked, try sliding along the segment
        if !placed {
            let placement = slide_label_along_segment(
                segment,
                label,
                label_width,
                label_height,
                &graph.nodes,
                &all_segments,
                &mut placed_bboxes,
            );
            placements.push(placement);
        }
    }

    placements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_label_width() {
        let w = measure_label_width("Yes");
        assert!((w - 3.0 * LABEL_CHAR_WIDTH).abs() < 0.01);
    }

    #[test]
    fn test_measure_label_height() {
        let h = measure_label_height("Yes");
        assert!((h - LABEL_LINE_HEIGHT).abs() < 0.01);
    }

    #[test]
    fn test_segment_direction() {
        let horiz = Segment {
            x1: 0.0,
            y1: 10.0,
            x2: 50.0,
            y2: 10.0,
        };
        assert_eq!(horiz.direction(), SegmentDirection::Horizontal);

        let vert = Segment {
            x1: 10.0,
            y1: 0.0,
            x2: 10.0,
            y2: 50.0,
        };
        assert_eq!(vert.direction(), SegmentDirection::Vertical);
    }

    #[test]
    fn test_segment_midpoint() {
        let seg = Segment {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
        };
        let (mx, my) = seg.midpoint();
        assert!((mx - 50.0).abs() < 0.01);
        assert!((my - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_select_best_segment_middle() {
        let segments = vec![
            Segment { x1: 0.0, y1: 0.0, x2: 30.0, y2: 0.0 },
            Segment { x1: 30.0, y1: 0.0, x2: 30.0, y2: 50.0 },
            Segment { x1: 30.0, y1: 50.0, x2: 80.0, y2: 50.0 },
        ];
        // Middle segment (index 1) is long enough
        assert_eq!(select_best_segment(&segments, 40.0), 1);
    }

    #[test]
    fn test_select_best_segment_longest_fallback() {
        let segments = vec![
            Segment { x1: 0.0, y1: 0.0, x2: 10.0, y2: 0.0 },   // 10px
            Segment { x1: 10.0, y1: 0.0, x2: 10.0, y2: 5.0 },   // 5px (middle)
            Segment { x1: 10.0, y1: 5.0, x2: 100.0, y2: 5.0 },  // 90px
        ];
        // Middle segment too short (5px < 40px), falls back to longest (index 2)
        assert_eq!(select_best_segment(&segments, 40.0), 2);
    }

    #[test]
    fn test_generate_candidates_horizontal() {
        let seg = Segment {
            x1: 0.0,
            y1: 50.0,
            x2: 100.0,
            y2: 50.0,
        };
        let candidates = generate_candidates(&seg, 30.0, 14.0);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].2, LabelSide::Above);
        assert_eq!(candidates[1].2, LabelSide::Below);
    }

    #[test]
    fn test_generate_candidates_vertical() {
        let seg = Segment {
            x1: 50.0,
            y1: 0.0,
            x2: 50.0,
            y2: 100.0,
        };
        let candidates = generate_candidates(&seg, 30.0, 14.0);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].2, LabelSide::Left);
        assert_eq!(candidates[1].2, LabelSide::Right);
    }

    #[test]
    fn test_bounding_box_overlaps() {
        let a = BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        let b = BoundingBox { x: 5.0, y: 5.0, width: 10.0, height: 10.0 };
        assert!(a.overlaps(&b));

        let c = BoundingBox { x: 20.0, y: 20.0, width: 10.0, height: 10.0 };
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn test_bounding_box_no_overlap_adjacent() {
        let a = BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        let b = BoundingBox { x: 10.0, y: 0.0, width: 10.0, height: 10.0 };
        // Touching but not overlapping
        assert!(!a.overlaps(&b));
    }
}
