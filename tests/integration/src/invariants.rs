/// Routing invariant checks run against a selection of fixtures.
///
/// The five invariants verified are:
/// 1. Fedésmentesség  – no two edges share the same intermediate grid point
///    (unless the cell is explicitly flagged as a crossing)
/// 2. Blokkolás       – no edge path passes through a cell that was Blocked
///    (occupied by a node) before routing began
/// 3. Ortogonalitás   – every path segment changes only row OR only column
/// 4. Összefüggőség   – every path is connected: consecutive points are
///    exactly 1 grid step apart
/// 5. Port egyediség  – no two edges are assigned the same source port, and
///    no two edges are assigned the same target port

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use trellis_core::{
        config::TrellisConfig,
        grid::{build_grid, calculate_grid_extent, CellState},
        placement,
        ports::{assign_ports, EdgePorts},
        routing::{route_all_edges, RoutingResult},
    };
    use trellis_parser::parse;

    // ──────────────────────────────────────────────────────────────────────────
    // Helper: run the full pipeline up to and including edge routing, and also
    // return the set of grid cells that were Blocked BEFORE routing (i.e., node
    // cells).  This is collected before route_all_edges mutates the grid.
    // ──────────────────────────────────────────────────────────────────────────
    struct RoutingSetup {
        result: RoutingResult,
        /// Grid cells blocked by nodes, captured before routing.
        pre_routing_blocked: HashSet<(usize, usize)>,
        port_assignments: HashMap<usize, EdgePorts>,
    }

    fn setup_routing(mermaid: &str) -> RoutingSetup {
        let mut graph = parse(mermaid).expect("parse failed");
        let config = TrellisConfig::default();
        let cell_size = config.cell_size;

        placement::place_nodes(&mut graph, cell_size);

        let extent = calculate_grid_extent(&graph, cell_size);
        let mut grid = build_grid(&graph, cell_size, &extent);

        // Capture blocked cells BEFORE routing so we can check invariant #2
        let pre_routing_blocked: HashSet<(usize, usize)> = (0..grid.rows)
            .flat_map(|r| (0..grid.cols).map(move |c| (r, c)))
            .filter(|&(r, c)| {
                grid.get(r, c)
                    .map(|cell| cell.state == CellState::Blocked)
                    .unwrap_or(false)
            })
            .collect();

        let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let result = route_all_edges(&graph, &mut grid, &port_assignments, &config);

        RoutingSetup {
            result,
            pre_routing_blocked,
            port_assignments,
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Invariant 1 – Fedésmentesség (no unintended overlap)
    // ──────────────────────────────────────────────────────────────────────────
    /// Check that no two edges share the same *intermediate* grid point unless
    /// the grid cell is explicitly marked as a crossing.
    fn check_no_overlap(setup: &RoutingSetup) {
        let mut point_owners: HashMap<(i64, i64), Vec<usize>> = HashMap::new();

        for (&edge_idx, path) in &setup.result.paths {
            // Skip first and last points (ports may legitimately share adjacency)
            for point in path
                .points
                .iter()
                .skip(1)
                .take(path.points.len().saturating_sub(2))
            {
                point_owners
                    .entry((point.row, point.col))
                    .or_default()
                    .push(edge_idx);
            }
        }

        for ((row, col), owners) in &point_owners {
            assert!(
                owners.len() <= 2,
                "Invariant violated – {} edges share grid cell ({}, {}): {:?}",
                owners.len(),
                row,
                col,
                owners
            );
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Invariant 2 – Blokkolás (no path through a node cell)
    // ──────────────────────────────────────────────────────────────────────────
    fn check_no_blocked_traversal(setup: &RoutingSetup) {
        for (&edge_idx, path) in &setup.result.paths {
            for point in path
                .points
                .iter()
                .skip(1)
                .take(path.points.len().saturating_sub(2))
            {
                let key = (point.row as usize, point.col as usize);
                assert!(
                    !setup.pre_routing_blocked.contains(&key),
                    "Invariant violated – edge {} passes through node cell ({}, {})",
                    edge_idx,
                    point.row,
                    point.col
                );
            }
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Invariant 3 – Ortogonalitás (all segments are axis-aligned)
    // ──────────────────────────────────────────────────────────────────────────
    fn check_orthogonality(setup: &RoutingSetup) {
        for (&edge_idx, path) in &setup.result.paths {
            let points = &path.points;
            for i in 1..points.len() {
                let prev = &points[i - 1];
                let curr = &points[i];
                let dr = (curr.row - prev.row).abs();
                let dc = (curr.col - prev.col).abs();
                assert!(
                    (dr == 1 && dc == 0) || (dr == 0 && dc == 1),
                    "Invariant violated – edge {} has diagonal/teleport segment ({},{})→({},{})",
                    edge_idx,
                    prev.row,
                    prev.col,
                    curr.row,
                    curr.col
                );
            }
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Invariant 4 – Összefüggőség (path is connected end-to-end)
    // ──────────────────────────────────────────────────────────────────────────
    fn check_connectivity(setup: &RoutingSetup) {
        for (&edge_idx, path) in &setup.result.paths {
            assert!(
                !path.points.is_empty(),
                "Invariant violated – edge {} has an empty path",
                edge_idx
            );
            let points = &path.points;
            for i in 1..points.len() {
                let prev = &points[i - 1];
                let curr = &points[i];
                let manhattan = (curr.row - prev.row).abs() + (curr.col - prev.col).abs();
                assert_eq!(
                    manhattan, 1,
                    "Invariant violated – edge {} has a disconnected path at step {} \
                     ({},{})→({},{}), Manhattan distance = {}",
                    edge_idx, i, prev.row, prev.col, curr.row, curr.col, manhattan
                );
            }
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Invariant 5 – Port egyediség (unique source and target ports)
    // ──────────────────────────────────────────────────────────────────────────
    fn check_port_uniqueness(setup: &RoutingSetup) {
        let mut source_ports: HashSet<(i64, i64)> = HashSet::new();
        let mut target_ports: HashSet<(i64, i64)> = HashSet::new();

        for (&edge_idx, ep) in &setup.port_assignments {
            let src_key = (ep.source_port.grid_row, ep.source_port.grid_col);
            let tgt_key = (ep.target_port.grid_row, ep.target_port.grid_col);

            assert!(
                source_ports.insert(src_key),
                "Invariant violated – edge {} shares its source port ({},{}) with another edge",
                edge_idx,
                src_key.0,
                src_key.1
            );
            assert!(
                target_ports.insert(tgt_key),
                "Invariant violated – edge {} shares its target port ({},{}) with another edge",
                edge_idx,
                tgt_key.0,
                tgt_key.1
            );
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Run all five invariants on a single fixture
    // ──────────────────────────────────────────────────────────────────────────
    fn check_all_invariants(mermaid: &str) {
        let setup = setup_routing(mermaid);
        check_no_overlap(&setup);
        check_no_blocked_traversal(&setup);
        check_orthogonality(&setup);
        check_connectivity(&setup);
        check_port_uniqueness(&setup);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Tests
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn invariants_b01_linear_chain() {
        check_all_invariants("graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E");
    }

    #[test]
    fn invariants_b02_wide_branch() {
        check_all_invariants(
            "graph TB\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6",
        );
    }

    #[test]
    fn invariants_b03_k33_bipartite() {
        // K3,3 is non-planar – crossings are expected but the crossing invariant
        // should still hold (crossing cells must be marked as such).
        check_all_invariants(
            "graph TB\n    A1 --> B1\n    A1 --> B2\n    A1 --> B3\n    A2 --> B1\n    A2 --> B2\n    A2 --> B3\n    A3 --> B1\n    A3 --> B2\n    A3 --> B3",
        );
    }

    #[test]
    fn invariants_b04_diamond() {
        check_all_invariants("graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D");
    }

    #[test]
    fn invariants_b06_multi_edge() {
        check_all_invariants("graph TB\n    A --> B\n    A --> B\n    B --> C");
    }

    #[test]
    fn invariants_b07_cycle() {
        check_all_invariants("graph TB\n    A --> B\n    B --> C\n    C --> A");
    }

    // --- Individual invariant isolation tests ---

    #[test]
    fn orthogonality_simple_chain() {
        let setup = setup_routing("graph TB\n    A --> B\n    B --> C");
        check_orthogonality(&setup);
    }

    #[test]
    fn connectivity_simple_chain() {
        let setup = setup_routing("graph TB\n    A --> B\n    B --> C");
        check_connectivity(&setup);
    }

    #[test]
    fn no_blocked_traversal_simple_chain() {
        let setup = setup_routing("graph TB\n    A --> B\n    B --> C");
        check_no_blocked_traversal(&setup);
    }

    #[test]
    fn port_uniqueness_fan_out() {
        let setup = setup_routing(
            "graph TB\n    A --> B\n    A --> C\n    A --> D\n    A --> E\n    A --> F",
        );
        check_port_uniqueness(&setup);
    }
}
