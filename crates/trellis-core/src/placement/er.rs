use std::collections::HashMap;

use trellis_parser::{Direction, Graph};

use super::sugiyama;
use super::NODE_SPACING;

/// Line height for ER attribute rows (pixels)
pub const ER_LINE_HEIGHT: f64 = 18.0;
/// Header compartment height (entity name)
pub const ER_HEADER_HEIGHT: f64 = 30.0;
/// Minimum node width
const ER_MIN_WIDTH: f64 = 100.0;
/// Approximate character width for width estimation
const CHAR_WIDTH: f64 = 7.5;
/// Horizontal padding inside the box
const PADDING_X: f64 = 20.0;
/// Vertical gap between Sugiyama layers
const LAYER_GAP: f64 = 100.0;
/// Margin from the SVG edge to the first node
const MARGIN: f64 = 50.0;

/// Place all ER entity nodes using the Sugiyama hierarchical layout (TB direction).
///
/// Layer assignment and within-layer ordering come from the standard Sugiyama phases.
/// Coordinate assignment is custom: each node's x is anchored to the mean x of its
/// parents in the previous layer, then a left-to-right sweep resolves overlaps.
/// This keeps 1-to-1 connected pairs vertically aligned while spreading hub targets
/// outward from their shared parent.
pub fn place_er_diagram(graph: &mut Graph) {
    if graph.nodes.is_empty() {
        return;
    }

    // Calculate node sizes based on attribute count before placement
    for node in &mut graph.nodes {
        size_er_node(node);
    }

    graph.direction = Direction::TB;

    let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let node_index: HashMap<String, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect();

    let reversed = sugiyama::break_cycles(&node_ids, &node_index, &mut graph.edges);
    let layers = sugiyama::assign_layers(&node_ids, &node_index, &graph.edges);
    let order = sugiyama::order_within_layers(&node_ids, &node_index, &graph.edges, &layers);

    er_assign_coordinates(graph, &layers, &order);

    for idx in reversed {
        let edge = &mut graph.edges[idx];
        std::mem::swap(&mut edge.from, &mut edge.to);
    }
}

/// Assign (x, y) coordinates to ER nodes.
///
/// - Layer 0 nodes are packed left-to-right in barycenter order.
/// - Each subsequent layer's nodes are positioned at the mean x of their parents
///   in the previous layer, then a left-to-right sweep resolves overlaps.
/// - y positions are determined by layer index and maximum node height per layer.
fn er_assign_coordinates(
    graph: &mut Graph,
    layers: &HashMap<String, usize>,
    order: &HashMap<String, usize>,
) {
    let max_layer = match layers.values().copied().max() {
        Some(m) => m,
        None => return,
    };

    // Group node indices by layer, sorted by within-layer barycenter order.
    let mut by_layer: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for (i, node) in graph.nodes.iter().enumerate() {
        if let Some(&l) = layers.get(&node.id) {
            by_layer[l].push(i);
        }
    }
    for layer_nodes in &mut by_layer {
        layer_nodes.sort_by_key(|&i| order.get(&graph.nodes[i].id).copied().unwrap_or(0));
    }

    // Build backward adjacency: node → list of its parents (layer k-1 → layer k edges).
    let mut parents_of: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        if let (Some(&fl), Some(&tl)) = (layers.get(&edge.from), layers.get(&edge.to)) {
            if tl == fl + 1 {
                parents_of
                    .entry(edge.to.clone())
                    .or_default()
                    .push(edge.from.clone());
            }
        }
    }

    // Compute y position for each layer (TB: y increases downward).
    let mut layer_y: Vec<f64> = Vec::with_capacity(max_layer + 1);
    layer_y.push(MARGIN);
    for nodes in by_layer[..max_layer].iter() {
        let prev_max_h = nodes
            .iter()
            .map(|&i| graph.nodes[i].height)
            .fold(0.0_f64, f64::max);
        let prev = *layer_y.last().unwrap();
        layer_y.push(prev + prev_max_h + LAYER_GAP);
    }

    // Center-x for each node (by graph.nodes index).
    let mut cx: Vec<f64> = vec![0.0; graph.nodes.len()];

    // Layer 0: pack nodes left-to-right in barycenter order.
    let mut x = 0.0_f64;
    for &ni in &by_layer[0] {
        cx[ni] = x + graph.nodes[ni].width / 2.0;
        x += graph.nodes[ni].width + NODE_SPACING;
    }

    // Layers 1+: ideal x = mean of parent centers; spread overlaps left-to-right.
    for layer_nodes in by_layer[1..].iter() {
        // Snapshot parent cx values before modifying anything in this layer.
        let id_to_cx: HashMap<String, f64> = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), cx[i]))
            .collect();

        // Compute ideal center x for each node in this layer.
        let mut ideal: Vec<(usize, f64)> = layer_nodes
            .iter()
            .map(|&ni| {
                let id = &graph.nodes[ni].id;
                let ideal_x = parents_of
                    .get(id)
                    .map(|ps| {
                        let xs: Vec<f64> = ps
                            .iter()
                            .filter_map(|p| id_to_cx.get(p.as_str()).copied())
                            .collect();
                        if xs.is_empty() {
                            // No known parent positions → place at the far right.
                            f64::INFINITY
                        } else {
                            xs.iter().sum::<f64>() / xs.len() as f64
                        }
                    })
                    // Node with no edges → place at the far right.
                    .unwrap_or(f64::INFINITY);
                (ni, ideal_x)
            })
            .collect();

        // Sort by ideal x so the left-to-right sweep is well-defined.
        ideal.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Left-to-right sweep: push any node right if it would overlap the previous.
        // `min_cx` tracks the minimum allowed center-x for the next node.
        // Starting at NEG_INFINITY means the first node keeps its ideal position.
        // Nodes with no known parent positions (ideal_x == INFINITY) are placed at
        // the current right edge so they don't produce non-finite coordinates.
        let mut min_cx = f64::NEG_INFINITY;
        for (ni, ideal_x) in &mut ideal {
            let half_w = graph.nodes[*ni].width / 2.0;
            let clamped = if ideal_x.is_finite() {
                ideal_x.max(min_cx + half_w)
            } else {
                // No parent positions known — place after all previously placed nodes.
                (min_cx + half_w).max(0.0)
            };
            *ideal_x = clamped;
            min_cx = clamped + half_w + NODE_SPACING;
        }

        for (ni, final_cx) in ideal {
            cx[ni] = final_cx;
        }
    }

    // Shift the whole layout right so the leftmost node's left edge lands at MARGIN.
    let min_left = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| cx[i] - n.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    let x_shift = MARGIN - min_left;

    // Write back top-left (x, y) for every node.
    for (i, node) in graph.nodes.iter_mut().enumerate() {
        let l = *layers.get(&node.id).unwrap_or(&0);
        node.x = cx[i] + x_shift - node.width / 2.0;
        node.y = layer_y[l];
    }
}

/// Calculate the pixel dimensions of an ER entity node based on its label
/// and attribute list.
fn size_er_node(node: &mut trellis_parser::Node) {
    let name_width = node.label.len() as f64 * CHAR_WIDTH + PADDING_X * 2.0;
    let max_attr_width = node
        .er_attributes
        .iter()
        .map(|a| {
            // Width estimate: key indicator (3 chars) + type + space + name + padding
            let key_chars = if a.keys.is_empty() { 0 } else { 4 };
            let text_len = key_chars + a.attr_type.len() + 1 + a.name.len();
            text_len as f64 * CHAR_WIDTH + PADDING_X * 2.0
        })
        .fold(0.0_f64, f64::max);

    node.width = name_width.max(max_attr_width).max(ER_MIN_WIDTH);
    node.height = ER_HEADER_HEIGHT + node.er_attributes.len() as f64 * ER_LINE_HEIGHT + 4.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{parse, DiagramType};

    #[test]
    fn test_place_er_diagram_sets_coordinates() {
        let mut graph = parse(
            "erDiagram\n    USER ||--o{ ORDER : places\n    ORDER ||--|{ LINE-ITEM : contains\n",
        )
        .expect("parse failed");
        assert_eq!(graph.diagram_type, DiagramType::ErDiagram);

        place_er_diagram(&mut graph);

        for node in &graph.nodes {
            // After placement, all nodes should have non-negative coordinates
            assert!(node.width > 0.0, "node {} has zero width", node.id);
            assert!(node.height > 0.0, "node {} has zero height", node.id);
        }
    }

    #[test]
    fn test_er_node_sizing_with_attrs() {
        let mut graph =
            parse("erDiagram\n    USER {\n        int id PK\n        string email UK\n    }\n")
                .expect("parse failed");
        place_er_diagram(&mut graph);
        let user = graph.nodes.iter().find(|n| n.id == "USER").unwrap();
        // height = HEADER (30) + 2 * LINE_HEIGHT (18) + 4 = 70
        assert!((user.height - (ER_HEADER_HEIGHT + 2.0 * ER_LINE_HEIGHT + 4.0)).abs() < 5.0);
    }

    #[test]
    fn test_one_to_one_pairs_are_vertically_aligned() {
        // EO_SRC → EO_TGT should end up at the same x (within 1px)
        let mut graph = parse(
            "erDiagram\n    EO_SRC ||--|| EO_TGT : \"pair\"\n    OTHER_SRC ||--|| OTHER_TGT : \"pair2\"\n",
        )
        .expect("parse failed");
        place_er_diagram(&mut graph);

        let src = graph.nodes.iter().find(|n| n.id == "EO_SRC").unwrap();
        let tgt = graph.nodes.iter().find(|n| n.id == "EO_TGT").unwrap();
        let src_cx = src.x + src.width / 2.0;
        let tgt_cx = tgt.x + tgt.width / 2.0;
        assert!(
            (src_cx - tgt_cx).abs() < 1.0,
            "1-to-1 pair should be vertically aligned: src_cx={src_cx}, tgt_cx={tgt_cx}"
        );
    }
}
