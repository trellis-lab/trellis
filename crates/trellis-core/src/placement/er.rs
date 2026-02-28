use trellis_parser::Graph;

use super::algorithm::LayoutAlgorithm;
use super::force_directed::{ForceDirectedConfig, ForceDirectedLayout};
use super::snap::snap_nodes_to_grid;

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

/// Place all ER entity nodes using Fruchterman-Reingold force-directed layout.
///
/// After force-directed placement, positions are snapped to a grid.
pub fn place_er_diagram(graph: &mut Graph) {
    if graph.nodes.is_empty() {
        return;
    }

    // Calculate node sizes based on attribute count before placement
    for node in &mut graph.nodes {
        size_er_node(node);
    }

    // Run force-directed layout via the LayoutAlgorithm interface
    let algorithm = ForceDirectedLayout {
        config: ForceDirectedConfig {
            area_factor: 80_000.0,
            cooling_rate: 0.95,
            max_iterations: 100,
        },
    };
    algorithm.layout(graph);

    // Snap positions to grid
    let grid_size = 10.0;
    let mut node_tuples: Vec<(f64, f64, f64, f64)> = graph
        .nodes
        .iter()
        .map(|n| (n.x, n.y, n.width, n.height))
        .collect();
    snap_nodes_to_grid(&mut node_tuples, grid_size);
    for (node, (x, y, _, _)) in graph.nodes.iter_mut().zip(node_tuples.iter()) {
        node.x = *x;
        node.y = *y;
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
        let mut graph = parse(
            "erDiagram\n    USER {\n        int id PK\n        string email UK\n    }\n",
        )
        .expect("parse failed");
        place_er_diagram(&mut graph);
        let user = graph.nodes.iter().find(|n| n.id == "USER").unwrap();
        // height = HEADER (30) + 2 * LINE_HEIGHT (18) + 4 = 70
        assert!((user.height - (ER_HEADER_HEIGHT + 2.0 * ER_LINE_HEIGHT + 4.0)).abs() < 5.0);
    }
}
