use crate::grid::Grid;
use crate::theme::Theme;

/// Render crossing indicators as SVG elements.
///
/// At each crossing point (where `cell.crossing == true`), draws a small
/// white circle background with an arc, creating the "bridge" visual effect
/// that clearly communicates two edges cross at this point.
pub fn render_crossings(grid: &Grid, theme: &Theme) -> String {
    let mut svg = String::new();

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(cell) = grid.get(row, col) {
                if cell.crossing {
                    let (x, y) = grid.grid_to_world(row, col);
                    let r = grid.cell_size as f64 * 0.4;

                    // Background circle to create the "gap" effect
                    svg.push_str(&format!(
                        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
                         fill=\"{}\" stroke=\"none\"/>\n",
                        x, y, r, theme.crossing_bg
                    ));

                    // Small arc to show the bridge
                    svg.push_str(&format!(
                        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
                         fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                        x, y, r, theme.crossing_stroke
                    ));
                }
            }
        }
    }

    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellState, Grid};
    use crate::theme::themes::DEFAULT;

    #[test]
    fn test_no_crossings_empty_output() {
        let grid = Grid::new(5, 5, 10, 0, 0);
        let svg = render_crossings(&grid, &DEFAULT);
        assert!(svg.is_empty());
    }

    #[test]
    fn test_crossing_generates_svg() {
        let mut grid = Grid::new(5, 5, 10, 0, 0);
        if let Some(cell) = grid.get_mut(2, 3) {
            cell.state = CellState::Occupied;
            cell.crossing = true;
        }

        let svg = render_crossings(&grid, &DEFAULT);
        assert!(svg.contains("<circle"));
        assert!(svg.contains(&format!("fill=\"{}\"", DEFAULT.crossing_bg)));
    }
}
