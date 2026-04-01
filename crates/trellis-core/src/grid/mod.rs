pub mod builder;
pub mod params;

pub use builder::{build_grid, BoundarySide, Cell, CellState, Grid};
pub use params::{calculate_grid_extent, suggest_cell_size, GridExtent};
