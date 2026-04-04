use std::collections::HashMap;
use trellis_parser::Node;

use super::assignment::Side;

/// An edge associated with a particular node, tracking which end this node is.
#[derive(Debug, Clone)]
pub struct NodeEdgeInfo {
    pub edge_index: usize,
    pub angle_deg: f64,
    pub other_node_id: String,
    pub is_source: bool,
}

/// A connector: a grid point on a node boundary available for edge routing.
#[derive(Debug, Clone)]
pub struct Connector {
    pub grid_row: i64,
    pub grid_col: i64,
    pub x: f64,
    pub y: f64,
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

/// Calculate the angle in degrees from one node center to another.
/// 0 degrees = right, 90 degrees = down (screen coordinates).
pub fn calculate_angle(from: &Node, to: &Node) -> f64 {
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

/// Enumerate the available connector grid points on a given side of a node.
///
/// Connectors are non-corner grid points on the node boundary.
pub fn enumerate_connectors(
    node: &Node,
    side: Side,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> Vec<Connector> {
    let cs = cell_size as f64;
    let ox = offset_x as f64;
    let oy = offset_y as f64;

    let gc = ((node.x - ox) / cs).round() as i64;
    let gr = ((node.y - oy) / cs).round() as i64;

    let w_points = (node.width / cs).round() as i64 + 1;
    let h_points = (node.height / cs).round() as i64 + 1;

    let mut connectors = Vec::new();

    match side {
        Side::Top => {
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

/// Handle overflow: if too many edges are assigned to one side, move excess to adjacent sides.
pub fn handle_overflow(
    sides: &mut HashMap<Side, Vec<NodeEdgeInfo>>,
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

/// Sort edges on a given side by the perpendicular axis center position of the other node.
pub fn sort_edges_on_side(
    edges: &mut [NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &Node>,
) {
    match side {
        Side::Top | Side::Bottom => {
            edges.sort_by(|a, b| {
                let ax = node_map
                    .get(a.other_node_id.as_str())
                    .map(|n| n.x + n.width / 2.0)
                    .unwrap_or(0.0);
                let bx = node_map
                    .get(b.other_node_id.as_str())
                    .map(|n| n.x + n.width / 2.0)
                    .unwrap_or(0.0);
                ax.partial_cmp(&bx).unwrap()
            });
        }
        Side::Left | Side::Right => {
            edges.sort_by(|a, b| {
                let ay = node_map
                    .get(a.other_node_id.as_str())
                    .map(|n| n.y + n.height / 2.0)
                    .unwrap_or(0.0);
                let by = node_map
                    .get(b.other_node_id.as_str())
                    .map(|n| n.y + n.height / 2.0)
                    .unwrap_or(0.0);
                ay.partial_cmp(&by).unwrap()
            });
        }
    }
}

/// Build the node lookup, edge grouping, and side assignment that all algorithms share.
///
/// Returns `(node_map, per_node_sides)` where `per_node_sides` maps each node id
/// to a side→edges mapping.
#[allow(clippy::type_complexity)]
pub fn build_edge_side_map(
    graph: &trellis_parser::Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> (
    HashMap<&str, &Node>,
    HashMap<&str, HashMap<Side, Vec<NodeEdgeInfo>>>,
) {
    let node_map: HashMap<&str, &Node> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Group edges by node
    let mut node_edges: HashMap<&str, Vec<NodeEdgeInfo>> = HashMap::new();
    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        if let (Some(source), Some(target)) = (
            node_map.get(edge.from.as_str()),
            node_map.get(edge.to.as_str()),
        ) {
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

    // For each node, group by side and handle overflow
    let mut per_node_sides: HashMap<&str, HashMap<Side, Vec<NodeEdgeInfo>>> = HashMap::new();
    for node in &graph.nodes {
        let edges = match node_edges.remove(node.id.as_str()) {
            Some(e) => e,
            None => continue,
        };

        let mut sides: HashMap<Side, Vec<NodeEdgeInfo>> = HashMap::new();
        for info in edges {
            let side = angle_to_side(info.angle_deg);
            sides.entry(side).or_default().push(info);
        }

        handle_overflow(&mut sides, node, cell_size, offset_x, offset_y);
        per_node_sides.insert(node.id.as_str(), sides);
    }

    (node_map, per_node_sides)
}

/// Assign connector positions to edges on each side of a node, producing port entries.
///
/// This is the final step shared by all algorithms: given sorted edges per side,
/// distribute them evenly among the available connectors.
pub fn assign_connectors(
    node: &Node,
    sides: &HashMap<Side, Vec<NodeEdgeInfo>>,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
    port_assignments: &mut HashMap<usize, super::EdgePorts>,
) {
    for (&side, edge_list) in sides {
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
            let connector_idx = if n_edges == 1 {
                n_connectors / 2
            } else {
                let fraction = (i as f64 + 1.0) / (n_edges as f64 + 1.0);
                ((fraction * n_connectors as f64).round() as usize).min(n_connectors - 1)
            };

            let conn = &connectors[connector_idx];
            let port = super::Port {
                x: conn.x,
                y: conn.y,
                grid_row: conn.grid_row,
                grid_col: conn.grid_col,
                side,
            };

            let entry =
                port_assignments
                    .entry(info.edge_index)
                    .or_insert_with(|| super::EdgePorts {
                        source_port: super::Port {
                            x: 0.0,
                            y: 0.0,
                            grid_row: 0,
                            grid_col: 0,
                            side: Side::Top,
                        },
                        target_port: super::Port {
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

/// Check if two edges on the same side would cross based on target positions.
///
/// Two edges cross if their source port order (left-to-right or top-to-bottom)
/// is inverted relative to their target node positions.
pub fn would_cross(
    edge_a: &NodeEdgeInfo,
    edge_b: &NodeEdgeInfo,
    side: Side,
    node_map: &HashMap<&str, &Node>,
) -> bool {
    let (a_pos, b_pos) = match side {
        Side::Top | Side::Bottom => {
            let ax = node_map
                .get(edge_a.other_node_id.as_str())
                .map(|n| n.x + n.width / 2.0)
                .unwrap_or(0.0);
            let bx = node_map
                .get(edge_b.other_node_id.as_str())
                .map(|n| n.x + n.width / 2.0)
                .unwrap_or(0.0);
            (ax, bx)
        }
        Side::Left | Side::Right => {
            let ay = node_map
                .get(edge_a.other_node_id.as_str())
                .map(|n| n.y + n.height / 2.0)
                .unwrap_or(0.0);
            let by = node_map
                .get(edge_b.other_node_id.as_str())
                .map(|n| n.y + n.height / 2.0)
                .unwrap_or(0.0);
            (ay, by)
        }
    };

    // Edges are in order (a before b in port list).
    // They cross if target positions are inverted.
    a_pos > b_pos
}

/// Count inversions (potential crossings) for a set of edges on a side.
pub fn count_inversions(
    edges: &[NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &Node>,
) -> usize {
    let mut count = 0;
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            if would_cross(&edges[i], &edges[j], side, node_map) {
                count += 1;
            }
        }
    }
    count
}

/// Rebalance edges across sides to reduce congestion on overloaded sides.
///
/// Moves borderline edges from congested sides (>threshold_high) to
/// adjacent sides with low congestion (<threshold_low).
pub fn rebalance_sides(
    sides: &mut HashMap<Side, Vec<NodeEdgeInfo>>,
    node: &Node,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
    threshold_high: f64,
    threshold_low: f64,
) {
    let side_order = [Side::Top, Side::Right, Side::Bottom, Side::Left];

    let congestion = |s: Side, sides: &HashMap<Side, Vec<NodeEdgeInfo>>| -> f64 {
        let capacity = enumerate_connectors(node, s, cell_size, offset_x, offset_y).len();
        let load = sides.get(&s).map_or(0, |v| v.len());
        load as f64 / capacity.max(1) as f64
    };

    // Single pass: try to move one edge from each congested side
    for &side in &side_order {
        if congestion(side, sides) <= threshold_high {
            continue;
        }

        // Find least congested neighbor
        let cw = clockwise_neighbor(side);
        let ccw = counter_clockwise_neighbor(side);
        let cw_cong = congestion(cw, sides);
        let ccw_cong = congestion(ccw, sides);

        let (target_side, target_cong) = if cw_cong <= ccw_cong {
            (cw, cw_cong)
        } else {
            (ccw, ccw_cong)
        };

        if target_cong >= threshold_low {
            continue;
        }

        // Move the edge closest to the boundary between side and target_side
        if let Some(edges) = sides.get(&side) {
            if edges.len() <= 1 {
                continue;
            }
            let idx = find_edge_closest_to_side_boundary(edges, side, target_side);
            let edge = sides.get_mut(&side).unwrap().remove(idx);
            sides.entry(target_side).or_default().push(edge);
        }
    }
}

// ─── private helpers ─────────────────────────────────────────────────────────

fn find_edge_closest_to_boundary(edges: &[NodeEdgeInfo], side: Side) -> usize {
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

fn find_edge_closest_to_side_boundary(
    edges: &[NodeEdgeInfo],
    _from_side: Side,
    to_side: Side,
) -> usize {
    // Pick the edge whose angle is closest to the target side's center angle
    let target_center = match to_side {
        Side::Right => 0.0,
        Side::Bottom => 90.0,
        Side::Left => 180.0,
        Side::Top => 270.0,
    };

    edges
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = angle_distance(a.angle_deg, target_center);
            let db = angle_distance(b.angle_deg, target_center);
            da.partial_cmp(&db).unwrap()
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

fn clockwise_neighbor(side: Side) -> Side {
    match side {
        Side::Top => Side::Right,
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
    }
}

fn counter_clockwise_neighbor(side: Side) -> Side {
    match side {
        Side::Top => Side::Left,
        Side::Left => Side::Bottom,
        Side::Bottom => Side::Right,
        Side::Right => Side::Top,
    }
}
