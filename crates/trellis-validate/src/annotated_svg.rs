use trellis_core::grid::Grid;

use crate::report::{DiagramReport, EdgeReport};

// --- Color palette ------------------------------------------------------------

/// Quality score ≥ this value → green (good).
const GREEN_THRESHOLD: f64 = 0.8;
/// Quality score ≥ this value and < GREEN_THRESHOLD → yellow (suboptimal).
const YELLOW_THRESHOLD: f64 = 0.5;

const COLOR_GREEN: &str = "#22c55e";
const COLOR_YELLOW: &str = "#eab308";
const COLOR_RED: &str = "#ef4444";

// --- Public API ---------------------------------------------------------------

/// Inject a diagnostic overlay into an existing rendered SVG.
///
/// For each routed edge in `report`, a semi-transparent `<path>` is added on
/// top of the original diagram.  The color encodes the quality score:
///
/// | Score       | Color  | Meaning         |
/// |-------------|--------|-----------------|
/// | ≥ 0.8       | Green  | Good            |
/// | 0.5 – 0.79  | Yellow | Suboptimal      |
/// | < 0.5       | Red    | Bad             |
///
/// Each path element has:
/// - `data-edge-id="source-->target"` for scripting / selection.
/// - A `<title>` child element with a human-readable metrics summary for
///   browser tooltip support.
///
/// The original SVG content is preserved unchanged; the overlay is appended
/// just before the closing `</svg>` tag.
pub fn annotate_svg(svg: &[u8], report: &DiagramReport, grid: &Grid) -> Vec<u8> {
    let svg_str = match std::str::from_utf8(svg) {
        Ok(s) => s,
        Err(_) => return svg.to_vec(),
    };

    let overlay = build_overlay(report, grid);

    // Insert the overlay group immediately before </svg>.
    let close_tag = "</svg>";
    if let Some(pos) = svg_str.rfind(close_tag) {
        let mut out = String::with_capacity(svg_str.len() + overlay.len() + 16);
        out.push_str(&svg_str[..pos]);
        out.push_str(&overlay);
        out.push_str(close_tag);
        out.into_bytes()
    } else {
        // No closing tag found — return original unchanged.
        svg.to_vec()
    }
}

// --- Private helpers ----------------------------------------------------------

fn build_overlay(report: &DiagramReport, grid: &Grid) -> String {
    let mut out = String::new();
    out.push_str("<g id=\"trellis-diagnostics\" pointer-events=\"stroke\">\n");

    for edge in &report.edges {
        if let Some(path_svg) = edge_overlay_path(edge, grid) {
            out.push_str("  ");
            out.push_str(&path_svg);
            out.push('\n');
        }
    }

    out.push_str("</g>\n");
    out
}

/// Build a single SVG group element for one edge overlay, or `None` if the
/// path has fewer than two points (nothing to draw).
fn edge_overlay_path(edge: &EdgeReport, grid: &Grid) -> Option<String> {
    if edge.path_cells.len() < 2 {
        return None;
    }

    let d = build_path_data(&edge.path_cells, grid);
    let color = quality_color(edge.quality_score);
    let title = build_title(edge);
    let edge_id = escape_xml(&edge.id);

    Some(format!(
        "<g data-edge-id=\"{}\">\
         <title>{}</title>\
         <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"4\" \
         stroke-opacity=\"0.55\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\
         </g>",
        edge_id, title, d, color,
    ))
}

/// Convert `path_cells` (`[row, col]` pairs) to an SVG polyline path string.
///
/// Collinear intermediate points are collapsed so the path only contains
/// start, bend points, and end — matching the shape of the original rendered edge.
fn build_path_data(cells: &[[i64; 2]], grid: &Grid) -> String {
    if cells.is_empty() {
        return String::new();
    }

    // Convert grid cells to world (pixel) coordinates.
    let world: Vec<(f64, f64)> = cells
        .iter()
        .map(|c| {
            // grid_to_world takes usize; clamp negatives to 0 (should not occur
            // in practice for valid paths, but guard against test data).
            let row = c[0].max(0) as usize;
            let col = c[1].max(0) as usize;
            grid.grid_to_world(row, col)
        })
        .collect();

    // Simplify: keep only start, direction-change points, and end.
    let simplified = simplify_world_path(&world);

    if simplified.is_empty() {
        return String::new();
    }

    let mut d = format!("M {:.1} {:.1}", simplified[0].0, simplified[0].1);
    for pt in &simplified[1..] {
        d.push_str(&format!(" L {:.1} {:.1}", pt.0, pt.1));
    }
    d
}

/// Remove collinear intermediate points from a world-coordinate path.
fn simplify_world_path(pts: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }

    let mut out = vec![pts[0]];
    for i in 1..pts.len() - 1 {
        let prev = pts[i - 1];
        let curr = pts[i];
        let next = pts[i + 1];
        if dir_sign(prev, curr) != dir_sign(curr, next) {
            out.push(curr);
        }
    }
    out.push(*pts.last().unwrap());
    out
}

fn dir_sign(from: (f64, f64), to: (f64, f64)) -> (i8, i8) {
    let eps = 0.001;
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let sx = if dx.abs() < eps {
        0
    } else if dx > 0.0 {
        1
    } else {
        -1
    };
    let sy = if dy.abs() < eps {
        0
    } else if dy > 0.0 {
        1
    } else {
        -1
    };
    (sx, sy)
}

fn quality_color(score: f64) -> &'static str {
    if score >= GREEN_THRESHOLD {
        COLOR_GREEN
    } else if score >= YELLOW_THRESHOLD {
        COLOR_YELLOW
    } else {
        COLOR_RED
    }
}

/// Build a plain-text metrics summary for the SVG `<title>` tooltip.
fn build_title(edge: &EdgeReport) -> String {
    let detour = if edge.detour_factor.is_finite() {
        format!("{:.2}", edge.detour_factor)
    } else {
        "∞".to_string()
    };

    let flags = if edge.flags.is_empty() {
        "none".to_string()
    } else {
        edge.flags.join(", ")
    };

    let title = format!(
        "{} | score: {:.2} | bends: {} | detour: {} | crossings: {} | flags: {}",
        edge.id, edge.quality_score, edge.bends, detour, edge.crossings, flags,
    );

    escape_xml(&title)
}

/// Escape the five XML special characters so the string is safe inside an
/// attribute value or element content.
fn escape_xml(s: &str) -> String {
    s.chars()
        .fold(String::with_capacity(s.len()), |mut acc, c| {
            match c {
                '&' => acc.push_str("&amp;"),
                '<' => acc.push_str("&lt;"),
                '>' => acc.push_str("&gt;"),
                '"' => acc.push_str("&quot;"),
                '\'' => acc.push_str("&apos;"),
                other => acc.push(other),
            }
            acc
        })
}

// --- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{DiagramReport, EdgeReport, GlobalMetrics};
    use trellis_core::grid::Grid;

    fn make_report(edges: Vec<EdgeReport>) -> DiagramReport {
        DiagramReport {
            fixture: "test.mmd".to_string(),
            edges,
            global_metrics: GlobalMetrics {
                total_edges: 0,
                routed_edges: 0,
                failed_edges: 0,
                total_crossings: 0,
                total_bends: 0,
                avg_quality_score: 0.0,
                avg_detour_factor: 0.0,
                flagged_edges: 0,
            },
        }
    }

    fn make_edge_report(id: &str, score: f64, cells: Vec<[i64; 2]>) -> EdgeReport {
        EdgeReport {
            id: id.to_string(),
            source: id.split("-->").next().unwrap_or("A").to_string(),
            target: id.split("-->").nth(1).unwrap_or("B").to_string(),
            bends: 0,
            detour_factor: 1.0,
            crossings: 0,
            port_side_source: "South".to_string(),
            port_side_target: "North".to_string(),
            path_cells: cells,
            quality_score: score,
            flags: vec![],
        }
    }

    fn make_svg(body: &str) -> Vec<u8> {
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{}</svg>", body).into_bytes()
    }

    #[test]
    fn overlay_inserted_before_closing_tag() {
        let svg = make_svg("<rect/>");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let report = make_report(vec![]);

        let out = annotate_svg(&svg, &report, &grid);
        let s = String::from_utf8(out).unwrap();

        assert!(s.ends_with("</svg>"));
        assert!(s.contains("trellis-diagnostics"));
    }

    #[test]
    fn good_edge_gets_green() {
        let svg = make_svg("");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge_report("A-->B", 0.95, vec![[0, 0], [1, 0], [2, 0]]);
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        assert!(out.contains(COLOR_GREEN), "expected green for score 0.95");
    }

    #[test]
    fn suboptimal_edge_gets_yellow() {
        let svg = make_svg("");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge_report("A-->B", 0.65, vec![[0, 0], [1, 0]]);
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        assert!(out.contains(COLOR_YELLOW), "expected yellow for score 0.65");
    }

    #[test]
    fn bad_edge_gets_red() {
        let svg = make_svg("");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge_report("A-->B", 0.3, vec![[0, 0], [1, 0]]);
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        assert!(out.contains(COLOR_RED), "expected red for score 0.3");
    }

    #[test]
    fn data_edge_id_present() {
        let svg = make_svg("");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge_report("Foo-->Bar", 0.9, vec![[0, 0], [0, 1]]);
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        assert!(out.contains("data-edge-id=\"Foo--&gt;Bar\""));
    }

    #[test]
    fn title_tooltip_contains_metrics() {
        let svg = make_svg("");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge_report("A-->B", 0.42, vec![[0, 0], [1, 0]]);
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        assert!(out.contains("<title>"));
        assert!(out.contains("score:"));
        assert!(out.contains("bends:"));
        assert!(out.contains("detour:"));
    }

    #[test]
    fn edge_with_single_point_skipped() {
        let svg = make_svg("<rect/>");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge_report("A-->B", 0.9, vec![[0, 0]]); // only one cell
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        // Group present but no path drawn for single-point edge
        assert!(!out.contains("<path"));
    }

    #[test]
    fn invalid_utf8_returns_original() {
        let bad: Vec<u8> = vec![0xFF, 0xFE, 0x00];
        let grid = Grid::new(5, 5, 10, 0, 0);
        let report = make_report(vec![]);

        let out = annotate_svg(&bad, &report, &grid);

        assert_eq!(out, bad);
    }

    #[test]
    fn xml_special_chars_escaped_in_edge_id() {
        let svg = make_svg("");
        let grid = Grid::new(10, 10, 10, 0, 0);
        let mut edge = make_edge_report("A&B-->C", 0.9, vec![[0, 0], [1, 0]]);
        edge.id = "A&B-->C".to_string();
        let report = make_report(vec![edge]);

        let out = String::from_utf8(annotate_svg(&svg, &report, &grid)).unwrap();

        assert!(out.contains("&amp;"), "& must be escaped");
        assert!(
            !out.contains("data-edge-id=\"A&B"),
            "raw & must not appear in attribute"
        );
    }
}
