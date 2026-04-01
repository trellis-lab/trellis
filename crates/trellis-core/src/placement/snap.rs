use std::collections::HashSet;

/// Snap a coordinate to the nearest grid position.
pub fn snap_to_grid(value: f64, grid_size: f64) -> f64 {
    (value / grid_size).round() * grid_size
}

/// Snap all node coordinates to a grid, resolving collisions with spiral search.
///
/// Each node occupies a rectangular area on the grid. If two nodes would occupy
/// the same grid cell, the second node is moved to the nearest free cell using
/// a spiral search pattern.
pub fn snap_nodes_to_grid(
    nodes: &mut [(f64, f64, f64, f64)], // (x, y, width, height)
    grid_size: f64,
) {
    let mut occupied: HashSet<(i64, i64)> = HashSet::new();

    for (x, y, width, height) in nodes.iter_mut() {
        let snapped_x = snap_to_grid(*x, grid_size);
        let snapped_y = snap_to_grid(*y, grid_size);

        // Calculate grid cells occupied by this node
        let grid_x = (snapped_x / grid_size).round() as i64;
        let grid_y = (snapped_y / grid_size).round() as i64;

        if !occupied.contains(&(grid_x, grid_y)) {
            *x = snapped_x;
            *y = snapped_y;
            // Mark cells occupied by this node
            let cells_w = ((*width / grid_size).ceil() as i64).max(1);
            let cells_h = ((*height / grid_size).ceil() as i64).max(1);
            for dx in 0..cells_w {
                for dy in 0..cells_h {
                    occupied.insert((grid_x + dx, grid_y + dy));
                }
            }
        } else {
            // Spiral search for free position
            if let Some((free_x, free_y)) = spiral_search(grid_x, grid_y, &occupied) {
                *x = free_x as f64 * grid_size;
                *y = free_y as f64 * grid_size;
                let cells_w = ((*width / grid_size).ceil() as i64).max(1);
                let cells_h = ((*height / grid_size).ceil() as i64).max(1);
                for dx in 0..cells_w {
                    for dy in 0..cells_h {
                        occupied.insert((free_x + dx, free_y + dy));
                    }
                }
            } else {
                *x = snapped_x;
                *y = snapped_y;
            }
        }
    }
}

/// Spiral search outward from (cx, cy) to find the nearest unoccupied grid cell.
fn spiral_search(cx: i64, cy: i64, occupied: &HashSet<(i64, i64)>) -> Option<(i64, i64)> {
    for radius in 1i64..=50 {
        // Check all cells at manhattan distance == radius
        for dx in -radius..=radius {
            let dy = radius - dx.abs();
            for &dy in &[dy, -dy] {
                let (nx, ny) = (cx + dx, cy + dy);
                if !occupied.contains(&(nx, ny)) {
                    return Some((nx, ny));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_to_grid() {
        assert_eq!(snap_to_grid(23.0, 10.0), 20.0);
        assert_eq!(snap_to_grid(25.0, 10.0), 30.0); // rounds to nearest
        assert_eq!(snap_to_grid(50.0, 10.0), 50.0);
        assert_eq!(snap_to_grid(0.0, 10.0), 0.0);
    }

    #[test]
    fn test_snap_nodes_no_collision() {
        let mut nodes = vec![(100.0, 100.0, 60.0, 40.0), (200.0, 200.0, 60.0, 40.0)];
        snap_nodes_to_grid(&mut nodes, 20.0);

        // Should snap to nearest grid points
        assert_eq!(nodes[0].0 % 20.0, 0.0);
        assert_eq!(nodes[0].1 % 20.0, 0.0);
        assert_eq!(nodes[1].0 % 20.0, 0.0);
        assert_eq!(nodes[1].1 % 20.0, 0.0);
    }

    #[test]
    fn test_spiral_search_finds_free() {
        let mut occupied = HashSet::new();
        occupied.insert((5, 5));

        let result = spiral_search(5, 5, &occupied);
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        assert!(!occupied.contains(&(x, y)));
    }
}
