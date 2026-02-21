//! C4 diagram placement: boundary-aware row-flow layout.
//!
//! Elements are placed in two stages:
//! 1. Top-level elements (not in any `graph.subgraph`) are placed first.
//! 2. Each boundary group (one `Subgraph` per boundary block) is placed
//!    as a separate section below the top-level elements.
//!
//! After placement, `compute_c4_subgraph_data` converts the `Subgraph` tree
//! into the generic `(SubgraphTree, HashMap<String, BoundingBox>)` format so
//! that the viewBox calculation and boundary rendering can reuse common
//! infrastructure.

use std::collections::HashMap;
use trellis_parser::{Graph, Subgraph};

use crate::types::{BoundingBox, SubgraphTree, SubgraphTreeNode};

/// Horizontal gap between elements (pixels)
const ELEM_GAP_X: f64 = 40.0;
/// Vertical gap between rows (pixels)
const ELEM_GAP_Y: f64 = 60.0;
/// Elements per row (default)
const DEFAULT_SHAPES_PER_ROW: usize = 4;
/// Left margin
const MARGIN_X: f64 = 40.0;
/// Top margin
const MARGIN_Y: f64 = 40.0;
/// Padding inside a boundary frame (matches c4_boundary.rs PADDING)
const BOUNDARY_PADDING: f64 = 24.0;
/// Extra height for the boundary label at the top of the frame
const BOUNDARY_LABEL_HEIGHT: f64 = 20.0;

// ── Public entry points ───────────────────────────────────────────────────────

/// Place all C4 element nodes using boundary-aware row-flow layout.
///
/// Top-level elements (not inside any boundary) are placed first.
/// Elements inside each boundary are placed below as a separate group.
/// Boundary frame nodes themselves are skipped (rendered as overlays).
pub fn place_c4_diagram(graph: &mut Graph) {
    if graph.nodes.is_empty() {
        return;
    }

    // Build a node_id → boundary_id lookup from graph.subgraphs
    let containment = build_containment_map(graph);

    // ── Classify nodes ───────────────────────────────────────────────────────
    let mut top_level: Vec<usize> = Vec::new();
    // Keep boundaries in declaration order (same order as graph.subgraphs)
    let mut boundary_order: Vec<String> = Vec::new();
    let mut boundary_map: HashMap<String, Vec<usize>> = HashMap::new();

    // Pre-populate boundary_order from the subgraph list (preserves parse order)
    for sg in &graph.subgraphs {
        collect_subgraph_order(sg, &mut boundary_order, &mut boundary_map);
    }

    for (i, node) in graph.nodes.iter().enumerate() {
        let is_boundary = node.c4_type.map(|t| t.is_boundary()).unwrap_or(false);

        if is_boundary {
            // Boundary frame nodes are skipped in row-flow placement
        } else if let Some(bid) = containment.get(&node.id) {
            if let Some(group) = boundary_map.get_mut(bid) {
                group.push(i);
            }
        } else {
            top_level.push(i);
        }
    }

    // ── Place top-level elements ─────────────────────────────────────────────
    let shapes_per_row = DEFAULT_SHAPES_PER_ROW;
    let mut current_y = MARGIN_Y;

    if !top_level.is_empty() {
        let row_heights = compute_row_heights(&graph.nodes, &top_level, shapes_per_row);
        current_y = place_in_rows(
            graph,
            &top_level,
            MARGIN_X,
            current_y,
            shapes_per_row,
            &row_heights,
        );
        current_y += ELEM_GAP_Y;
    }

    // ── Place each boundary group below the top-level elements ───────────────
    for bid in &boundary_order {
        let contained = match boundary_map.get(bid) {
            Some(v) if !v.is_empty() => v.clone(),
            _ => continue,
        };

        // Space for the boundary label and top padding
        current_y += BOUNDARY_LABEL_HEIGHT + BOUNDARY_PADDING;

        let row_heights = compute_row_heights(&graph.nodes, &contained, shapes_per_row);
        current_y = place_in_rows(
            graph,
            &contained,
            MARGIN_X + BOUNDARY_PADDING,
            current_y,
            shapes_per_row,
            &row_heights,
        );

        // Bottom padding before the next group (or end of diagram)
        current_y += BOUNDARY_PADDING + ELEM_GAP_Y;
    }
}

/// Build the `(SubgraphTree, BoundingBox)` data from placed node positions.
///
/// Called after `place_c4_diagram` to produce the subgraph metadata that the
/// rendering pipeline uses for viewBox extension and boundary frame drawing.
pub fn compute_c4_subgraph_data(graph: &Graph) -> (SubgraphTree, HashMap<String, BoundingBox>) {
    let mut tree = SubgraphTree {
        nodes: HashMap::new(),
        root_id: "ROOT".to_string(),
    };

    // ROOT collects top-level boundary IDs
    let root_children: Vec<String> = graph.subgraphs.iter().map(|sg| sg.id.clone()).collect();
    tree.nodes.insert(
        "ROOT".to_string(),
        SubgraphTreeNode {
            id: "ROOT".to_string(),
            label: None,
            children: root_children,
            direct_node_ids: Vec::new(),
        },
    );

    // Flatten the Subgraph tree into SubgraphTreeNode entries
    for sg in &graph.subgraphs {
        flatten_subgraph(sg, &mut tree);
    }

    // Compute bounding boxes from placed node positions (bottom-up, recursive)
    let boxes = compute_bboxes(graph);

    (tree, boxes)
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Build a flat `node_id → boundary_id` lookup from the subgraph tree.
fn build_containment_map(graph: &Graph) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for sg in &graph.subgraphs {
        collect_sg_nodes(sg, &mut map);
    }
    map
}

/// Recursively collect `node_id → boundary_id` entries.
fn collect_sg_nodes(sg: &Subgraph, map: &mut HashMap<String, String>) {
    for id in &sg.nodes {
        map.insert(id.clone(), sg.id.clone());
    }
    for child in &sg.subgraphs {
        collect_sg_nodes(child, map);
    }
}

/// Pre-populate `boundary_order` and `boundary_map` from the subgraph list
/// so that boundaries appear in declaration order during placement.
fn collect_subgraph_order(
    sg: &Subgraph,
    order: &mut Vec<String>,
    map: &mut HashMap<String, Vec<usize>>,
) {
    order.push(sg.id.clone());
    map.insert(sg.id.clone(), Vec::new());
    for child in &sg.subgraphs {
        collect_subgraph_order(child, order, map);
    }
}

/// Flatten a `Subgraph` tree into `SubgraphTreeNode` entries in the `SubgraphTree`.
fn flatten_subgraph(sg: &Subgraph, tree: &mut SubgraphTree) {
    tree.nodes.insert(
        sg.id.clone(),
        SubgraphTreeNode {
            id: sg.id.clone(),
            label: sg.label.clone(),
            children: sg.subgraphs.iter().map(|c| c.id.clone()).collect(),
            direct_node_ids: sg.nodes.clone(),
        },
    );
    for child in &sg.subgraphs {
        flatten_subgraph(child, tree);
    }
}

/// Compute bounding boxes for every boundary subgraph from placed node positions.
///
/// Uses a bottom-up recursive traversal of `graph.subgraphs` so that outer
/// boundaries (e.g. a `Deployment_Node` wrapping other `Deployment_Node`s)
/// automatically encompass the already-computed inner bounding boxes.
///
/// The constants mirror those in `render/c4_boundary.rs` so the rendered frame
/// exactly matches the viewBox extension.
fn compute_bboxes(graph: &Graph) -> HashMap<String, BoundingBox> {
    const PADDING: f64 = 24.0;
    const LABEL_HEIGHT: f64 = 20.0;

    let mut boxes = HashMap::new();

    for sg in &graph.subgraphs {
        compute_bbox_recursive(sg, graph, &mut boxes, PADDING, LABEL_HEIGHT);
    }

    boxes
}

/// Recursively compute the bounding box for one subgraph and all its children.
///
/// Children are processed first (bottom-up), so when the parent reads child
/// bboxes they are already available in `boxes`.
fn compute_bbox_recursive(
    sg: &Subgraph,
    graph: &Graph,
    boxes: &mut HashMap<String, BoundingBox>,
    padding: f64,
    label_height: f64,
) {
    // Recurse into children first (bottom-up order)
    for child in &sg.subgraphs {
        compute_bbox_recursive(child, graph, boxes, padding, label_height);
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    // Extend from directly-contained nodes
    for node_id in &sg.nodes {
        if let Some(n) = graph.nodes.iter().find(|n| &n.id == node_id) {
            min_x = min_x.min(n.x);
            min_y = min_y.min(n.y);
            max_x = max_x.max(n.x + n.width);
            max_y = max_y.max(n.y + n.height);
        }
    }

    // Extend from child-boundary bounding boxes (already computed above)
    for child_sg in &sg.subgraphs {
        if let Some(child_bbox) = boxes.get(&child_sg.id) {
            min_x = min_x.min(child_bbox.x);
            min_y = min_y.min(child_bbox.y);
            max_x = max_x.max(child_bbox.x + child_bbox.width);
            max_y = max_y.max(child_bbox.y + child_bbox.height);
        }
    }

    if min_x.is_finite() {
        boxes.insert(
            sg.id.clone(),
            BoundingBox {
                x: min_x - padding,
                y: min_y - padding - label_height,
                width: (max_x - min_x) + 2.0 * padding,
                height: (max_y - min_y) + 2.0 * padding + label_height,
            },
        );
    }
}

/// Compute the maximum height of each row for a slice of node indices.
fn compute_row_heights(nodes: &[trellis_parser::Node], indices: &[usize], shapes_per_row: usize) -> Vec<f64> {
    let mut row_heights: Vec<f64> = Vec::new();
    let mut max_h = 0.0_f64;

    for (slot, &idx) in indices.iter().enumerate() {
        max_h = max_h.max(nodes[idx].height);
        if (slot + 1) % shapes_per_row == 0 || slot + 1 == indices.len() {
            row_heights.push(max_h);
            max_h = 0.0;
        }
    }

    row_heights
}

/// Place nodes in row-flow order. Returns the y coordinate after the last row.
fn place_in_rows(
    graph: &mut Graph,
    indices: &[usize],
    start_x: f64,
    start_y: f64,
    shapes_per_row: usize,
    row_heights: &[f64],
) -> f64 {
    if indices.is_empty() {
        return start_y;
    }

    let mut row_start_y = start_y;
    let mut x_cursor = start_x;
    let mut last_row_idx = 0;

    for (slot, &idx) in indices.iter().enumerate() {
        let row_idx = slot / shapes_per_row;
        let col_idx = slot % shapes_per_row;
        last_row_idx = row_idx;

        if col_idx == 0 {
            x_cursor = start_x;
        }

        graph.nodes[idx].x = x_cursor;
        graph.nodes[idx].y = row_start_y;

        x_cursor += graph.nodes[idx].width + ELEM_GAP_X;

        // Advance to the next row when this row is full and more nodes follow
        if col_idx + 1 == shapes_per_row && slot + 1 < indices.len() {
            row_start_y += row_heights.get(row_idx).copied().unwrap_or(0.0) + ELEM_GAP_Y;
        }
    }

    row_start_y + row_heights.get(last_row_idx).copied().unwrap_or(0.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::parse;

    #[test]
    fn test_place_c4_two_elements() {
        let mut graph = parse(
            "C4Context\n\
             Person(alice, \"Alice\")\n\
             System(app, \"App\")\n",
        )
        .expect("parse failed");

        place_c4_diagram(&mut graph);

        let alice = graph.nodes.iter().find(|n| n.id == "alice").unwrap();
        let app = graph.nodes.iter().find(|n| n.id == "app").unwrap();

        assert!(alice.y >= MARGIN_Y);
        assert!(app.y >= MARGIN_Y);
        assert!(app.x > alice.x, "app should be to the right of alice");
    }

    #[test]
    fn test_place_c4_row_wrapping() {
        let mut graph = parse(
            "C4Context\n\
             Person(a, \"A\")\n\
             System(b, \"B\")\n\
             System(c, \"C\")\n\
             System(d, \"D\")\n\
             System(e, \"E\")\n",
        )
        .expect("parse failed");

        place_c4_diagram(&mut graph);

        let a_y = graph.nodes.iter().find(|n| n.id == "a").unwrap().y;
        let b_y = graph.nodes.iter().find(|n| n.id == "b").unwrap().y;
        let e_y = graph.nodes.iter().find(|n| n.id == "e").unwrap().y;

        assert_eq!(a_y, b_y, "a and b should be in same row");
        assert!(e_y > a_y, "e should be in a lower row than a");
    }

    #[test]
    fn test_boundary_group_placed_below_top_level() {
        let src = "C4Container\n\
                   Person(customer, \"Customer\")\n\
                   System_Boundary(sb, \"My System\") {\n\
                   Container(api, \"API\", \"Rust\", \"Backend\")\n\
                   }\n\
                   Rel(customer, api, \"Uses\")\n";

        let mut graph = parse(src).expect("parse failed");
        place_c4_diagram(&mut graph);

        let customer = graph.nodes.iter().find(|n| n.id == "customer").unwrap();
        let api = graph.nodes.iter().find(|n| n.id == "api").unwrap();

        assert!(
            api.y > customer.y,
            "boundary-contained 'api' (y={}) should be below top-level 'customer' (y={})",
            api.y,
            customer.y
        );
    }

    #[test]
    fn test_compute_c4_subgraph_data() {
        let src = "C4Container\n\
                   System_Boundary(sb, \"My System\") {\n\
                   Container(api, \"API\", \"Rust\", \"Backend\")\n\
                   }\n";

        let mut graph = parse(src).expect("parse failed");
        place_c4_diagram(&mut graph);

        let (tree, boxes) = compute_c4_subgraph_data(&graph);

        // Tree should contain ROOT and sb
        assert!(tree.nodes.contains_key("ROOT"));
        assert!(tree.nodes.contains_key("sb"));
        assert!(tree.nodes["sb"].direct_node_ids.contains(&"api".to_string()));

        // BoundingBox should be computed for sb
        assert!(boxes.contains_key("sb"), "bbox should exist for boundary 'sb'");
        let bbox = &boxes["sb"];
        assert!(bbox.width > 0.0);
        assert!(bbox.height > 0.0);
    }
}
