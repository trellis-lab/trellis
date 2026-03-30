//! Generic row-flow layout algorithm.
//!
//! Places a slice of nodes left-to-right with a configurable number of items
//! per row. The algorithm is diagram-agnostic: it operates on indexed node
//! slots and does not inspect any diagram-type-specific fields.

use trellis_parser::Graph;

use super::algorithm::LayoutAlgorithm;

/// Horizontal gap between elements (pixels).
pub const ELEM_GAP_X: f64 = 40.0;
/// Vertical gap between rows (pixels).
pub const ELEM_GAP_Y: f64 = 60.0;

/// Compute the maximum height of each row for a slice of node indices.
///
/// Returns one height value per row (in row order).
pub fn compute_row_heights(
    nodes: &[trellis_parser::Node],
    indices: &[usize],
    shapes_per_row: usize,
) -> Vec<f64> {
    let mut row_heights: Vec<f64> = Vec::new();
    let mut max_h = 0.0_f64;

    for (slot, &idx) in indices.iter().enumerate() {
        max_h = max_h.max(nodes[idx].height);
        if (slot + 1) % shapes_per_row == 0 || slot + 1 == indices.len() {
            row_heights.push(max_h);
            max_h = 0.0;
        }
    }

    row_heights
}

/// Place nodes in row-flow order.
///
/// Nodes are placed left-to-right starting at (`start_x`, `start_y`),
/// wrapping to a new row after every `shapes_per_row` elements.
/// `row_heights` must have been produced by [`compute_row_heights`] for the
/// same `indices` slice.
///
/// Returns the y coordinate of the bottom edge of the last row
/// (i.e. `start_y` of next section, before any additional gap).
pub fn place_in_rows(
    graph: &mut Graph,
    indices: &[usize],
    start_x: f64,
    start_y: f64,
    shapes_per_row: usize,
    row_heights: &[f64],
) -> f64 {
    if indices.is_empty() {
        return start_y;
    }

    let mut row_start_y = start_y;
    let mut x_cursor = start_x;
    let mut last_row_idx = 0;

    for (slot, &idx) in indices.iter().enumerate() {
        let row_idx = slot / shapes_per_row;
        let col_idx = slot % shapes_per_row;
        last_row_idx = row_idx;

        if col_idx == 0 {
            x_cursor = start_x;
        }

        graph.nodes[idx].x = x_cursor;
        graph.nodes[idx].y = row_start_y;

        x_cursor += graph.nodes[idx].width + ELEM_GAP_X;

        // Advance to the next row when this row is full and more nodes follow
        if col_idx + 1 == shapes_per_row && slot + 1 < indices.len() {
            row_start_y += row_heights.get(row_idx).copied().unwrap_or(0.0) + ELEM_GAP_Y;
        }
    }

    row_start_y + row_heights.get(last_row_idx).copied().unwrap_or(0.0)
}

/// Handle for the row-flow layout algorithm.
///
/// Places every node in the graph left-to-right in rows of `shapes_per_row`
/// elements, starting at the origin `(0, 0)`.
///
/// For sub-group row-flow (e.g. C4 boundary groups) call
/// [`place_in_rows`] / [`compute_row_heights`] directly instead.
pub struct RowFlowLayout {
    pub shapes_per_row: usize,
}

impl LayoutAlgorithm for RowFlowLayout {
    fn layout(&self, graph: &mut Graph) {
        let indices: Vec<usize> = (0..graph.nodes.len()).collect();
        let row_heights = compute_row_heights(&graph.nodes, &indices, self.shapes_per_row);
        place_in_rows(graph, &indices, 0.0, 0.0, self.shapes_per_row, &row_heights);
    }
}
