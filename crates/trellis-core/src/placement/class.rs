use std::collections::{HashMap, HashSet};
use trellis_parser::{ClassEdgeType, Graph};

use super::algorithm::LayoutAlgorithm;
use super::sugiyama::SugiyamaLayout;
use super::{overlap, LAYER_SPACING, NODE_SPACING};

/// Place all nodes in a class diagram using a hybrid algorithm:
///
/// 1. Build an "inheritance graph" from Inheritance and Realization edges.
/// 2. Run Sugiyama on the inheritance graph → assigns (x, y) to those nodes.
/// 3. Place remaining nodes laterally next to their most-connected placed neighbour.
/// 4. Resolve overlaps with spiral search.
pub fn place_class_diagram(graph: &mut Graph) {
    if graph.nodes.is_empty() {
        return;
    }

    // Collect inheritance / realization edge pairs
    let inherit_edge_pairs: Vec<(String, String)> = graph
        .edges
        .iter()
        .filter(|e| {
            matches!(
                e.class_edge_type,
                Some(ClassEdgeType::Inheritance) | Some(ClassEdgeType::Realization)
            )
        })
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    // Build set of nodes that participate in inheritance hierarchy
    let inheritance_nodes: HashSet<String> = inherit_edge_pairs
        .iter()
        .flat_map(|(f, t)| [f.clone(), t.clone()])
        .collect();

    if !inheritance_nodes.is_empty() {
        // Run Sugiyama using only inheritance/realization edges
        let original_edges = graph.edges.clone();
        graph.edges.retain(|e| {
            matches!(
                e.class_edge_type,
                Some(ClassEdgeType::Inheritance) | Some(ClassEdgeType::Realization)
            )
        });
        SugiyamaLayout.layout(graph);
        graph.edges = original_edges;
    }

    // After Sugiyama, record which nodes have been placed
    let mut placed: HashSet<String> = inheritance_nodes.clone();

    // Build adjacency for all edges (for lateral placement of unplaced nodes)
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        adj.entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }

    // Build coordinate and size maps (using centre coordinates)
    let mut coords: HashMap<String, (f64, f64)> = graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), (n.x + n.width / 2.0, n.y + n.height / 2.0)))
        .collect();
    let sizes: HashMap<String, (f64, f64)> =
        graph.nodes.iter().map(|n| (n.id.clone(), (n.width, n.height))).collect();

    let all_node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();

    // Iteratively place unplaced nodes next to their most-connected placed neighbour
    let mut made_progress = true;
    while made_progress {
        made_progress = false;
        for node_id in &all_node_ids {
            if placed.contains(node_id) {
                continue;
            }
            if let Some(best) =
                find_most_connected_placed_neighbour(node_id, &adj, &placed)
            {
                let (bx, by) = coords[&best];
                let (bw, _bh) = sizes[&best];
                let (my_w, my_h) = sizes[node_id];

                // Place to the right of the best neighbour
                let candidate_x = bx + bw / 2.0 + NODE_SPACING + my_w / 2.0;
                let candidate_y = by;

                let pos = overlap::find_free_position(
                    candidate_x,
                    candidate_y,
                    my_w,
                    my_h,
                    &coords,
                    &sizes,
                );
                coords.insert(node_id.clone(), pos);
                placed.insert(node_id.clone());
                made_progress = true;
            }
        }
    }

    // Place any remaining completely-disconnected nodes in a row below everything
    let max_y = coords.values().map(|(_, y)| *y).fold(0.0_f64, f64::max);
    let base_y = if placed.is_empty() { 0.0 } else { max_y + LAYER_SPACING };
    let mut x_cursor = 0.0_f64;

    for node_id in &all_node_ids {
        if !placed.contains(node_id) {
            let (my_w, my_h) = sizes[node_id];
            let candidate_x = x_cursor + my_w / 2.0;
            let candidate_y = base_y + my_h / 2.0;

            let pos = overlap::find_free_position(
                candidate_x,
                candidate_y,
                my_w,
                my_h,
                &coords,
                &sizes,
            );
            coords.insert(node_id.clone(), pos);
            placed.insert(node_id.clone());
            x_cursor = pos.0 + my_w / 2.0 + NODE_SPACING;
        }
    }

    // Write centre coordinates back to graph nodes as top-left (x, y)
    for node in &mut graph.nodes {
        if let Some((cx, cy)) = coords.get(&node.id) {
            node.x = cx - node.width / 2.0;
            node.y = cy - node.height / 2.0;
        }
    }
}

/// Find the neighbour of `node_id` in `adj` that is already placed
/// and itself has the most connections to placed nodes.
fn find_most_connected_placed_neighbour(
    node_id: &str,
    adj: &HashMap<String, Vec<String>>,
    placed: &HashSet<String>,
) -> Option<String> {
    let neighbours = adj.get(node_id)?;
    let placed_neighbours: Vec<&String> =
        neighbours.iter().filter(|n| placed.contains(*n)).collect();
    if placed_neighbours.is_empty() {
        return None;
    }
    placed_neighbours
        .into_iter()
        .max_by_key(|n| {
            adj.get(*n)
                .map(|ns| ns.iter().filter(|nn| placed.contains(*nn)).count())
                .unwrap_or(0)
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{ClassEdgeType, DiagramType, Edge, EdgeStyle, Graph, Node, NodeShape};

    fn make_class_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::ClassBox,
            width: 120.0,
            height: 80.0,
            ..Default::default()
        }
    }

    fn make_inherit_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            class_edge_type: Some(ClassEdgeType::Inheritance),
            style: EdgeStyle::Solid,
            ..Default::default()
        }
    }

    #[test]
    fn test_place_simple_hierarchy() {
        let mut graph = Graph {
            nodes: vec![
                make_class_node("Animal"),
                make_class_node("Dog"),
                make_class_node("Cat"),
            ],
            edges: vec![
                make_inherit_edge("Animal", "Dog"),
                make_inherit_edge("Animal", "Cat"),
            ],
            diagram_type: DiagramType::ClassDiagram,
            ..Default::default()
        };

        place_class_diagram(&mut graph);

        for node in &graph.nodes {
            assert!(node.x.is_finite(), "Node {} has non-finite x", node.id);
            assert!(node.y.is_finite(), "Node {} has non-finite y", node.id);
        }
    }

    #[test]
    fn test_place_disconnected_nodes() {
        let mut graph = Graph {
            nodes: vec![
                make_class_node("A"),
                make_class_node("B"),
                make_class_node("C"),
            ],
            diagram_type: DiagramType::ClassDiagram,
            ..Default::default()
        };

        place_class_diagram(&mut graph);

        for node in &graph.nodes {
            assert!(node.x.is_finite());
            assert!(node.y.is_finite());
        }
    }

    #[test]
    fn test_no_overlap_after_placement() {
        let mut graph = Graph {
            nodes: vec![
                make_class_node("A"),
                make_class_node("B"),
                make_class_node("C"),
            ],
            diagram_type: DiagramType::ClassDiagram,
            ..Default::default()
        };
        place_class_diagram(&mut graph);

        for i in 0..graph.nodes.len() {
            for j in i + 1..graph.nodes.len() {
                let a = &graph.nodes[i];
                let b = &graph.nodes[j];
                let ax = a.x + a.width / 2.0;
                let ay = a.y + a.height / 2.0;
                let bx = b.x + b.width / 2.0;
                let by_ = b.y + b.height / 2.0;
                let overlap_x = (ax - bx).abs() < (a.width + b.width) / 2.0;
                let overlap_y = (ay - by_).abs() < (a.height + b.height) / 2.0;
                assert!(
                    !(overlap_x && overlap_y),
                    "Nodes {} and {} overlap",
                    a.id,
                    b.id
                );
            }
        }
    }
}
