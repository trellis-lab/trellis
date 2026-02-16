use crate::labels::placement::{BoundingBox, Segment};
use trellis_parser::Node;

/// Check if a bounding box collides with any nodes, edge segments, or already-placed labels.
pub fn collides(
    bbox: &BoundingBox,
    nodes: &[Node],
    all_segments: &[Vec<Segment>],
    placed_labels: &[BoundingBox],
) -> bool {
    // 1. Check collision with nodes
    for node in nodes {
        let node_bbox = BoundingBox {
            x: node.x,
            y: node.y,
            width: node.width,
            height: node.height,
        };
        if bbox.overlaps(&node_bbox) {
            return true;
        }
    }

    // 2. Check collision with edge segments
    for segments in all_segments {
        for segment in segments {
            if bbox_intersects_segment(bbox, segment) {
                return true;
            }
        }
    }

    // 3. Check collision with already-placed labels
    for placed in placed_labels {
        if bbox.overlaps(placed) {
            return true;
        }
    }

    false
}

/// Check if a bounding box intersects with a line segment.
/// Uses axis-aligned bounding box vs line segment intersection.
fn bbox_intersects_segment(bbox: &BoundingBox, segment: &Segment) -> bool {
    let left = bbox.x;
    let right = bbox.x + bbox.width;
    let top = bbox.y;
    let bottom = bbox.y + bbox.height;

    let (sx1, sy1) = (segment.x1, segment.y1);
    let (sx2, sy2) = (segment.x2, segment.y2);

    // For orthogonal segments (horizontal or vertical), we can simplify
    let eps = 0.01;

    if (sy1 - sy2).abs() < eps {
        // Horizontal segment
        let seg_y = sy1;
        let seg_min_x = sx1.min(sx2);
        let seg_max_x = sx1.max(sx2);

        // Check if the segment's y is within the bbox and x ranges overlap
        seg_y > top && seg_y < bottom && seg_max_x > left && seg_min_x < right
    } else if (sx1 - sx2).abs() < eps {
        // Vertical segment
        let seg_x = sx1;
        let seg_min_y = sy1.min(sy2);
        let seg_max_y = sy1.max(sy2);

        seg_x > left && seg_x < right && seg_max_y > top && seg_min_y < bottom
    } else {
        // Diagonal segment (shouldn't happen with orthogonal routing, but handle gracefully)
        // Use simple AABB of the segment vs bbox overlap
        let seg_bbox = BoundingBox {
            x: sx1.min(sx2),
            y: sy1.min(sy2),
            width: (sx2 - sx1).abs(),
            height: (sy2 - sy1).abs(),
        };
        bbox.overlaps(&seg_bbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_intersects_horizontal_segment() {
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };

        // Segment passing through the bbox horizontally
        let seg = Segment {
            x1: 0.0,
            y1: 20.0,
            x2: 40.0,
            y2: 20.0,
        };
        assert!(bbox_intersects_segment(&bbox, &seg));

        // Segment above the bbox
        let seg_above = Segment {
            x1: 0.0,
            y1: 5.0,
            x2: 40.0,
            y2: 5.0,
        };
        assert!(!bbox_intersects_segment(&bbox, &seg_above));
    }

    #[test]
    fn test_bbox_intersects_vertical_segment() {
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };

        // Segment passing through the bbox vertically
        let seg = Segment {
            x1: 20.0,
            y1: 0.0,
            x2: 20.0,
            y2: 40.0,
        };
        assert!(bbox_intersects_segment(&bbox, &seg));

        // Segment to the left of the bbox
        let seg_left = Segment {
            x1: 5.0,
            y1: 0.0,
            x2: 5.0,
            y2: 40.0,
        };
        assert!(!bbox_intersects_segment(&bbox, &seg_left));
    }

    #[test]
    fn test_no_collision_with_empty() {
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        assert!(!collides(&bbox, &[], &[], &[]));
    }

    #[test]
    fn test_collision_with_node() {
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        let node = Node {
            id: "A".to_string(),
            label: "A".to_string(),
            shape: trellis_parser::NodeShape::Rectangle,
            width: 50.0,
            height: 30.0,
            x: 15.0,
            y: 15.0,
        };
        assert!(collides(&bbox, &[node], &[], &[]));
    }

    #[test]
    fn test_collision_with_placed_label() {
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        let placed = BoundingBox {
            x: 15.0,
            y: 15.0,
            width: 10.0,
            height: 10.0,
        };
        assert!(collides(&bbox, &[], &[], &[placed]));
    }

    #[test]
    fn test_segment_on_bbox_edge_no_collision() {
        // A segment exactly on the top edge of the bbox should NOT collide
        // (using strict inequalities)
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        let seg = Segment {
            x1: 0.0,
            y1: 10.0,
            x2: 40.0,
            y2: 10.0,
        };
        assert!(!bbox_intersects_segment(&bbox, &seg));
    }
}
