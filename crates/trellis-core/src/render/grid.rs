use crate::grid::Grid;
use crate::theme::Theme;

/// Render a themed point at a grid cell.
pub fn render_grid_dot(row: usize, col: usize, grid: &Grid, theme: &Theme) -> String {
    let (pos_x, pos_y) = grid.grid_to_world(row, col);

    format!(
        "    <circle cx=\"{}\" cy=\"{}\" r=\"0.5\" fill=\"{}\" />",
        pos_x, pos_y, theme.grid_dot
    )
}
