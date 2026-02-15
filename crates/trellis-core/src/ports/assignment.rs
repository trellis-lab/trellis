use std::collections::HashMap;
use trellis_parser::{Graph, Node};

/// Side of a node where a port can be placed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

/// A port on a node's edge (connection point at an exact grid point)
#[derive(Debug, Clone)]
pub struct Port {
    pub x: f64,
    pub y: f64,
    pub grid_row: i64,
    pub grid_col: i64,
    pub side: Side,
}

/// Port assignments for an edge (source and target ports)
#[derive(Debug, Clone)]
pub struct EdgePorts {
    pub source_port: Port,
    pub target_port: Port,
}

/// An edge associated with a particular node, tracking which end this node is
#[derive(Debug, Clone)]
struct NodeEdgeInfo {
    edge_index: usize,
    angle_deg: f64,
    other_node_id: String,
    is_source: bool, // true if this node is the source of the edge
}

/// A connector: a grid point on a node boundary available for edge routing.
#[derive(Debug, Clone)]
struct Connector {
    grid_row: i64,
    grid_col: i64,
    x: f64,
    y: f64,
}

/// Convert an angle (degrees, 0=right, 90=down) to a side of a node.
///
/// Uses 4 sectors of 90 degrees each.
pub fn angle_to_side(angle_deg: f64) -> Side {
    let angle = ((angle_deg % 360.0) + 360.0) % 360.0;
    if !(45.0..315.0).contains(&angle) {
        Side::Right
    } else if (45.0..135.0).contains(&angle) {
        Side::Bottom
    } else if (135.0..225.0).contains(&angle) {
        Side::Left
    } else {
        Side::Top
    }
}

/// Enumerate the available connector grid points on a given side of a node.
///
/// Connectors are non-corner grid points on the node boundary.
/// Node positions are top-left, dimensions are grid-aligned.
fn enumerate_connectors(
    node: &Node,
    side: Side,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> Vec<Connector> {
    let cs = cell_size as f64;
    let ox = offset_x as f64;
    let oy = offset_y as f64;

    // Grid position of node top-left corner
    let gc = ((node.x - ox) / cs).round() as i64;
    let gr = ((node.y - oy) / cs).round() as i64;

    // Grid-point counts (N grid points = (N-1)*cs pixels)
    let w_points = (node.width / cs).round() as i64 + 1;
    let h_points = (node.height / cs).round() as i64 + 1;

    let mut connectors = Vec::new();

    match side {
        Side::Top => {
            // Non-corner points on the top edge: (gr, gc+1) through (gr, gc+w_points-2)
            for c in (gc + 1)..=(gc + w_points - 2) {
                connectors.push(Connector {
                    grid_row: gr,
                    grid_col: c,
                    x: c as f64 * cs + ox,
                    y: gr as f64 * cs + oy,
                });
            }
        }
        Side::Bottom => {
            let bottom_row = gr + h_points - 1;
            for c in (gc + 1)..=(gc + w_points - 2) {
                connectors.push(Connector {
                    grid_row: bottom_row,
                    grid_col: c,
                    x: c as f64 * cs + ox,
                    y: bottom_row as f64 * cs + oy,
                });
            }
        }
        Side::Left => {
            // Non-corner points on the left edge: (gr+1, gc) through (gr+h_points-2, gc)
            for r in (gr + 1)..=(gr + h_points - 2) {
                connectors.push(Connector {
                    grid_row: r,
                    grid_col: gc,
                    x: gc as f64 * cs + ox,
                    y: r as f64 * cs + oy,
                });
            }
        }
        Side::Right => {
            let right_col = gc + w_points - 1;
            for r in (gr + 1)..=(gr + h_points - 2) {
                connectors.push(Connector {
                    grid_row: r,
                    grid_col: right_col,
                    x: right_col as f64 * cs + ox,
                    y: r as f64 * cs + oy,
                });
            }
        }
    }

    connectors
}

/// Assign ports to all edges in the graph using discrete grid-point connectors.
///
/// For each node, edges are grouped by side (based on the angle to the connected node),
/// overflow is handled, edges are sorted within each side, and connector positions are assigned.
pub fn assign_ports(graph: &Graph, cell_size: i32, offset_x: i32, offset_y: i32) -> HashMap<usize, EdgePorts> {
    let mut port_assignments: HashMap<usize, EdgePorts> = HashMap::new();

    // Build a node lookup by id
    let node_map: HashMap<&str, &Node> = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();

    // Group edges by node
    let mut node_edges: HashMap<&str, Vec<NodeEdgeInfo>> = HashMap::new();
    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        if let (Some(source), Some(target)) = (node_map.get(edge.from.as_str()), node_map.get(edge.to.as_str())) {
            let angle_from_source = calculate_angle(source, target);
            node_edges
                .entry(edge.from.as_str())
                .or_default()
                .push(NodeEdgeInfo {
                    edge_index: edge_idx,
                    angle_deg: angle_from_source,
                    other_node_id: edge.to.clone(),
                    is_source: true,
                });

            let angle_from_target = calculate_angle(target, source);
            node_edges
                .entry(edge.to.as_str())
                .or_default()
                .push(NodeEdgeInfo {
                    edge_index: edge_idx,
                    angle_deg: angle_from_target,
                    other_node_id: edge.from.clone(),
                    is_source: false,
                });
        }
    }

    // For each node, assign ports
    for node in &graph.nodes {
        let edges = match node_edges.get(node.id.as_str()) {
            Some(e) => e,
            None => continue,
        };

        // Group by side
        let mut sides: HashMap<Side, Vec<&NodeEdgeInfo>> = HashMap::new();
        for info in edges {
            let side = angle_to_side(info.angle_deg);
            sides.entry(side).or_default().push(info);
        }

        // Handle overflow using connector counts
        handle_overflow(&mut sides, node, cell_size, offset_x, offset_y);

        // Sort edges on each side
        for (&side, edge_list) in sides.iter_mut() {
            sort_edges_on_side(edge_list, side, &node_map);
        }

        // Assign connector positions to each edge on each side
        for (&side, edge_list) in &sides {
            let n_edges = edge_list.len();
            if n_edges == 0 {
                continue;
            }

            let connectors = enumerate_connectors(node, side, cell_size, offset_x, offset_y);
            let n_connectors = connectors.len();

            if n_connectors == 0 {
                continue;
            }

            for (i, info) in edge_list.iter().enumerate() {
                // Evenly distribute edges among available connectors
                let connector_idx = if n_edges == 1 {
                    n_connectors / 2 // center connector for single edge
                } else {
                    // Spread evenly: fraction = (i+1) / (n_edges+1), index = fraction * n_connectors
                    let fraction = (i as f64 + 1.0) / (n_edges as f64 + 1.0);
                    ((fraction * n_connectors as f64).round() as usize).min(n_connectors - 1)
                };

                let conn = &connectors[connector_idx];
                let port = Port {
                    x: conn.x,
                    y: conn.y,
                    grid_row: conn.grid_row,
                    grid_col: conn.grid_col,
                    side,
                };

                let entry = port_assignments
                    .entry(info.edge_index)
                    .or_insert_with(|| EdgePorts {
                        source_port: Port {
                            x: 0.0,
                            y: 0.0,
                            grid_row: 0,
                            grid_col: 0,
                            side: Side::Top,
                        },
                        target_port: Port {
                            x: 0.0,
                            y: 0.0,
                            grid_row: 0,
                            grid_col: 0,
                            side: Side::Top,
                        },
                    });

                if info.is_source {
                    entry.source_port = port;
                } else {
                    entry.target_port = port;
                }
            }
        }
    }

    port_assignments
}

/// Calculate the angle in degrees from one node center to another.
/// 0 degrees = right, 90 degrees = down (screen coordinates).
/// Node positions are top-left, so centers are computed as (x + w/2, y + h/2).
fn calculate_angle(from: &Node, to: &Node) -> f64 {
    let from_cx = from.x + from.width / 2.0;
    let from_cy = from.y + from.height / 2.0;
    let to_cx = to.x + to.width / 2.0;
    let to_cy = to.y + to.height / 2.0;
    let dx = to_cx - from_cx;
    let dy = to_cy - from_cy;
    let rad = dy.atan2(dx);
    let deg = rad.to_degrees();
    ((deg % 360.0) + 360.0) % 360.0
}

/// Handle overflow: if too many edges are assigned to one side, move excess to adjacent sides.
/// Uses connector count (grid-point based) instead of pixel-based spacing.
fn handle_overflow(
    sides: &mut HashMap<Side, Vec<&NodeEdgeInfo>>,
    node: &Node,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) {
    let max_connectors = |side: Side| -> usize {
        enumerate_connectors(node, side, cell_size, offset_x, offset_y).len()
    };

    let side_order = [Side::Top, Side::Right, Side::Bottom, Side::Left];

    for &side in &side_order {
        let max = max_connectors(side).max(1);
        while sides.get(&side).map_or(0, |v| v.len()) > max {
            let edges = sides.get(&side).unwrap();
            let boundary_edge_idx = find_edge_closest_to_boundary(edges, side);

            let edge_info = sides.get_mut(&side).unwrap().remove(boundary_edge_idx);
            let neighbor = clockwise_neighbor(side);
            sides.entry(neighbor).or_default().push(edge_info);
        }
    }
}

/// Find the index of the edge whose angle is closest to the boundary of the given side.
fn find_edge_closest_to_boundary(edges: &[&NodeEdgeInfo], side: Side) -> usize {
    let (low_boundary, high_boundary) = match side {
        Side::Right => (315.0, 45.0),
        Side::Bottom => (45.0, 135.0),
        Side::Left => (135.0, 225.0),
        Side::Top => (225.0, 315.0),
    };

    edges
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let dist_a = min_angle_distance_to_boundary(a.angle_deg, low_boundary, high_boundary);
            let dist_b = min_angle_distance_to_boundary(b.angle_deg, low_boundary, high_boundary);
            dist_a.partial_cmp(&dist_b).unwrap().reverse()
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn min_angle_distance_to_boundary(angle: f64, low: f64, high: f64) -> f64 {
    let d1 = angle_distance(angle, low);
    let d2 = angle_distance(angle, high);
    d1.min(d2)
}

fn angle_distance(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs();
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Get the clockwise neighbor side
fn clockwise_neighbor(side: Side) -> Side {
    match side {
        Side::Top => Side::Right,
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
    }
}

/// Sort edges on a given side by the perpendicular axis center position of the other node.
fn sort_edges_on_side(edges: &mut Vec<&NodeEdgeInfo>, side: Side, node_map: &HashMap<&str, &Node>) {
    match side {
        Side::Top | Side::Bottom => {
            // Sort by other node's center x coordinate (left to right)
            edges.sort_by(|a, b| {
                let ax = node_map.get(a.other_node_id.as_str()).map(|n| n.x + n.width / 2.0).unwrap_or(0.0);
                let bx = node_map.get(b.other_node_id.as_str()).map(|n| n.x + n.width / 2.0).unwrap_or(0.0);
                ax.partial_cmp(&bx).unwrap()
            });
        }
        Side::Left | Side::Right => {
            // Sort by other node's center y coordinate (top to bottom)
            edges.sort_by(|a, b| {
                let ay = node_map.get(a.other_node_id.as_str()).map(|n| n.y + n.height / 2.0).unwrap_or(0.0);
                let by = node_map.get(b.other_node_id.as_str()).map(|n| n.y + n.height / 2.0).unwrap_or(0.0);
                ay.partial_cmp(&by).unwrap()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Edge, EdgeStyle, ArrowHead, NodeShape};

    fn make_node(id: &str, w: f64, h: f64, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: w,
            height: h,
            x,
            y,
        }
    }

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
        }
    }

    #[test]
    fn test_angle_to_side_right() {
        assert_eq!(angle_to_side(0.0), Side::Right);
        assert_eq!(angle_to_side(44.9), Side::Right);
        assert_eq!(angle_to_side(350.0), Side::Right);
    }

    #[test]
    fn test_angle_to_side_bottom() {
        assert_eq!(angle_to_side(45.0), Side::Bottom);
        assert_eq!(angle_to_side(90.0), Side::Bottom);
        assert_eq!(angle_to_side(134.9), Side::Bottom);
    }

    #[test]
    fn test_angle_to_side_left() {
        assert_eq!(angle_to_side(135.0), Side::Left);
        assert_eq!(angle_to_side(180.0), Side::Left);
        assert_eq!(angle_to_side(224.9), Side::Left);
    }

    #[test]
    fn test_angle_to_side_top() {
        assert_eq!(angle_to_side(225.0), Side::Top);
        assert_eq!(angle_to_side(270.0), Side::Top);
        assert_eq!(angle_to_side(314.9), Side::Top);
    }

    #[test]
    fn test_enumerate_connectors_top() {
        // Node at top-left (0,0), 40x20 pixels, cell_size=10
        // w_points = 40/10 + 1 = 5, h_points = 20/10 + 1 = 3
        // Top connectors: cols 1,2,3 → 3 connectors
        let node = make_node("A", 40.0, 20.0, 0.0, 0.0);
        let connectors = enumerate_connectors(&node, Side::Top, 10, 0, 0);
        assert_eq!(connectors.len(), 3);
        assert_eq!(connectors[0].grid_col, 1);
        assert_eq!(connectors[1].grid_col, 2);
        assert_eq!(connectors[2].grid_col, 3);
        assert_eq!(connectors[0].grid_row, 0);
    }

    #[test]
    fn test_enumerate_connectors_left() {
        // Node at (0,0), 40x20, cell_size=10
        // h_points = 3, left connectors: rows 1 → 1 connector
        let node = make_node("A", 40.0, 20.0, 0.0, 0.0);
        let connectors = enumerate_connectors(&node, Side::Left, 10, 0, 0);
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0].grid_row, 1);
        assert_eq!(connectors[0].grid_col, 0);
    }

    #[test]
    fn test_enumerate_connectors_with_offset() {
        // Node at (50,30), 40x20, cell_size=10, offset (0,0)
        // gc = 5, gr = 3, w_points = 5, h_points = 3
        let node = make_node("A", 40.0, 20.0, 50.0, 30.0);
        let connectors = enumerate_connectors(&node, Side::Bottom, 10, 0, 0);
        assert_eq!(connectors.len(), 3);
        // Bottom row = 3 + 3 - 1 = 5
        assert_eq!(connectors[0].grid_row, 5);
        assert_eq!(connectors[0].grid_col, 6); // gc+1 = 6
        assert_eq!(connectors[0].x, 60.0);
        assert_eq!(connectors[0].y, 50.0);
    }

    #[test]
    fn test_assign_ports_simple_vertical() {
        // A above B (TB layout), top-left coordinates, grid-aligned dimensions
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 60.0, 30.0),
            make_node("B", 40.0, 20.0, 60.0, 130.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let ports = assign_ports(&graph, 10, 0, 0);
        assert_eq!(ports.len(), 1);

        let edge_ports = &ports[&0];
        // A→B: A's port should be on bottom, B's port on top
        assert_eq!(edge_ports.source_port.side, Side::Bottom);
        assert_eq!(edge_ports.target_port.side, Side::Top);
        // Port should be at exact grid point
        assert_eq!(edge_ports.source_port.x % 10.0, 0.0);
        assert_eq!(edge_ports.source_port.y % 10.0, 0.0);
    }

    #[test]
    fn test_assign_ports_horizontal() {
        // A left of B, top-left coordinates
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 10.0, 80.0),
            make_node("B", 40.0, 20.0, 210.0, 80.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let ports = assign_ports(&graph, 10, 0, 0);
        let edge_ports = &ports[&0];
        assert_eq!(edge_ports.source_port.side, Side::Right);
        assert_eq!(edge_ports.target_port.side, Side::Left);
    }

    #[test]
    fn test_port_symmetry_for_opposite_edges() {
        // A in middle, B above, C below → symmetrical ports (top-left coords)
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 60.0, 80.0),
            make_node("B", 40.0, 20.0, 60.0, 0.0),
            make_node("C", 40.0, 20.0, 60.0, 180.0),
        ];
        graph.edges = vec![make_edge("A", "B"), make_edge("A", "C")];

        let ports = assign_ports(&graph, 10, 0, 0);

        let ab = &ports[&0];
        assert_eq!(ab.source_port.side, Side::Top);

        let ac = &ports[&1];
        assert_eq!(ac.source_port.side, Side::Bottom);

        // Both source ports should have the same x (center connector)
        assert!((ab.source_port.x - ac.source_port.x).abs() < 0.01);
    }

    #[test]
    fn test_multiple_edges_same_side() {
        // A above, B1/B2/B3 below at different x positions (top-left coords)
        // A is wider (80px = 9 grid points wide, 7 top/bottom connectors)
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 40.0, 30.0),
            make_node("B1", 40.0, 20.0, 20.0, 180.0),
            make_node("B2", 40.0, 20.0, 70.0, 180.0),
            make_node("B3", 40.0, 20.0, 120.0, 180.0),
        ];
        graph.edges = vec![
            make_edge("A", "B1"),
            make_edge("A", "B2"),
            make_edge("A", "B3"),
        ];

        let ports = assign_ports(&graph, 10, 0, 0);

        // All source ports should be on A's bottom
        for i in 0..3 {
            assert_eq!(ports[&i].source_port.side, Side::Bottom);
        }

        // Ports should be ordered left to right
        let x0 = ports[&0].source_port.x;
        let x1 = ports[&1].source_port.x;
        let x2 = ports[&2].source_port.x;
        assert!(x0 < x1, "Port for B1 should be left of B2: {} < {}", x0, x1);
        assert!(x1 < x2, "Port for B2 should be left of B3: {} < {}", x1, x2);

        // All ports should be at exact grid points
        assert_eq!(x0 % 10.0, 0.0);
        assert_eq!(x1 % 10.0, 0.0);
        assert_eq!(x2 % 10.0, 0.0);
    }
}
