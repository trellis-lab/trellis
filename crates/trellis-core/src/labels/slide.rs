use crate::labels::collision::collides;
use crate::labels::placement::{BoundingBox, LabelPlacement, LabelSide, Segment, SegmentDirection};
use trellis_parser::Node;

// Re-export for clarity — slide always places labels on-edge.
const ON_EDGE_SIDES: &[LabelSide] = &[LabelSide::OnEdge];

/// Padding around label text in pixels.
const LABEL_PADDING: f64 = 4.0;
/// Step size in pixels when sliding along a segment.
const STEP: f64 = 5.0;
/// Gap between staggered fallback labels.
const STAGGER_GAP: f64 = 2.0;
/// Max stagger attempts before accepting whatever position we have.
const MAX_STAGGER_STEPS: u32 = 30;

/// Slide a label along a segment to find a collision-free position.
/// Starts from the midpoint and alternates outward in both directions.
/// If no collision-free position is found, falls back to the segment midpoint.
pub fn slide_label_along_segment(
    segment: &Segment,
    label: &str,
    label_width: f64,
    label_height: f64,
    nodes: &[Node],
    all_segments: &[Vec<Segment>],
    placed_bboxes: &mut Vec<BoundingBox>,
) -> LabelPlacement {
    let seg_length = segment.length();
    let seg_dir = segment.direction();

    let sides: &[LabelSide] = ON_EDGE_SIDES;

    let bbox_width = label_width + 2.0 * LABEL_PADDING;
    let bbox_height = label_height + 2.0 * LABEL_PADDING;

    // Try positions from center outward
    let mut offset = 0.0_f64;
    let mut step_count = 0u32;

    loop {
        if offset.abs() > seg_length / 2.0 {
            break;
        }

        let t = 0.5 + offset / seg_length;
        let pos_x = segment.x1 + (segment.x2 - segment.x1) * t;
        let pos_y = segment.y1 + (segment.y2 - segment.y1) * t;

        for &side in sides {
            let (cx, cy) = candidate_position(pos_x, pos_y, label_width, label_height, side);
            let bbox = BoundingBox {
                x: cx,
                y: cy,
                width: bbox_width,
                height: bbox_height,
            };

            if !collides(&bbox, nodes, all_segments, placed_bboxes) {
                placed_bboxes.push(bbox);
                return LabelPlacement {
                    text: label.to_string(),
                    x: cx,
                    y: cy,
                    width: label_width,
                    height: label_height,
                    side,
                };
            }
        }

        // Alternate: +STEP, -STEP, +2*STEP, -2*STEP, ...
        step_count += 1;
        let abs_offset = (step_count as f64 / 2.0).ceil() * STEP;
        let step_sign: f64 = if step_count.is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        offset = abs_offset * step_sign;
    }

    // Slide exhausted — stagger perpendicular to the segment to avoid piling
    // up on top of already-placed labels. For vertical segments we shift Y;
    // for horizontal segments we shift X. Alternate ±1, ±2, … stagger_step
    // increments until we find a spot clear of placed labels.
    let (mx, my) = segment.midpoint();
    let default_side = sides[0];
    let (fx, fy) = candidate_position(mx, my, label_width, label_height, default_side);

    // Step size matches the label dimension in the stagger direction.
    let stagger_step = match seg_dir {
        SegmentDirection::Horizontal => label_width + STAGGER_GAP,
        SegmentDirection::Vertical => label_height + STAGGER_GAP,
    };

    for stagger in 1..=MAX_STAGGER_STEPS {
        let sign: f64 = if stagger % 2 == 1 { 1.0 } else { -1.0 };
        let magnitude = stagger.div_ceil(2) as f64 * stagger_step;
        let (sx, sy) = match seg_dir {
            SegmentDirection::Vertical => (fx, fy + sign * magnitude),
            SegmentDirection::Horizontal => (fx + sign * magnitude, fy),
        };

        let bbox = BoundingBox {
            x: sx,
            y: sy,
            width: bbox_width,
            height: bbox_height,
        };

        if !collides(&bbox, nodes, all_segments, placed_bboxes) {
            placed_bboxes.push(bbox);
            return LabelPlacement {
                text: label.to_string(),
                x: sx,
                y: sy,
                width: label_width,
                height: label_height,
                side: default_side,
            };
        }
    }

    // All stagger positions exhausted — accept the original midpoint position.
    let fallback_bbox = BoundingBox {
        x: fx,
        y: fy,
        width: bbox_width,
        height: bbox_height,
    };
    placed_bboxes.push(fallback_bbox);

    LabelPlacement {
        text: label.to_string(),
        x: fx,
        y: fy,
        width: label_width,
        height: label_height,
        side: default_side,
    }
}

/// Calculate the top-left position of a label given the anchor point and side.
fn candidate_position(
    anchor_x: f64,
    anchor_y: f64,
    label_width: f64,
    label_height: f64,
    side: LabelSide,
) -> (f64, f64) {
    match side {
        LabelSide::Above => (
            anchor_x - label_width / 2.0,
            anchor_y - label_height - LABEL_PADDING,
        ),
        LabelSide::Below => (anchor_x - label_width / 2.0, anchor_y + LABEL_PADDING),
        LabelSide::Left => (
            anchor_x - label_width - LABEL_PADDING,
            anchor_y - label_height / 2.0,
        ),
        LabelSide::Right => (anchor_x + LABEL_PADDING, anchor_y - label_height / 2.0),
        LabelSide::OnEdge => (
            anchor_x - label_width / 2.0,
            anchor_y - label_height / 2.0,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_finds_free_position() {
        let segment = Segment {
            x1: 0.0,
            y1: 50.0,
            x2: 200.0,
            y2: 50.0,
        };
        let mut placed = Vec::new();

        let result = slide_label_along_segment(&segment, "Test", 28.0, 14.0, &[], &[], &mut placed);

        // Should have placed a label and added to placed_bboxes
        assert_eq!(placed.len(), 1);
        assert_eq!(result.text, "Test");
    }

    #[test]
    fn test_slide_avoids_existing_label() {
        let segment = Segment {
            x1: 0.0,
            y1: 50.0,
            x2: 200.0,
            y2: 50.0,
        };

        // Place a label at the midpoint above
        let blocking = BoundingBox {
            x: 86.0, // roughly centered at 100
            y: 28.0, // above the segment
            width: 36.0,
            height: 22.0,
        };
        let mut placed = vec![blocking];

        let result = slide_label_along_segment(&segment, "Test", 28.0, 14.0, &[], &[], &mut placed);

        // Should have found a different position
        assert_eq!(placed.len(), 2);
        assert_eq!(result.text, "Test");
    }

    #[test]
    fn test_candidate_position_above() {
        let (x, y) = candidate_position(50.0, 50.0, 20.0, 14.0, LabelSide::Above);
        assert!((x - 40.0).abs() < 0.01); // 50 - 20/2
        assert!((y - 32.0).abs() < 0.01); // 50 - 14 - 4
    }

    #[test]
    fn test_candidate_position_below() {
        let (x, y) = candidate_position(50.0, 50.0, 20.0, 14.0, LabelSide::Below);
        assert!((x - 40.0).abs() < 0.01); // 50 - 20/2
        assert!((y - 54.0).abs() < 0.01); // 50 + 4
    }
}
