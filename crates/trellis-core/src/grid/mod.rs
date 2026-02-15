pub mod builder;
pub mod params;

pub use builder::{build_grid, Cell, CellState, Grid};
pub use params::{suggest_cell_size, calculate_grid_extent, GridExtent};
