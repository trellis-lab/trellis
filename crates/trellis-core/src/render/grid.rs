use crate::grid::Grid;

/// Render a grey point at a grid point
pub fn render_grid_dot(row: usize, col: usize, grid: &Grid) -> String {
    let (pos_x, pos_y) = grid.grid_to_world(row, col);

    format!(
        "    <circle cx=\"{}\" cy=\"{}\" r=\"0.5\" fill=\"grey\" />",
        pos_x, pos_y
    )
}
