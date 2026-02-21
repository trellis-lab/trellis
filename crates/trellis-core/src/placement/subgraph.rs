use std::collections::{HashMap, HashSet};
use trellis_parser::{Edge, Graph, Node, NodeShape};

use crate::types::{BoundingBox, SubgraphTree, SubgraphTreeNode};

use super::sugiyama;

/// Height reserved for the subgraph label (pixels), only added when a label exists
pub const LABEL_HEIGHT: f64 = 25.0;

/// Minimum gap between external nodes and a sibling subgraph boundary (in cell_size units)
const SUBGRAPH_MARGIN_CELLS: f64 = 2.0;

/// Prefix for virtual subgraph nodes (used for subgraph edges)
pub const VIRTUAL_PREFIX: &str = "__sg_virtual_";

// ---------------------------------------------------------------------------
// Subgraph tree building
// ---------------------------------------------------------------------------

/// Build a flattened subgraph tree from the parser's recursive `Vec<Subgraph>`.
///
/// This is a two-pass approach:
/// 1. Flatten the tree structure
/// 2. For each node, find which subgraphs contain it, pick the deepest
pub fn build_subgraph_tree_with_ownership(graph: &Graph) -> SubgraphTree {
    let mut tree = SubgraphTree {
        nodes: HashMap::new(),
        root_id: "ROOT".to_string(),
    };

    tree.nodes.insert(
        "ROOT".to_string(),
        SubgraphTreeNode {
            id: "ROOT".to_string(),
            label: None,
            children: Vec::new(),
            direct_node_ids: Vec::new(),
        },
    );

    // Build a map of subgraph_id -> set of node IDs it contains (from parser)
    let mut sg_node_sets: HashMap<String, HashSet<String>> = HashMap::new();

    for sg in &graph.subgraphs {
        flatten_subgraph_with_sets(sg, "ROOT", &mut tree, &mut sg_node_sets);
    }

    // For each node, find the deepest subgraph that contains it
    for node in &graph.nodes {
        let owner = find_deepest_containing_sg(&node.id, "ROOT", &tree, &sg_node_sets)
            .unwrap_or_else(|| "ROOT".to_string());
        if let Some(tree_node) = tree.nodes.get_mut(&owner) {
            if !tree_node.direct_node_ids.contains(&node.id) {
                tree_node.direct_node_ids.push(node.id.clone());
            }
        }
    }

    tree
}

fn flatten_subgraph_with_sets(
    sg: &trellis_parser::Subgraph,
    parent_id: &str,
    tree: &mut SubgraphTree,
    sg_node_sets: &mut HashMap<String, HashSet<String>>,
) {
    let tree_node = SubgraphTreeNode {
        id: sg.id.clone(),
        label: sg.label.clone(),
        children: sg.subgraphs.iter().map(|c| c.id.clone()).collect(),
        direct_node_ids: Vec::new(),
    };

    tree.nodes.insert(sg.id.clone(), tree_node);

    if let Some(parent) = tree.nodes.get_mut(parent_id) {
        if !parent.children.contains(&sg.id) {
            parent.children.push(sg.id.clone());
        }
    }

    // Store the node set from the parser
    sg_node_sets.insert(sg.id.clone(), sg.nodes.iter().cloned().collect());

    for child in &sg.subgraphs {
        flatten_subgraph_with_sets(child, &sg.id, tree, sg_node_sets);
    }
}

/// Find the deepest subgraph under `subtree_id` that contains `node_id`.
///
/// This recursively searches all descendants. A node may be in a grandchild
/// without being in the parent's node set (the parser only records nodes
/// in the subgraph where they are directly referenced).
///
/// Returns `Some(subgraph_id)` if found, `None` if not found in this subtree.
fn find_deepest_containing_sg(
    node_id: &str,
    subtree_id: &str,
    tree: &SubgraphTree,
    sg_node_sets: &HashMap<String, HashSet<String>>,
) -> Option<String> {
    if let Some(tree_node) = tree.nodes.get(subtree_id) {
        for child_id in &tree_node.children {
            // Recursively search this child's entire subtree first (prefer deepest)
            if let Some(found) = find_deepest_containing_sg(node_id, child_id, tree, sg_node_sets) {
                return Some(found);
            }
        }
    }

    // Check if this subtree itself directly contains the node
    if subtree_id != "ROOT" {
        if let Some(node_set) = sg_node_sets.get(subtree_id) {
            if node_set.contains(node_id) {
                return Some(subtree_id.to_string());
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Recursive placement
// ---------------------------------------------------------------------------

/// Place nodes with subgraph awareness using bottom-up recursive layout.
///
/// Returns the subgraph tree and bounding boxes for rendering.
pub fn place_with_subgraphs(
    graph: &mut Graph,
    cell_size: i32,
) -> (SubgraphTree, HashMap<String, BoundingBox>) {
    let tree = build_subgraph_tree_with_ownership(graph);
    let mut subgraph_boxes: HashMap<String, BoundingBox> = HashMap::new();

    // Recursively place from ROOT
    place_subgraph_recursive("ROOT", &tree, graph, &mut subgraph_boxes, cell_size);

    (tree, subgraph_boxes)
}

/// Recursively place nodes within a subtree (bottom-up).
fn place_subgraph_recursive(
    subtree_id: &str,
    tree: &SubgraphTree,
    graph: &mut Graph,
    subgraph_boxes: &mut HashMap<String, BoundingBox>,
    cell_size: i32,
) {
    let tree_node = match tree.nodes.get(subtree_id) {
        Some(n) => n.clone(),
        None => return,
    };

    // 1. Recurse into children first (bottom-up)
    for child_id in &tree_node.children {
        place_subgraph_recursive(child_id, tree, graph, subgraph_boxes, cell_size);
    }

    // 2. Build a local graph with virtual nodes
    let mut local_nodes: Vec<Node> = Vec::new();
    let mut local_edges: Vec<Edge> = Vec::new();

    // Direct real nodes
    let direct_ids: HashSet<String> = tree_node.direct_node_ids.iter().cloned().collect();
    for node in &graph.nodes {
        if direct_ids.contains(&node.id) {
            local_nodes.push(node.clone());
        }
    }

    // Child subgraphs as virtual nodes
    for child_id in &tree_node.children {
        if let Some(bbox) = subgraph_boxes.get(child_id) {
            local_nodes.push(Node {
                id: child_id.clone(),
                label: String::new(),
                shape: NodeShape::Rectangle,
                width: bbox.width,
                height: bbox.height,
                x: 0.0,
                y: 0.0,
            ..Default::default()            });
        }
    }

    let cs = cell_size as f64;
    let padding = cs * 2.0; // one grid cell on each side
    let has_label = tree_node.label.is_some();
    let label_h = if has_label { LABEL_HEIGHT } else { 0.0 };

    if local_nodes.is_empty() {
        // Empty subgraph
        if subtree_id != "ROOT" {
            subgraph_boxes.insert(
                subtree_id.to_string(),
                BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 2.0 * padding,
                    height: 2.0 * padding + label_h,
                },
            );
        }
        return;
    }

    // Collect edges that connect nodes within this level
    let local_node_ids: HashSet<String> = local_nodes.iter().map(|n| n.id.clone()).collect();
    for edge in &graph.edges {
        let from_local = local_node_ids.contains(&edge.from);
        let to_local = local_node_ids.contains(&edge.to);
        if from_local && to_local {
            local_edges.push(edge.clone());
        }
    }

    // 3. Run Sugiyama on the local graph
    let mut local_graph = Graph {
        nodes: local_nodes,
        edges: local_edges,
        subgraphs: Vec::new(),
        direction: graph.direction,
        diagram_type: graph.diagram_type,
    };

    // Snap local node dimensions to grid before layout
    super::snap_node_dimensions_to_grid(&mut local_graph, cell_size);
    sugiyama::layout(&mut local_graph);

    // 4. Apply padding offset for non-ROOT subtrees
    let padding_x = if subtree_id == "ROOT" { 0.0 } else { padding };
    let padding_y = if subtree_id == "ROOT" {
        0.0
    } else {
        padding + label_h
    };

    // Normalize: shift so minimum coordinates start at padding
    let min_x = local_graph
        .nodes
        .iter()
        .map(|n| n.x)
        .fold(f64::INFINITY, f64::min);
    let min_y = local_graph
        .nodes
        .iter()
        .map(|n| n.y)
        .fold(f64::INFINITY, f64::min);

    for node in &mut local_graph.nodes {
        node.x = node.x - min_x + padding_x;
        node.y = node.y - min_y + padding_y;
    }

    // 5. Write back real node positions to the main graph
    let child_ids: HashSet<String> = tree_node.children.iter().cloned().collect();

    for local_node in &local_graph.nodes {
        if child_ids.contains(&local_node.id) {
            // This is a child subgraph virtual node - offset its contents
            let child_bbox = subgraph_boxes.get(&local_node.id).cloned();
            if let Some(bbox) = child_bbox {
                let dx = local_node.x - bbox.x;
                let dy = local_node.y - bbox.y;
                offset_subgraph_contents(&local_node.id, dx, dy, graph, tree, subgraph_boxes);
            }
        } else {
            // Real node - write position back
            if let Some(graph_node) = graph.nodes.iter_mut().find(|n| n.id == local_node.id) {
                graph_node.x = local_node.x;
                graph_node.y = local_node.y;
                graph_node.width = local_node.width;
                graph_node.height = local_node.height;
            }
        }
    }

    // 5b. Enforce minimum spacing between real nodes and sibling subgraph boxes
    {
        let direct_ids_set: HashSet<String> = tree_node.direct_node_ids.iter().cloned().collect();
        let margin = SUBGRAPH_MARGIN_CELLS * cs;
        enforce_subgraph_margin(graph, &direct_ids_set, &child_ids, subgraph_boxes, margin);
    }

    // 6. Compute bounding box for this subtree
    if subtree_id != "ROOT" {
        let mut bb_min_x = f64::INFINITY;
        let mut bb_min_y = f64::INFINITY;
        let mut bb_max_x = f64::NEG_INFINITY;
        let mut bb_max_y = f64::NEG_INFINITY;

        // Consider real node positions
        for node_id in &tree_node.direct_node_ids {
            if let Some(node) = graph.nodes.iter().find(|n| n.id == *node_id) {
                bb_min_x = bb_min_x.min(node.x);
                bb_min_y = bb_min_y.min(node.y);
                bb_max_x = bb_max_x.max(node.x + node.width);
                bb_max_y = bb_max_y.max(node.y + node.height);
            }
        }

        // Consider child subgraph bounding boxes
        for child_id in &tree_node.children {
            if let Some(bbox) = subgraph_boxes.get(child_id) {
                bb_min_x = bb_min_x.min(bbox.x);
                bb_min_y = bb_min_y.min(bbox.y);
                bb_max_x = bb_max_x.max(bbox.x + bbox.width);
                bb_max_y = bb_max_y.max(bbox.y + bbox.height);
            }
        }

        if bb_min_x.is_finite() {
            subgraph_boxes.insert(
                subtree_id.to_string(),
                BoundingBox {
                    x: bb_min_x - padding,
                    y: bb_min_y - padding - label_h,
                    width: (bb_max_x - bb_min_x) + 2.0 * padding,
                    height: (bb_max_y - bb_min_y) + 2.0 * padding + label_h,
                },
            );
        }
    }
}

/// Recursively offset all nodes and bounding boxes within a subgraph by (dx, dy).
///
/// Offsets the subgraph's own bounding box, all direct nodes, and all
/// descendant subgraph bounding boxes and their nodes.
fn offset_subgraph_contents(
    subgraph_id: &str,
    dx: f64,
    dy: f64,
    graph: &mut Graph,
    tree: &SubgraphTree,
    subgraph_boxes: &mut HashMap<String, BoundingBox>,
) {
    // Offset this subgraph's own bounding box
    if let Some(bbox) = subgraph_boxes.get_mut(subgraph_id) {
        bbox.x += dx;
        bbox.y += dy;
    }

    if let Some(tree_node) = tree.nodes.get(subgraph_id).cloned() {
        // Offset direct nodes
        for node_id in &tree_node.direct_node_ids {
            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                node.x += dx;
                node.y += dy;
            }
        }

        // Recurse into children (they handle their own bbox + nodes)
        for child_id in &tree_node.children {
            offset_subgraph_contents(child_id, dx, dy, graph, tree, subgraph_boxes);
        }
    }
}

/// Push external nodes away from subgraph bounding boxes to maintain `margin`.
///
/// Only operates on nodes that are direct members of the current subtree level
/// (`direct_node_ids`), not nodes nested inside child subgraphs.
fn enforce_subgraph_margin(
    graph: &mut Graph,
    direct_node_ids: &HashSet<String>,
    child_sg_ids: &HashSet<String>,
    subgraph_boxes: &HashMap<String, BoundingBox>,
    margin: f64,
) {
    // Collect child subgraph bounding boxes
    let sg_boxes: Vec<&BoundingBox> = child_sg_ids
        .iter()
        .filter_map(|id| subgraph_boxes.get(id))
        .collect();

    if sg_boxes.is_empty() {
        return;
    }

    for node in &mut graph.nodes {
        // Only process nodes that are direct members of this level
        if !direct_node_ids.contains(&node.id) {
            continue;
        }

        let node_right = node.x + node.width;
        let node_bottom = node.y + node.height;

        for bbox in &sg_boxes {
            let sg_right = bbox.x + bbox.width;
            let sg_bottom = bbox.y + bbox.height;

            // Check if there's horizontal and vertical overlap/proximity
            let h_overlap = node.x < sg_right + margin && node_right > bbox.x - margin;
            let v_overlap = node.y < sg_bottom + margin && node_bottom > bbox.y - margin;

            if !h_overlap || !v_overlap {
                continue; // No proximity issue
            }

            // Determine the push direction: find which axis has the smallest overlap
            // and push the node away along that axis
            let push_right = sg_right + margin - node.x;
            let push_left = node_right - (bbox.x - margin);
            let push_down = sg_bottom + margin - node.y;
            let push_up = node_bottom - (bbox.y - margin);

            // Find the smallest positive push
            let mut min_push = f64::INFINITY;
            let mut best_dx = 0.0;
            let mut best_dy = 0.0;

            if push_right > 0.0 && push_right < min_push {
                min_push = push_right;
                best_dx = push_right;
                best_dy = 0.0;
            }
            if push_left > 0.0 && push_left < min_push {
                min_push = push_left;
                best_dx = -push_left;
                best_dy = 0.0;
            }
            if push_down > 0.0 && push_down < min_push {
                min_push = push_down;
                best_dx = 0.0;
                best_dy = push_down;
            }
            if push_up > 0.0 && push_up < min_push {
                best_dx = 0.0;
                best_dy = -push_up;
            }

            node.x += best_dx;
            node.y += best_dy;
        }
    }
}

// ---------------------------------------------------------------------------
// Subgraph edge resolution
// ---------------------------------------------------------------------------

/// Resolve edges that reference subgraph IDs as source/target.
///
/// Creates virtual nodes at the subgraph bounding box position/size.
/// Virtual nodes have IDs prefixed with `__sg_virtual_` and do NOT block the grid.
pub fn resolve_subgraph_edges(graph: &mut Graph, subgraph_boxes: &HashMap<String, BoundingBox>) {
    let subgraph_ids: HashSet<String> = subgraph_boxes.keys().cloned().collect();
    let mut virtual_nodes_added: HashSet<String> = HashSet::new();

    for edge in &mut graph.edges {
        if subgraph_ids.contains(&edge.from) {
            let virtual_id = format!("{}{}", VIRTUAL_PREFIX, edge.from);
            if !virtual_nodes_added.contains(&virtual_id) {
                if let Some(bbox) = subgraph_boxes.get(&edge.from) {
                    graph.nodes.push(Node {
                        id: virtual_id.clone(),
                        label: String::new(),
                        shape: NodeShape::Rectangle,
                        width: bbox.width,
                        height: bbox.height,
                        x: bbox.x,
                        y: bbox.y,
            ..Default::default()                    });
                    virtual_nodes_added.insert(virtual_id.clone());
                }
            }
            edge.from = format!("{}{}", VIRTUAL_PREFIX, edge.from);
        }

        if subgraph_ids.contains(&edge.to) {
            let virtual_id = format!("{}{}", VIRTUAL_PREFIX, edge.to);
            if !virtual_nodes_added.contains(&virtual_id) {
                if let Some(bbox) = subgraph_boxes.get(&edge.to) {
                    graph.nodes.push(Node {
                        id: virtual_id.clone(),
                        label: String::new(),
                        shape: NodeShape::Rectangle,
                        width: bbox.width,
                        height: bbox.height,
                        x: bbox.x,
                        y: bbox.y,
            ..Default::default()                    });
                    virtual_nodes_added.insert(virtual_id.clone());
                }
            }
            edge.to = format!("{}{}", VIRTUAL_PREFIX, edge.to);
        }
    }

    // Remove original subgraph nodes from graph.nodes — they are replaced by virtual nodes
    // and should not be rendered or processed further.
    graph.nodes.retain(|n| !subgraph_ids.contains(&n.id));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tree_b08_nested() {
        // B08: graph TB; A-->B; subgraph S1; B-->C; subgraph S2; C-->D; end; end; D-->E
        let graph = trellis_parser::parse(
            "graph TB\n    A --> B\n    subgraph S1\n        B --> C\n        subgraph S2\n            C --> D\n        end\n    end\n    D --> E\n",
        ).unwrap();

        let tree = build_subgraph_tree_with_ownership(&graph);

        // ROOT should have 1 child: S1
        let root = tree.nodes.get("ROOT").unwrap();
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0], "S1");

        // ROOT direct nodes: A, E (not in any subgraph)
        assert!(root.direct_node_ids.contains(&"A".to_string()));
        assert!(root.direct_node_ids.contains(&"E".to_string()));
        assert!(!root.direct_node_ids.contains(&"B".to_string()));

        // S1 should have 1 child: S2
        let s1 = tree.nodes.get("S1").unwrap();
        assert_eq!(s1.children.len(), 1);
        assert_eq!(s1.children[0], "S2");

        // S1 direct nodes: B (C is in S2, not directly in S1)
        assert!(s1.direct_node_ids.contains(&"B".to_string()));
        assert!(
            !s1.direct_node_ids.contains(&"C".to_string()),
            "C should be in S2, not S1"
        );

        // S2 direct nodes: C, D
        let s2 = tree.nodes.get("S2").unwrap();
        assert!(s2.direct_node_ids.contains(&"C".to_string()));
        assert!(s2.direct_node_ids.contains(&"D".to_string()));
        assert!(s2.children.is_empty());
    }

    #[test]
    fn test_build_tree_b09() {
        // B09: graph TB; A-->S1; subgraph S1; B-->C; end; S1-->D
        let graph = trellis_parser::parse(
            "graph TB\n    A --> S1\n    subgraph S1\n        B --> C\n    end\n    S1 --> D\n",
        )
        .unwrap();

        let tree = build_subgraph_tree_with_ownership(&graph);

        let root = tree.nodes.get("ROOT").unwrap();
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0], "S1");

        // S1 direct nodes: B, C
        let s1 = tree.nodes.get("S1").unwrap();
        assert!(s1.direct_node_ids.contains(&"B".to_string()));
        assert!(s1.direct_node_ids.contains(&"C".to_string()));
    }

    #[test]
    fn test_place_nested_nodes_within_padding() {
        let graph = trellis_parser::parse(
            "graph TB\n    A --> B\n    subgraph S1\n        B --> C\n        subgraph S2\n            C --> D\n        end\n    end\n    D --> E\n",
        ).unwrap();

        let mut graph = graph.clone();
        let cell_size = 10;
        super::super::snap_node_dimensions_to_grid(&mut graph, cell_size);
        let (tree, boxes) = place_with_subgraphs(&mut graph, cell_size);

        // S2 should have a bounding box
        let s2_box = boxes.get("S2").expect("S2 should have a bounding box");
        // S1 should have a bounding box
        let s1_box = boxes.get("S1").expect("S1 should have a bounding box");

        // S2 bbox should be inside S1 bbox
        assert!(s2_box.x >= s1_box.x, "S2 left edge should be inside S1");
        assert!(s2_box.y >= s1_box.y, "S2 top edge should be inside S1");
        assert!(
            s2_box.x + s2_box.width <= s1_box.x + s1_box.width,
            "S2 right edge should be inside S1"
        );
        assert!(
            s2_box.y + s2_box.height <= s1_box.y + s1_box.height,
            "S2 bottom edge should be inside S1"
        );

        // All nodes in S2 should be within S2's bounding box
        let s2_tree = tree.nodes.get("S2").unwrap();
        for node_id in &s2_tree.direct_node_ids {
            let node = graph.nodes.iter().find(|n| n.id == *node_id).unwrap();
            assert!(
                node.x >= s2_box.x && node.x + node.width <= s2_box.x + s2_box.width,
                "Node {} x ({}) should be within S2 bbox ({} - {})",
                node_id,
                node.x,
                s2_box.x,
                s2_box.x + s2_box.width
            );
            assert!(
                node.y >= s2_box.y && node.y + node.height <= s2_box.y + s2_box.height,
                "Node {} y ({}) should be within S2 bbox ({} - {})",
                node_id,
                node.y,
                s2_box.y,
                s2_box.y + s2_box.height
            );
        }
    }

    #[test]
    fn test_resolve_subgraph_edges_creates_virtual_nodes() {
        let mut graph = trellis_parser::parse(
            "graph TB\n    A --> S1\n    subgraph S1\n        B --> C\n    end\n    S1 --> D\n",
        )
        .unwrap();

        let cell_size = 10;
        super::super::snap_node_dimensions_to_grid(&mut graph, cell_size);
        let (_tree, boxes) = place_with_subgraphs(&mut graph, cell_size);

        let initial_node_count = graph.nodes.len();

        resolve_subgraph_edges(&mut graph, &boxes);

        // Should have created a virtual node for S1
        let virtual_id = format!("{}S1", VIRTUAL_PREFIX);
        let virtual_node = graph.nodes.iter().find(|n| n.id == virtual_id);
        assert!(
            virtual_node.is_some(),
            "Virtual node for S1 should be created"
        );

        // Virtual node added (+1) and original S1 node removed (-1) = same count
        assert_eq!(graph.nodes.len(), initial_node_count);

        // Edges should now reference the virtual node
        let edge_a_s1 = graph.edges.iter().find(|e| e.from == "A");
        assert!(edge_a_s1.is_some());
        assert_eq!(edge_a_s1.unwrap().to, virtual_id);

        let edge_s1_d = graph.edges.iter().find(|e| e.to == "D");
        assert!(edge_s1_d.is_some());
        assert_eq!(edge_s1_d.unwrap().from, virtual_id);
    }
}
