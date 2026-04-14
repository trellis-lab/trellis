use std::collections::{HashMap, HashSet, VecDeque};
use trellis_parser::{Direction, Graph, Node, NodeShape};

use super::assignment::Side;

/// An edge associated with a particular node, tracking which end this node is.
#[derive(Debug, Clone)]
pub struct NodeEdgeInfo {
    pub edge_index: usize,
    pub angle_deg: f64,
    pub other_node_id: String,
    pub is_source: bool,
    /// Pre-computed side assignment (set by flow-aware or back-edge logic).
    /// When `None`, the side is computed from `angle_deg` at grouping time.
    pub side_override: Option<Side>,
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
/// Uses uniform 4 sectors of 90 degrees each.
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

/// Convert an angle to a side using flow-aware sector widths.
///
/// For TB graphs the Bottom sector widens from 90° to 135°
/// (112.5°..247.5°) so edges near quadrant boundaries go South rather
/// than East/West.  For LR graphs the same widening applies to the Right
/// sector.  For no-direction graphs the uniform 90° sectors are used.
pub fn angle_to_side_flow_aware(angle_deg: f64, direction: Direction) -> Side {
    let angle = ((angle_deg % 360.0) + 360.0) % 360.0;
    match direction {
        Direction::TB => {
            // North: 315..360 | 0..45   (45° arc — unchanged)
            // East:  45..112.5          (67.5° arc — narrowed from 90°)
            // South: 112.5..247.5       (135° arc — widened, dominant downward flow)
            // West:  247.5..315         (67.5° arc — narrowed from 90°)
            if (45.0..112.5).contains(&angle) {
                Side::Right // East
            } else if (112.5..247.5).contains(&angle) {
                Side::Bottom // South (dominant)
            } else if (247.5..315.0).contains(&angle) {
                Side::Left // West (narrowed)
            } else {
                // 315..360 | 0..45 — North
                Side::Top // North (back-edge direction)
            }
        }
        Direction::BT => {
            // Mirror of TB: Top/North sector widens (flow goes upward)
            // South: 315..360 | 0..45  (45° arc)
            // East:  45..112.5         (67.5°)
            // North: 112.5..247.5      (widened to 135°)
            // West:  247.5..315        (67.5°)
            if (45.0..112.5).contains(&angle) {
                Side::Right // East
            } else if (112.5..247.5).contains(&angle) {
                Side::Top // North widens for BT flow
            } else if (247.5..315.0).contains(&angle) {
                Side::Left // West
            } else {
                // 315..360 | 0..45 — South (downstream direction in BT)
                Side::Bottom // South
            }
        }
        Direction::LR => {
            // 90° rotation of TB: Right/East sector widens (dominant rightward flow)
            // North: 247.5..315   (67.5° arc)
            // East:  315..360 | 0..67.5  (135° arc — widened, dominant)
            // South: 67.5..157.5  (67.5° arc — narrowed)
            // West:  157.5..247.5 (67.5° arc — narrowed)
            if (315.0..360.0).contains(&angle) || angle < 67.5 {
                Side::Right // East dominant for LR
            } else if (67.5..157.5).contains(&angle) {
                Side::Bottom // South
            } else if (157.5..247.5).contains(&angle) {
                Side::Left // West
            } else {
                Side::Top // North
            }
        }
        Direction::RL => {
            // Mirror of LR: Left/West sector widens (dominant leftward flow)
            // East:  315..360 | 0..45    (45° arc)
            // South: 45..112.5           (67.5° arc)
            // West:  112.5..247.5        (135° arc — widened, dominant)
            // North: 247.5..315          (67.5° arc)
            if (112.5..247.5).contains(&angle) {
                Side::Left // West dominant for RL
            } else if (247.5..315.0).contains(&angle) {
                Side::Top // North
            } else if (45.0..112.5).contains(&angle) {
                Side::Bottom // South
            } else {
                Side::Right // East
            }
        }
    }
}

/// Compute a topological rank for each node (BFS from roots).
///
/// Nodes with no incoming edges get rank 0; their successors get rank 1, etc.
/// For cyclic graphs (back-edges), nodes reachable only via cycles get the
/// rank of their predecessor + 1 on the first BFS visit.
pub fn compute_topo_rank(graph: &Graph) -> HashMap<String, usize> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

    for node in &graph.nodes {
        in_degree.entry(node.id.as_str()).or_insert(0);
        adjacency.entry(node.id.as_str()).or_default();
    }
    for edge in &graph.edges {
        *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    let mut rank: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    // Seed with nodes that have no incoming edges
    for (node_id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node_id);
            rank.insert(node_id.to_string(), 0);
        }
    }

    // BFS
    let mut visited: HashSet<&str> = HashSet::new();
    while let Some(node_id) = queue.pop_front() {
        if visited.contains(node_id) {
            continue;
        }
        visited.insert(node_id);
        let current_rank = *rank.get(node_id).unwrap_or(&0);
        if let Some(successors) = adjacency.get(node_id) {
            for &succ in successors {
                if visited.contains(succ) {
                    // Back-edge or cross-edge — do not update rank of already-settled node
                    continue;
                }
                let new_rank = current_rank + 1;
                let entry = rank.entry(succ.to_string()).or_insert(new_rank);
                if new_rank > *entry {
                    *entry = new_rank;
                }
                queue.push_back(succ);
            }
        }
    }

    // Assign rank 0 to any nodes not yet visited (isolated or in pure cycles)
    for node in &graph.nodes {
        rank.entry(node.id.clone()).or_insert(0);
    }

    rank
}

/// Returns true if this edge goes against the primary flow direction
/// (i.e. the source has a higher topological rank than the target).
pub fn is_back_edge(from: &str, to: &str, topo_rank: &HashMap<String, usize>) -> bool {
    let src_rank = topo_rank.get(from).copied().unwrap_or(0);
    let tgt_rank = topo_rank.get(to).copied().unwrap_or(0);
    src_rank > tgt_rank
}

/// For a back-edge, override the side assignment so the source exits via the
/// upstream side and the target is entered from the downstream side.
pub fn override_side_for_back_edge(is_source: bool, direction: Direction) -> Side {
    match direction {
        Direction::TB => {
            if is_source {
                Side::Top
            } else {
                Side::Bottom
            }
        }
        Direction::BT => {
            if is_source {
                Side::Bottom
            } else {
                Side::Top
            }
        }
        Direction::LR => {
            if is_source {
                Side::Left
            } else {
                Side::Right
            }
        }
        Direction::RL => {
            if is_source {
                Side::Right
            } else {
                Side::Left
            }
        }
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

    // Diamond: only one port per side — the visual corner at the midpoint of each bbox edge.
    // Since diamonds are forced square (w_points == h_points, odd), the center index is exact.
    if node.shape == NodeShape::Diamond {
        let mid_col = gc + w_points / 2;
        let mid_row = gr + h_points / 2;
        let connector = match side {
            Side::Top => Connector {
                grid_row: gr,
                grid_col: mid_col,
                x: mid_col as f64 * cs + ox,
                y: gr as f64 * cs + oy,
            },
            Side::Bottom => {
                let bottom_row = gr + h_points - 1;
                Connector {
                    grid_row: bottom_row,
                    grid_col: mid_col,
                    x: mid_col as f64 * cs + ox,
                    y: bottom_row as f64 * cs + oy,
                }
            }
            Side::Left => Connector {
                grid_row: mid_row,
                grid_col: gc,
                x: gc as f64 * cs + ox,
                y: mid_row as f64 * cs + oy,
            },
            Side::Right => {
                let right_col = gc + w_points - 1;
                Connector {
                    grid_row: mid_row,
                    grid_col: right_col,
                    x: right_col as f64 * cs + ox,
                    y: mid_row as f64 * cs + oy,
                }
            }
        };
        return vec![connector];
    }

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
pub fn sort_edges_on_side(edges: &mut [NodeEdgeInfo], side: Side, node_map: &HashMap<&str, &Node>) {
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
/// `direction` controls whether flow-aware sector widths are used.
/// Pass `None` to use uniform 90° sectors (backwards-compatible).
///
/// Returns `(node_map, per_node_sides)` where `per_node_sides` maps each node id
/// to a side→edges mapping.
#[allow(clippy::type_complexity)]
pub fn build_edge_side_map<'a>(
    graph: &'a Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
    direction: Option<Direction>,
    topo_rank: &HashMap<String, usize>,
) -> (
    HashMap<&'a str, &'a Node>,
    HashMap<&'a str, HashMap<Side, Vec<NodeEdgeInfo>>>,
) {
    let node_map: HashMap<&str, &Node> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Group edges by node
    let mut node_edges: HashMap<&str, Vec<NodeEdgeInfo>> = HashMap::new();
    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        if let (Some(source), Some(target)) = (
            node_map.get(edge.from.as_str()),
            node_map.get(edge.to.as_str()),
        ) {
            let angle_from_source = calculate_angle(source, target);
            let angle_from_target = calculate_angle(target, source);

            // Determine sides using flow-aware sectors if direction is set,
            // with back-edge override applied first.
            let (src_side, tgt_side) = if let Some(dir) = direction {
                if is_back_edge(edge.from.as_str(), edge.to.as_str(), topo_rank) {
                    (
                        override_side_for_back_edge(true, dir),
                        override_side_for_back_edge(false, dir),
                    )
                } else {
                    (
                        angle_to_side_flow_aware(angle_from_source, dir),
                        angle_to_side_flow_aware(angle_from_target, dir),
                    )
                }
            } else {
                (
                    angle_to_side(angle_from_source),
                    angle_to_side(angle_from_target),
                )
            };

            node_edges
                .entry(edge.from.as_str())
                .or_default()
                .push(NodeEdgeInfo {
                    edge_index: edge_idx,
                    angle_deg: angle_from_source,
                    other_node_id: edge.to.clone(),
                    is_source: true,
                    side_override: Some(src_side),
                });

            node_edges
                .entry(edge.to.as_str())
                .or_default()
                .push(NodeEdgeInfo {
                    edge_index: edge_idx,
                    angle_deg: angle_from_target,
                    other_node_id: edge.from.clone(),
                    is_source: false,
                    side_override: Some(tgt_side),
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
            let side = info
                .side_override
                .unwrap_or_else(|| angle_to_side(info.angle_deg));
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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Flow-aware side selection tests ────────────────────────────────────────

    #[test]
    fn flow_aware_south_wins_near_boundary_tb() {
        // 113° is just inside the East quadrant with uniform sectors (45..135)
        // but with TB bias, 113° is in South (112.5..247.5)
        assert_eq!(angle_to_side_flow_aware(113.0, Direction::TB), Side::Bottom);
        // Deep into South sector — still Bottom
        assert_eq!(angle_to_side_flow_aware(180.0, Direction::TB), Side::Bottom);
        // East side (narrowed): 45..112.5
        assert_eq!(angle_to_side_flow_aware(80.0, Direction::TB), Side::Right);
        // West side (narrowed): 247.5..315 → Side::Left
        assert_eq!(angle_to_side_flow_aware(280.0, Direction::TB), Side::Left);
        // North sector: 315..360 | 0..45 → Side::Top
        assert_eq!(angle_to_side_flow_aware(0.0, Direction::TB), Side::Top);
        assert_eq!(angle_to_side_flow_aware(350.0, Direction::TB), Side::Top);
    }

    #[test]
    fn flow_aware_boundary_angles_tb() {
        // At exactly 112.5 — starts South sector
        assert_eq!(angle_to_side_flow_aware(112.5, Direction::TB), Side::Bottom);
        // At 247.4 — still South
        assert_eq!(angle_to_side_flow_aware(247.4, Direction::TB), Side::Bottom);
        // At 247.5 — starts West sector
        assert_eq!(angle_to_side_flow_aware(247.5, Direction::TB), Side::Left);
    }

    #[test]
    fn flow_aware_lr_east_dominant() {
        // LR: angles near 0/360 → Right (East dominant)
        assert_eq!(angle_to_side_flow_aware(0.0, Direction::LR), Side::Right);
        assert_eq!(angle_to_side_flow_aware(50.0, Direction::LR), Side::Right); // within 22.5..67.5
                                                                                // South: 67.5..157.5
        assert_eq!(angle_to_side_flow_aware(100.0, Direction::LR), Side::Bottom);
        // West: 157.5..247.5
        assert_eq!(angle_to_side_flow_aware(200.0, Direction::LR), Side::Left);
        // North: 247.5..315
        assert_eq!(angle_to_side_flow_aware(280.0, Direction::LR), Side::Top);
    }

    #[test]
    fn flow_aware_bt_top_dominant() {
        // BT: Top (North) sector widens — 112.5..247.5 → Top
        assert_eq!(angle_to_side_flow_aware(180.0, Direction::BT), Side::Top);
        // Near 0° → South/Bottom (downstream direction in BT)
        assert_eq!(angle_to_side_flow_aware(0.0, Direction::BT), Side::Bottom);
    }

    #[test]
    fn uniform_sectors_unchanged() {
        // angle_to_side should remain unchanged from original behavior
        assert_eq!(angle_to_side(0.0), Side::Right);
        assert_eq!(angle_to_side(90.0), Side::Bottom);
        assert_eq!(angle_to_side(180.0), Side::Left);
        assert_eq!(angle_to_side(270.0), Side::Top);
        assert_eq!(angle_to_side(113.0), Side::Bottom); // 113° > 45 and < 135 → Bottom
    }

    // ─── Back-edge detection tests ───────────────────────────────────────────────

    #[test]
    fn back_edge_detected_by_rank() {
        let mut rank = HashMap::new();
        rank.insert("A".to_string(), 0usize);
        rank.insert("B".to_string(), 1usize);
        rank.insert("C".to_string(), 2usize);

        // Forward edges
        assert!(!is_back_edge("A", "B", &rank));
        assert!(!is_back_edge("B", "C", &rank));
        // Back-edge
        assert!(is_back_edge("C", "A", &rank));
        assert!(is_back_edge("B", "A", &rank));
    }

    #[test]
    fn back_edge_override_tb() {
        // TB back-edge: source exits Top, target enters Bottom
        assert_eq!(override_side_for_back_edge(true, Direction::TB), Side::Top);
        assert_eq!(
            override_side_for_back_edge(false, Direction::TB),
            Side::Bottom
        );
    }

    #[test]
    fn back_edge_override_lr() {
        // LR back-edge: source exits Left, target enters Right
        assert_eq!(override_side_for_back_edge(true, Direction::LR), Side::Left);
        assert_eq!(
            override_side_for_back_edge(false, Direction::LR),
            Side::Right
        );
    }

    // ─── Topological rank tests ──────────────────────────────────────────────────

    #[test]
    fn topo_rank_linear_chain() {
        use trellis_parser::{ArrowHead, Edge, EdgeStyle, Graph, Node, NodeShape};

        let mut graph = Graph::new();
        graph.nodes = vec![
            Node {
                id: "A".into(),
                label: "A".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 0.0,
                y: 0.0,
                ..Default::default()
            },
            Node {
                id: "B".into(),
                label: "B".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 0.0,
                y: 50.0,
                ..Default::default()
            },
            Node {
                id: "C".into(),
                label: "C".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 0.0,
                y: 100.0,
                ..Default::default()
            },
        ];
        graph.edges = vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
            Edge {
                from: "B".into(),
                to: "C".into(),
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
        ];

        let rank = compute_topo_rank(&graph);
        assert_eq!(rank["A"], 0);
        assert_eq!(rank["B"], 1);
        assert_eq!(rank["C"], 2);
    }

    #[test]
    fn topo_rank_back_edge_in_chain_plus_cycle() {
        use trellis_parser::{ArrowHead, Edge, EdgeStyle, Graph, Node, NodeShape};

        // A → B → C, plus C → B (back-edge from layer 2 to layer 1)
        let mut graph = Graph::new();
        graph.nodes = vec![
            Node {
                id: "A".into(),
                label: "A".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 0.0,
                y: 0.0,
                ..Default::default()
            },
            Node {
                id: "B".into(),
                label: "B".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 0.0,
                y: 50.0,
                ..Default::default()
            },
            Node {
                id: "C".into(),
                label: "C".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 0.0,
                y: 100.0,
                ..Default::default()
            },
        ];
        graph.edges = vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
            Edge {
                from: "B".into(),
                to: "C".into(),
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
            Edge {
                from: "C".into(),
                to: "B".into(),
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
        ];

        let rank = compute_topo_rank(&graph);
        // A has rank 0, B has rank >= 1, C has rank >= 2
        // C → B is a back-edge (rank[C] > rank[B])
        assert!(rank["C"] > rank["B"], "C should have higher rank than B");
        assert!(is_back_edge("C", "B", &rank));
        // Forward edges are not back-edges
        assert!(!is_back_edge("A", "B", &rank));
        assert!(!is_back_edge("B", "C", &rank));
    }
}
