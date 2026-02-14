use super::params::GridExtent;
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

/// A single cell in the routing grid
#[derive(Debug, Clone)]
pub struct Cell {
    pub state: CellState,
    pub cost: f64,
    pub owner: Option<String>,
    pub crossing: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            state: CellState::Free,
            cost: 1.0,
            owner: None,
            crossing: false,
        }
    }
}

/// The routing grid
#[derive(Debug, Clone)]
pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    pub cell_size: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    cells: Vec<Cell>,
}

impl Grid {
    /// Create a new grid with the given dimensions
    pub fn new(rows: usize, cols: usize, cell_size: f64, offset_x: f64, offset_y: f64) -> Self {
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
        let col = ((x - self.offset_x) / self.cell_size).round() as i64;
        let row = ((y - self.offset_y) / self.cell_size).round() as i64;
        (row, col)
    }

    /// Convert grid coordinates to world coordinates
    pub fn grid_to_world(&self, row: usize, col: usize) -> (f64, f64) {
        let x = col as f64 * self.cell_size + self.offset_x;
        let y = row as f64 * self.cell_size + self.offset_y;
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
/// Blocks cells underneath node bounding boxes.
pub fn build_grid(graph: &Graph, cell_size: f64, extent: &GridExtent) -> Grid {
    let cols = (extent.width / cell_size).ceil() as usize;
    let rows = (extent.height / cell_size).ceil() as usize;

    // Ensure minimum grid size
    let rows = rows.max(1);
    let cols = cols.max(1);

    let mut grid = Grid::new(rows, cols, cell_size, extent.offset_x, extent.offset_y);

    // Block cells underneath each node
    for node in &graph.nodes {
        // Node positions are center coordinates
        let node_left = node.x - node.width / 2.0;
        let node_top = node.y - node.height / 2.0;
        let node_right = node.x + node.width / 2.0;
        let node_bottom = node.y + node.height / 2.0;

        let start_col = ((node_left - extent.offset_x) / cell_size).floor() as i64;
        let end_col = ((node_right - extent.offset_x) / cell_size).ceil() as i64;
        let start_row = ((node_top - extent.offset_y) / cell_size).floor() as i64;
        let end_row = ((node_bottom - extent.offset_y) / cell_size).ceil() as i64;

        for row in start_row..end_row {
            for col in start_col..end_col {
                if grid.in_bounds(row, col) {
                    if let Some(cell) = grid.get_mut(row as usize, col as usize) {
                        cell.state = CellState::Blocked;
                        cell.cost = f64::INFINITY;
                        cell.owner = Some(node.id.clone());
                    }
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
        }
    }

    #[test]
    fn test_build_grid_basic() {
        let mut graph = Graph::new();
        graph.nodes = vec![make_node("A", 40.0, 20.0, 50.0, 50.0)];

        let extent = GridExtent {
            width: 200.0,
            height: 200.0,
            offset_x: 0.0,
            offset_y: 0.0,
        };

        let grid = build_grid(&graph, 10.0, &extent);
        assert_eq!(grid.rows, 20);
        assert_eq!(grid.cols, 20);

        // Node A centered at (50,50) with size 40x20 occupies (30..70, 40..60)
        // In grid coords: cols 3..7, rows 4..6
        assert_eq!(grid.get(5, 5).unwrap().state, CellState::Blocked);
        assert_eq!(grid.get(0, 0).unwrap().state, CellState::Free);
    }

    #[test]
    fn test_grid_utilization() {
        let grid = Grid::new(10, 10, 10.0, 0.0, 0.0);
        assert_eq!(grid.utilization(), 0.0);
        assert_eq!(grid.free_cell_count(), 100);
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = Grid::new(10, 10, 10.0, 0.0, 0.0);
        let (row, col) = grid.world_to_grid(55.0, 35.0);
        assert_eq!(col, 6); // round(55/10) = 6
        assert_eq!(row, 4); // round(35/10) = 4 (note: was 3, let me check - 35/10=3.5 rounds to 4)
    }

    #[test]
    fn test_grid_to_world_conversion() {
        let grid = Grid::new(10, 10, 10.0, 5.0, 5.0);
        let (x, y) = grid.grid_to_world(3, 4);
        assert_eq!(x, 45.0); // 4*10 + 5
        assert_eq!(y, 35.0); // 3*10 + 5
    }

    #[test]
    fn test_in_bounds() {
        let grid = Grid::new(5, 5, 10.0, 0.0, 0.0);
        assert!(grid.in_bounds(0, 0));
        assert!(grid.in_bounds(4, 4));
        assert!(!grid.in_bounds(5, 0));
        assert!(!grid.in_bounds(-1, 0));
    }
}
