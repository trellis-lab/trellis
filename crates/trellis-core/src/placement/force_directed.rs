use std::collections::HashMap;

use super::algorithm::LayoutAlgorithm;

/// Configuration for the Fruchterman-Reingold force-directed layout.
pub struct ForceDirectedConfig {
    /// Scale factor for the layout area: area = area_factor * node_count
    pub area_factor: f64,
    /// Temperature cooling rate per iteration (0 < rate < 1)
    pub cooling_rate: f64,
    /// Maximum number of layout iterations
    pub max_iterations: u32,
}

impl Default for ForceDirectedConfig {
    fn default() -> Self {
        Self {
            area_factor: 80_000.0,
            cooling_rate: 0.95,
            max_iterations: 100,
        }
    }
}

/// Fruchterman-Reingold force-directed layout algorithm.
///
/// Returns a map from node ID to `(x, y)` position (top-left corner).
///
/// Algorithm:
/// - Repulsive force between every pair of nodes: `K² / distance`
/// - Attractive force along every edge: `distance² / K`
/// - K = sqrt(area / n) — optimal pairwise distance
/// - Temperature starts at `width / 10` and decreases by `cooling_rate` each iteration.
pub fn force_directed_placement(
    node_ids: &[String],
    edges: &[(String, String)],
    config: &ForceDirectedConfig,
) -> HashMap<String, (f64, f64)> {
    let n = node_ids.len();
    if n == 0 {
        return HashMap::new();
    }

    let area = config.area_factor * n as f64;
    let side = area.sqrt();
    // Optimal distance between connected nodes
    let k = (area / n as f64).sqrt();

    // Index map for O(1) lookup
    let id_to_idx: HashMap<&str, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    // Initialize positions in a circle
    let mut pos: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            let r = side / 3.0;
            let cx = side / 2.0;
            let cy = side / 2.0;
            (cx + r * angle.cos(), cy + r * angle.sin())
        })
        .collect();

    let mut temperature = side / 10.0;

    for _ in 0..config.max_iterations {
        let mut disp: Vec<(f64, f64)> = vec![(0.0, 0.0); n];

        // Repulsive forces: every pair
        for i in 0..n {
            for j in (i + 1)..n {
                let (xi, yi) = pos[i];
                let (xj, yj) = pos[j];
                let dx = xi - xj;
                let dy = yi - yj;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = k * k / dist;
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                disp[i].0 += fx;
                disp[i].1 += fy;
                disp[j].0 -= fx;
                disp[j].1 -= fy;
            }
        }

        // Attractive forces: along edges
        for (from, to) in edges {
            let Some(&fi) = id_to_idx.get(from.as_str()) else {
                continue;
            };
            let Some(&ti) = id_to_idx.get(to.as_str()) else {
                continue;
            };
            let (xi, yi) = pos[fi];
            let (xj, yj) = pos[ti];
            let dx = xi - xj;
            let dy = yi - yj;
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let force = dist * dist / k;
            let fx = force * dx / dist;
            let fy = force * dy / dist;
            disp[fi].0 -= fx;
            disp[fi].1 -= fy;
            disp[ti].0 += fx;
            disp[ti].1 += fy;
        }

        // Apply displacements, limited by temperature and clamped to area
        for i in 0..n {
            let (dx, dy) = disp[i];
            let disp_len = (dx * dx + dy * dy).sqrt().max(0.001);
            let factor = disp_len.min(temperature) / disp_len;
            let (x, y) = &mut pos[i];
            *x = (*x + dx * factor).clamp(0.0, side);
            *y = (*y + dy * factor).clamp(0.0, side);
        }

        temperature *= config.cooling_rate;
    }

    // Build output map
    node_ids
        .iter()
        .zip(pos.iter())
        .map(|(id, &(x, y))| (id.clone(), (x, y)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let result = force_directed_placement(&[], &[], &ForceDirectedConfig::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_node() {
        let ids = vec!["A".to_string()];
        let result = force_directed_placement(&ids, &[], &ForceDirectedConfig::default());
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("A"));
    }

    #[test]
    fn test_two_connected_nodes_are_near_each_other() {
        let ids = vec!["A".to_string(), "B".to_string()];
        let edges = vec![("A".to_string(), "B".to_string())];
        let config = ForceDirectedConfig {
            area_factor: 10_000.0,
            cooling_rate: 0.9,
            max_iterations: 200,
        };
        let result = force_directed_placement(&ids, &edges, &config);
        let (ax, ay) = result["A"];
        let (bx, by) = result["B"];
        let dist = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
        // After convergence, connected nodes should be somewhat close
        assert!(dist < 500.0, "dist = {}", dist);
    }

    #[test]
    fn test_converges_without_panic_large_graph() {
        let n = 20;
        let ids: Vec<String> = (0..n).map(|i| format!("N{}", i)).collect();
        let edges: Vec<(String, String)> = (0..n - 1)
            .map(|i| (format!("N{}", i), format!("N{}", i + 1)))
            .collect();
        let result = force_directed_placement(&ids, &edges, &ForceDirectedConfig::default());
        assert_eq!(result.len(), n);
    }
}

/// Handle for the Fruchterman-Reingold force-directed layout algorithm.
///
/// Construct with an explicit [`ForceDirectedConfig`] or use
/// `ForceDirectedLayout { config: ForceDirectedConfig::default() }`.
pub struct ForceDirectedLayout {
    pub config: ForceDirectedConfig,
}

impl LayoutAlgorithm for ForceDirectedLayout {
    fn layout(&self, graph: &mut trellis_parser::Graph) {
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let edges: Vec<(String, String)> = graph
            .edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone()))
            .collect();
        let positions = force_directed_placement(&node_ids, &edges, &self.config);
        for node in &mut graph.nodes {
            if let Some(&(x, y)) = positions.get(&node.id) {
                node.x = x;
                node.y = y;
            }
        }
    }
}
