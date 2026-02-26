use super::params::GridExtent;
use crate::placement::subgraph::VIRTUAL_PREFIX;
use trellis_parser::Graph;

/// State of a grid cell
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// Cell is free for routing
    Free,
    /// Cell is blocked by a node (impassable)
    Blocked,
    /// Cell is occupied by an already-routed edge
    Occupied,
}

/// Which side of a node boundary a cell belongs to or faces.
///
/// Used by the cost function to enforce perpendicular edge approach:
/// movement parallel to the boundary side is penalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundarySide {
    Top,
    Bottom,
    Left,
    Right,
}

/// A single cell in the routing grid
#[derive(Debug, Clone)]
pub struct Cell {
    pub state: CellState,
    pub cost: f64,
    pub owner: Option<String>,
    pub crossing: bool,
    /// If set, this cell is on or adjacent to a node boundary on the given side.
    /// The cost function uses this to penalize movement parallel to this side,
    /// forcing edges to approach nodes perpendicularly.
    pub boundary_side: Option<BoundarySide>,
    /// True if this cell is directly on a node boundary (a connector point).
    /// False for approach-zone cells (1 cell outside the boundary).
    /// Connector cells receive a much stronger perpendicularity penalty
    /// to ensure the very first/last segment of an edge is perpendicular.
    pub is_boundary_connector: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            state: CellState::Free,
            cost: 1.0,
            owner: None,
            crossing: false,
            boundary_side: None,
            is_boundary_connector: false,
        }
    }
}

/// The routing grid
#[derive(Debug, Clone)]
pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    pub cell_size: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    cells: Vec<Cell>,
}

impl Grid {
    /// Create a new grid with the given dimensions
    pub fn new(rows: usize, cols: usize, cell_size: i32, offset_x: i32, offset_y: i32) -> Self {
        let cells = vec![Cell::default(); rows * cols];
        Self {
            rows,
            cols,
            cell_size,
            offset_x,
            offset_y,
            cells,
        }
    }

    /// Get a reference to a cell at (row, col)
    pub fn get(&self, row: usize, col: usize) -> Option<&Cell> {
        if row < self.rows && col < self.cols {
            Some(&self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    /// Get a mutable reference to a cell at (row, col)
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row < self.rows && col < self.cols {
            Some(&mut self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    /// Check if coordinates are within bounds
    pub fn in_bounds(&self, row: i64, col: i64) -> bool {
        row >= 0 && col >= 0 && (row as usize) < self.rows && (col as usize) < self.cols
    }

    /// Convert world coordinates to grid coordinates
    pub fn world_to_grid(&self, x: f64, y: f64) -> (i64, i64) {
        let col = ((x - self.offset_x as f64) / self.cell_size as f64).round() as i64;
        let row = ((y - self.offset_y as f64) / self.cell_size as f64).round() as i64;
        (row, col)
    }

    /// Convert grid coordinates to world coordinates
    pub fn grid_to_world(&self, row: usize, col: usize) -> (f64, f64) {
        let x = col as f64 * self.cell_size as f64 + self.offset_x as f64;
        let y = row as f64 * self.cell_size as f64 + self.offset_y as f64;
        (x, y)
    }

    /// Count the number of free cells
    pub fn free_cell_count(&self) -> usize {
        self.cells.iter().filter(|c| c.state == CellState::Free).count()
    }

    /// Count the number of blocked cells
    pub fn blocked_cell_count(&self) -> usize {
        self.cells.iter().filter(|c| c.state == CellState::Blocked).count()
    }

    /// Count the number of cells flagged as crossings (two routed paths share a cell)
    pub fn count_crossings(&self) -> usize {
        self.cells.iter().filter(|c| c.crossing).count()
    }

    /// Calculate grid utilization (fraction of non-free cells)
    pub fn utilization(&self) -> f64 {
        if self.cells.is_empty() {
            return 0.0;
        }
        let non_free = self.cells.iter().filter(|c| c.state != CellState::Free).count();
        non_free as f64 / self.cells.len() as f64
    }
}

/// Build a routing grid from the graph with placed nodes.
///
/// Distinguishes three types of grid points on node rectangles:
/// - **Interior** (strictly inside): `Blocked` (impassable)
/// - **Corners** (4 vertices): `Blocked` (impassable)
/// - **Boundary non-corner** (edge points excluding corners): `Free` (connectors)
pub fn build_grid(graph: &Graph, cell_size: i32, extent: &GridExtent) -> Grid {
    let cs = cell_size as f64;
    let cols = (extent.width / cs).ceil() as usize;
    let rows = (extent.height / cs).ceil() as usize;

    // Ensure minimum grid size
    let rows = rows.max(1);
    let cols = cols.max(1);

    let mut grid = Grid::new(rows, cols, cell_size, extent.offset_x, extent.offset_y);

    let ox = extent.offset_x as f64;
    let oy = extent.offset_y as f64;

    // Block cells underneath each node with boundary/interior distinction
    // and record boundary side for connector cells.
    //
    // Also collect connector positions so we can mark approach zones afterwards.
    let mut connectors: Vec<(i64, i64, BoundarySide)> = Vec::new();

    for node in &graph.nodes {
        let is_virtual = node.id.starts_with(VIRTUAL_PREFIX);

        // Node grid coordinates (top-left is on a grid point)
        let gc = ((node.x - ox) / cs).round() as i64;
        let gr = ((node.y - oy) / cs).round() as i64;

        // Grid-point counts
        let w_points = (node.width / cs).round() as i64 + 1;
        let h_points = (node.height / cs).round() as i64 + 1;

        let max_col = gc + w_points - 1;
        let max_row = gr + h_points - 1;

        for row in gr..=max_row {
            for col in gc..=max_col {
                if !grid.in_bounds(row, col) {
                    continue;
                }
                let on_top = row == gr;
                let on_bottom = row == max_row;
                let on_left = col == gc;
                let on_right = col == max_col;
                let on_boundary = on_top || on_bottom || on_left || on_right;
                let is_corner = (on_top || on_bottom) && (on_left || on_right);

                if let Some(cell) = grid.get_mut(row as usize, col as usize) {
                    if is_virtual {
                        // Virtual subgraph nodes: don't block cells, but mark
                        // boundary cells for perpendicularity enforcement so
                        // edges approach the subgraph frame perpendicularly.
                        if on_boundary && !is_corner {
                            let side = if on_top {
                                BoundarySide::Top
                            } else if on_bottom {
                                BoundarySide::Bottom
                            } else if on_left {
                                BoundarySide::Left
                            } else {
                                BoundarySide::Right
                            };
                            cell.boundary_side = Some(side);
                            cell.is_boundary_connector = true;
                            cell.owner = Some(node.id.clone());
                            connectors.push((row, col, side));
                        }
                    } else {
                        if !on_boundary || is_corner {
                            // Interior or corner → blocked
                            cell.state = CellState::Blocked;
                            cell.cost = f64::INFINITY;
                        } else {
                            // Boundary non-corner → Free connector point
                            let side = if on_top {
                                BoundarySide::Top
                            } else if on_bottom {
                                BoundarySide::Bottom
                            } else if on_left {
                                BoundarySide::Left
                            } else {
                                BoundarySide::Right
                            };
                            cell.boundary_side = Some(side);
                            cell.is_boundary_connector = true;
                            connectors.push((row, col, side));
                        }
                        cell.owner = Some(node.id.clone());
                    }
                }
            }
        }
    }

    // Mark approach zone cells (1 cell outside each boundary connector).
    // This extends the perpendicularity enforcement beyond the connector itself.
    for (row, col, side) in &connectors {
        let (ar, ac) = match side {
            BoundarySide::Top => (row - 1, *col),
            BoundarySide::Bottom => (row + 1, *col),
            BoundarySide::Left => (*row, col - 1),
            BoundarySide::Right => (*row, col + 1),
        };
        if grid.in_bounds(ar, ac) {
            if let Some(cell) = grid.get_mut(ar as usize, ac as usize) {
                if cell.state == CellState::Free && cell.boundary_side.is_none() {
                    cell.boundary_side = Some(*side);
                }
            }
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Node, NodeShape};

    fn make_node(id: &str, w: f64, h: f64, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: w,
            height: h,
            x,
            y,
            ..Default::default()        }
    }

    #[test]
    fn test_build_grid_basic() {
        let mut graph = Graph::new();
        // Top-left at (30, 40), size 40x20, cell_size=10
        // Grid points: gc=3, gr=4, w_points=5, h_points=3
        // Node spans grid rows 4..6, cols 3..7
        graph.nodes = vec![make_node("A", 40.0, 20.0, 30.0, 40.0)];

        let extent = GridExtent {
            width: 200.0,
            height: 200.0,
            offset_x: 0,
            offset_y: 0,
        };

        let grid = build_grid(&graph, 10, &extent);
        assert_eq!(grid.rows, 20);
        assert_eq!(grid.cols, 20);

        // Outside the node → Free
        assert_eq!(grid.get(0, 0).unwrap().state, CellState::Free);

        // Interior cell (row 5, col 5) → Blocked
        assert_eq!(grid.get(5, 5).unwrap().state, CellState::Blocked);

        // Corner (top-left: row 4, col 3) → Blocked
        assert_eq!(grid.get(4, 3).unwrap().state, CellState::Blocked);

        // Corner (bottom-right: row 6, col 7) → Blocked
        assert_eq!(grid.get(6, 7).unwrap().state, CellState::Blocked);

        // Boundary non-corner (top edge, row 4, col 5) → Free (connector)
        assert_eq!(grid.get(4, 5).unwrap().state, CellState::Free);

        // Boundary non-corner (left edge, row 5, col 3) → Free (connector)
        assert_eq!(grid.get(5, 3).unwrap().state, CellState::Free);

        // Owner should be set for all node grid points
        assert_eq!(grid.get(5, 5).unwrap().owner, Some("A".to_string()));
        assert_eq!(grid.get(4, 5).unwrap().owner, Some("A".to_string()));
    }

    #[test]
    fn test_grid_utilization() {
        let grid = Grid::new(10, 10, 10, 0, 0);
        assert_eq!(grid.utilization(), 0.0);
        assert_eq!(grid.free_cell_count(), 100);
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = Grid::new(10, 10, 10, 0, 0);
        let (row, col) = grid.world_to_grid(55.0, 35.0);
        assert_eq!(col, 6); // round(55/10) = 6
        assert_eq!(row, 4); // round(35/10) = 4 (note: was 3, let me check - 35/10=3.5 rounds to 4)
    }

    #[test]
    fn test_grid_to_world_conversion() {
        let grid = Grid::new(10, 10, 10, 5, 5);
        let (x, y) = grid.grid_to_world(3, 4);
        assert_eq!(x, 45.0); // 4*10 + 5
        assert_eq!(y, 35.0); // 3*10 + 5
    }

    #[test]
    fn test_in_bounds() {
        let grid = Grid::new(5, 5, 10, 0, 0);
        assert!(grid.in_bounds(0, 0));
        assert!(grid.in_bounds(4, 4));
        assert!(!grid.in_bounds(5, 0));
        assert!(!grid.in_bounds(-1, 0));
    }

    #[test]
    fn test_boundary_side_on_connector_cells() {
        // Node at (30, 40), size 40x20, cell_size=10
        // gc=3, gr=4, w_points=5, h_points=3
        // Node spans rows 4..6, cols 3..7
        let mut graph = Graph::new();
        graph.nodes = vec![make_node("A", 40.0, 20.0, 30.0, 40.0)];

        let extent = GridExtent {
            width: 200.0,
            height: 200.0,
            offset_x: 0,
            offset_y: 0,
        };
        let grid = build_grid(&graph, 10, &extent);

        // Top boundary connector (row 4, col 5) → BoundarySide::Top
        assert_eq!(grid.get(4, 5).unwrap().boundary_side, Some(BoundarySide::Top));

        // Bottom boundary connector (row 6, col 5) → BoundarySide::Bottom
        assert_eq!(grid.get(6, 5).unwrap().boundary_side, Some(BoundarySide::Bottom));

        // Left boundary connector (row 5, col 3) → BoundarySide::Left
        assert_eq!(grid.get(5, 3).unwrap().boundary_side, Some(BoundarySide::Left));

        // Right boundary connector (row 5, col 7) → BoundarySide::Right
        assert_eq!(grid.get(5, 7).unwrap().boundary_side, Some(BoundarySide::Right));

        // Interior cell → no boundary_side
        assert_eq!(grid.get(5, 5).unwrap().boundary_side, None);

        // Corner cell → no boundary_side (blocked)
        assert_eq!(grid.get(4, 3).unwrap().boundary_side, None);

        // Cell far away from node → no boundary_side
        assert_eq!(grid.get(0, 0).unwrap().boundary_side, None);
    }

    #[test]
    fn test_approach_zone_cells_marked() {
        // Node at (30, 40), size 40x20, cell_size=10
        // gc=3, gr=4, w_points=5, h_points=3
        // Node spans rows 4..6, cols 3..7
        let mut graph = Graph::new();
        graph.nodes = vec![make_node("A", 40.0, 20.0, 30.0, 40.0)];

        let extent = GridExtent {
            width: 200.0,
            height: 200.0,
            offset_x: 0,
            offset_y: 0,
        };
        let grid = build_grid(&graph, 10, &extent);

        // One cell above a top connector (row 3, col 5) → approach zone for Top
        assert_eq!(grid.get(3, 5).unwrap().boundary_side, Some(BoundarySide::Top));
        assert_eq!(grid.get(3, 5).unwrap().state, CellState::Free);

        // One cell below a bottom connector (row 7, col 5) → approach zone for Bottom
        assert_eq!(grid.get(7, 5).unwrap().boundary_side, Some(BoundarySide::Bottom));

        // One cell left of a left connector (row 5, col 2) → approach zone for Left
        assert_eq!(grid.get(5, 2).unwrap().boundary_side, Some(BoundarySide::Left));

        // One cell right of a right connector (row 5, col 8) → approach zone for Right
        assert_eq!(grid.get(5, 8).unwrap().boundary_side, Some(BoundarySide::Right));

        // Two cells away → no boundary_side
        assert_eq!(grid.get(2, 5).unwrap().boundary_side, None);
    }
}
