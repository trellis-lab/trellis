//! Diagram color themes.
//!
//! A [`Theme`] is a flat set of semantic color tokens consumed by every render
//! module.  The six built-in themes are stored as `&'static Theme` references —
//! zero heap allocation at render time.
//!
//! Select a theme via [`ThemeName`] in [`TrellisConfig`](crate::config::TrellisConfig).

use serde::{Deserialize, Serialize};

// ── Token struct ──────────────────────────────────────────────────────────────

/// Flat collection of semantic color tokens for one theme.
///
/// All values are CSS hex strings (e.g. `"#ff0000"` or `"white"`).
/// Stroke widths are geometry constants and are **not** part of the theme.
#[derive(Debug, Clone)]
pub struct Theme {
    // ── Canvas ────────────────────────────────────────────────────────────────
    pub background: &'static str,

    // ── Generic grid helper ───────────────────────────────────────────────────
    pub grid_dot: &'static str,

    // ── Crossing decoration ───────────────────────────────────────────────────
    pub crossing_bg: &'static str,
    pub crossing_stroke: &'static str,

    // ── Flowchart shape roles ─────────────────────────────────────────────────
    /// Process: Rectangle, RoundedRect, Stadium, Subroutine, Asymmetric, Parallelogram, Trapezoid
    pub shape_process_fill: &'static str,
    pub shape_process_stroke: &'static str,
    /// Decision: Diamond
    pub shape_decision_fill: &'static str,
    pub shape_decision_stroke: &'static str,
    /// Terminal: Circle, DoubleCircle
    pub shape_terminal_fill: &'static str,
    pub shape_terminal_stroke: &'static str,
    /// Storage: Cylinder body fill, top-face fill, stroke
    pub shape_storage_fill: &'static str,
    pub shape_storage_top_fill: &'static str,
    pub shape_storage_stroke: &'static str,
    /// Special: Hexagon
    pub shape_special_fill: &'static str,
    pub shape_special_stroke: &'static str,
    /// Node label text (default; overridden per diagram type below)
    pub node_text: &'static str,
    /// Fallback box fill/stroke (ClassBox/ErBox/C4Box rendered by generic path)
    pub fallback_box_fill: &'static str,
    pub fallback_box_stroke: &'static str,

    // ── Edges ─────────────────────────────────────────────────────────────────
    pub edge_stroke: &'static str,
    pub edge_fallback_stroke: &'static str,
    pub arrow_fill: &'static str,

    // ── Edge labels ───────────────────────────────────────────────────────────
    pub edge_label_text: &'static str,
    pub edge_label_bg: &'static str,
    pub edge_label_border: &'static str,
    pub multiplicity_text: &'static str,

    // ── Class diagram ─────────────────────────────────────────────────────────
    pub class_box_fill: &'static str,
    pub class_box_stroke: &'static str,
    pub class_header_text: &'static str,
    pub class_member_text: &'static str,
    pub class_separator: &'static str,
    /// Interior fill of hollow arrow markers (inheritance, aggregation)
    pub class_marker_fill: &'static str,

    // ── ER diagram ────────────────────────────────────────────────────────────
    pub er_box_fill: &'static str,
    pub er_box_stroke: &'static str,
    pub er_entity_text: &'static str,
    pub er_attribute_text: &'static str,

    // ── C4 elements (fill, stroke used also as text per C4 v4 convention) ────
    pub c4_person_fill: &'static str,
    pub c4_person_stroke: &'static str,
    pub c4_system_fill: &'static str,
    pub c4_system_stroke: &'static str,
    pub c4_container_fill: &'static str,
    pub c4_container_stroke: &'static str,
    pub c4_component_fill: &'static str,
    pub c4_component_stroke: &'static str,
    pub c4_external_fill: &'static str,
    pub c4_external_stroke: &'static str,
    pub c4_deployment_fill: &'static str,
    pub c4_deployment_stroke: &'static str,
    /// Fallback text for C4 elements when fill is dark (dark themes)
    pub c4_text_on_dark: &'static str,

    // ── C4 boundaries ─────────────────────────────────────────────────────────
    pub c4_boundary_enterprise_fill: &'static str,
    pub c4_boundary_enterprise_stroke: &'static str,
    pub c4_boundary_system_fill: &'static str,
    pub c4_boundary_system_stroke: &'static str,
    pub c4_boundary_container_fill: &'static str,
    pub c4_boundary_container_stroke: &'static str,
    pub c4_boundary_deployment_fill: &'static str,
    pub c4_boundary_deployment_stroke: &'static str,

    // ── Subgraph (4 depth levels, index = depth, clamped at 3) ───────────────
    pub subgraph_fill: [&'static str; 4],
    pub subgraph_stroke: [&'static str; 4],
    pub subgraph_label: [&'static str; 4],
}

// ── Theme selector ────────────────────────────────────────────────────────────

/// Named color theme for diagrams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    /// Codified version of the original hardcoded palette (zero visual change).
    #[default]
    Default,
    /// Warm off-white canvas, sepia tones — document-export feel.
    Paper,
    /// Saturated blue family throughout — presentation-ready.
    Blueprint,
    /// VS Code-style dark (`#1e1e1e` canvas).
    Dark,
    /// GitHub-dark (`#0d1117` canvas), cyan accents.
    Midnight,
    /// Dark green canvas, earth-tone palette.
    Forest,
    /// Classic blueprint colors
    ClassicBlueprint,
}

impl ThemeName {
    /// Return a reference to the corresponding static [`Theme`] instance.
    pub fn resolve(self) -> &'static Theme {
        match self {
            ThemeName::Default => &themes::DEFAULT,
            ThemeName::Paper => &themes::PAPER,
            ThemeName::Blueprint => &themes::BLUEPRINT,
            ThemeName::Dark => &themes::DARK,
            ThemeName::Midnight => &themes::MIDNIGHT,
            ThemeName::Forest => &themes::FOREST,
            ThemeName::ClassicBlueprint => &themes::CLASSIC,
        }
    }
}

// ── Static theme instances ────────────────────────────────────────────────────

pub mod themes {
    use super::Theme;

    // ── L1 — default ─────────────────────────────────────────────────────────
    /// Current hardcoded palette, codified.  Zero visual change on upgrade.
    pub static DEFAULT: Theme = Theme {
        background: "white",
        grid_dot: "grey",
        crossing_bg: "white",
        crossing_stroke: "#666",

        shape_process_fill: "#e8f4fd",
        shape_process_stroke: "#4a90d9",
        shape_decision_fill: "#fff3e0",
        shape_decision_stroke: "#e67e22",
        shape_terminal_fill: "#e8f5e9",
        shape_terminal_stroke: "#43a047",
        shape_storage_fill: "#e8f4fd",
        shape_storage_top_fill: "#cce5ff",
        shape_storage_stroke: "#4a90d9",
        shape_special_fill: "#f3e5f5",
        shape_special_stroke: "#8e24aa",
        node_text: "#333",
        fallback_box_fill: "#f5f5f5",
        fallback_box_stroke: "#555",

        edge_stroke: "#555",
        edge_fallback_stroke: "#ccc",
        arrow_fill: "#666",
        edge_label_text: "#333",
        edge_label_bg: "white",
        edge_label_border: "#ccc",
        multiplicity_text: "#555",

        class_box_fill: "#f5f5f5",
        class_box_stroke: "#555",
        class_header_text: "#111",
        class_member_text: "#333",
        class_separator: "#555",
        class_marker_fill: "white",

        er_box_fill: "#f0f7ff",
        er_box_stroke: "#336699",
        er_entity_text: "#111",
        er_attribute_text: "#333",

        c4_person_fill: "#ffffff",
        c4_person_stroke: "#08427b",
        c4_system_fill: "#ffffff",
        c4_system_stroke: "#1168bd",
        c4_container_fill: "#ffffff",
        c4_container_stroke: "#438dd5",
        c4_component_fill: "#ffffff",
        c4_component_stroke: "#85bbf0",
        c4_external_fill: "#ffffff",
        c4_external_stroke: "#999999",
        c4_deployment_fill: "#ffffff",
        c4_deployment_stroke: "#1c1c1c",
        c4_text_on_dark: "#ffffff",

        c4_boundary_enterprise_fill: "#ffffff",
        c4_boundary_enterprise_stroke: "#f9a825",
        c4_boundary_system_fill: "#ffffff",
        c4_boundary_system_stroke: "#388e3c",
        c4_boundary_container_fill: "#ffffff",
        c4_boundary_container_stroke: "#1565c0",
        c4_boundary_deployment_fill: "#ffffff",
        c4_boundary_deployment_stroke: "#616161",

        subgraph_fill: ["#f0f4f8", "#e2e8f0", "#cbd5e1", "#94a3b8"],
        subgraph_stroke: ["#b0bec5", "#90a4ae", "#78909c", "#607d8b"],
        subgraph_label: ["#546e7a", "#455a64", "#37474f", "#263238"],
    };

    // ── L2 — paper ───────────────────────────────────────────────────────────
    /// Warm off-white canvas, sepia tones.  Suitable for documentation exports.
    pub static PAPER: Theme = Theme {
        background: "#faf8f5",
        grid_dot: "#c8b89a",
        crossing_bg: "#faf8f5",
        crossing_stroke: "#8b7355",

        shape_process_fill: "#f0ece4",
        shape_process_stroke: "#8b7355",
        shape_decision_fill: "#fef3c7",
        shape_decision_stroke: "#d97706",
        shape_terminal_fill: "#ecfdf5",
        shape_terminal_stroke: "#059669",
        shape_storage_fill: "#f0ece4",
        shape_storage_top_fill: "#e4ddd0",
        shape_storage_stroke: "#8b7355",
        shape_special_fill: "#fdf4ff",
        shape_special_stroke: "#9333ea",
        node_text: "#3d2b1a",
        fallback_box_fill: "#f5f0e8",
        fallback_box_stroke: "#7c6a56",

        edge_stroke: "#6b5744",
        edge_fallback_stroke: "#d4c4b0",
        arrow_fill: "#7c6a56",
        edge_label_text: "#3d2b1a",
        edge_label_bg: "#faf8f5",
        edge_label_border: "#c8b89a",
        multiplicity_text: "#6b5744",

        class_box_fill: "#f5f0e8",
        class_box_stroke: "#7c6a56",
        class_header_text: "#1a0f00",
        class_member_text: "#3d2b1a",
        class_separator: "#7c6a56",
        class_marker_fill: "#faf8f5",

        er_box_fill: "#fef9ef",
        er_box_stroke: "#a0845c",
        er_entity_text: "#1a0f00",
        er_attribute_text: "#3d2b1a",

        c4_person_fill: "#faf8f5",
        c4_person_stroke: "#5c3d1e",
        c4_system_fill: "#faf8f5",
        c4_system_stroke: "#7c5c2e",
        c4_container_fill: "#faf8f5",
        c4_container_stroke: "#a0845c",
        c4_component_fill: "#faf8f5",
        c4_component_stroke: "#c8a882",
        c4_external_fill: "#faf8f5",
        c4_external_stroke: "#a09080",
        c4_deployment_fill: "#faf8f5",
        c4_deployment_stroke: "#4a3828",
        c4_text_on_dark: "#faf8f5",

        c4_boundary_enterprise_fill: "#fefcf0",
        c4_boundary_enterprise_stroke: "#d97706",
        c4_boundary_system_fill: "#f0fdf4",
        c4_boundary_system_stroke: "#059669",
        c4_boundary_container_fill: "#faf8f5",
        c4_boundary_container_stroke: "#7c5c2e",
        c4_boundary_deployment_fill: "#faf8f5",
        c4_boundary_deployment_stroke: "#7c6a56",

        subgraph_fill: ["#f0ece4", "#e8e0d8", "#ddd4c8", "#c8bca8"],
        subgraph_stroke: ["#c8b89a", "#b0a088", "#9a8870", "#806858"],
        subgraph_label: ["#6b5744", "#5c4834", "#4d3c28", "#3d2b1a"],
    };

    // ── L3 — blueprint ────────────────────────────────────────────────────────
    /// Saturated blue palette throughout.  High contrast, presentation-ready.
    pub static BLUEPRINT: Theme = Theme {
        background: "#f8faff",
        grid_dot: "#93c5fd",
        crossing_bg: "#f8faff",
        crossing_stroke: "#374151",

        shape_process_fill: "#dbeafe",
        shape_process_stroke: "#2563eb",
        shape_decision_fill: "#ede9fe",
        shape_decision_stroke: "#7c3aed",
        shape_terminal_fill: "#d1fae5",
        shape_terminal_stroke: "#059669",
        shape_storage_fill: "#dbeafe",
        shape_storage_top_fill: "#bfdbfe",
        shape_storage_stroke: "#2563eb",
        shape_special_fill: "#fce7f3",
        shape_special_stroke: "#db2777",
        node_text: "#1e3a5f",
        fallback_box_fill: "#eff6ff",
        fallback_box_stroke: "#1d4ed8",

        edge_stroke: "#374151",
        edge_fallback_stroke: "#93c5fd",
        arrow_fill: "#374151",
        edge_label_text: "#1e3a5f",
        edge_label_bg: "#f8faff",
        edge_label_border: "#93c5fd",
        multiplicity_text: "#374151",

        class_box_fill: "#eff6ff",
        class_box_stroke: "#1d4ed8",
        class_header_text: "#1e3a5f",
        class_member_text: "#374151",
        class_separator: "#1d4ed8",
        class_marker_fill: "#f8faff",

        er_box_fill: "#eff6ff",
        er_box_stroke: "#1e40af",
        er_entity_text: "#1e3a5f",
        er_attribute_text: "#374151",

        c4_person_fill: "#eff6ff",
        c4_person_stroke: "#1e40af",
        c4_system_fill: "#dbeafe",
        c4_system_stroke: "#1d4ed8",
        c4_container_fill: "#eff6ff",
        c4_container_stroke: "#2563eb",
        c4_component_fill: "#f0f9ff",
        c4_component_stroke: "#0369a1",
        c4_external_fill: "#f1f5f9",
        c4_external_stroke: "#64748b",
        c4_deployment_fill: "#f8faff",
        c4_deployment_stroke: "#1e293b",
        c4_text_on_dark: "#f8faff",

        c4_boundary_enterprise_fill: "#fefce8",
        c4_boundary_enterprise_stroke: "#ca8a04",
        c4_boundary_system_fill: "#f0fdf4",
        c4_boundary_system_stroke: "#16a34a",
        c4_boundary_container_fill: "#eff6ff",
        c4_boundary_container_stroke: "#1d4ed8",
        c4_boundary_deployment_fill: "#f8faff",
        c4_boundary_deployment_stroke: "#475569",

        subgraph_fill: ["#dbeafe", "#bfdbfe", "#93c5fd", "#60a5fa"],
        subgraph_stroke: ["#3b82f6", "#2563eb", "#1d4ed8", "#1e40af"],
        subgraph_label: ["#1e40af", "#1d4ed8", "#1e3a8a", "#172554"],
    };

    // ── D1 — dark ─────────────────────────────────────────────────────────────
    /// VS Code-style dark (`#1e1e1e` canvas).  The canonical dark mode.
    pub static DARK: Theme = Theme {
        background: "#1e1e1e",
        grid_dot: "#3d3d3d",
        crossing_bg: "#1e1e1e",
        crossing_stroke: "#9e9e9e",

        shape_process_fill: "#1e3a5f",
        shape_process_stroke: "#5b9bd5",
        shape_decision_fill: "#3d2b00",
        shape_decision_stroke: "#e0a020",
        shape_terminal_fill: "#1a3d2b",
        shape_terminal_stroke: "#4caf79",
        shape_storage_fill: "#1e3a5f",
        shape_storage_top_fill: "#0f2a4a",
        shape_storage_stroke: "#5b9bd5",
        shape_special_fill: "#2d1a40",
        shape_special_stroke: "#b07de0",
        node_text: "#e0e0e0",
        fallback_box_fill: "#2a2a2a",
        fallback_box_stroke: "#888888",

        edge_stroke: "#9e9e9e",
        edge_fallback_stroke: "#555555",
        arrow_fill: "#9e9e9e",
        edge_label_text: "#e0e0e0",
        edge_label_bg: "#2a2a2a",
        edge_label_border: "#555555",
        multiplicity_text: "#9e9e9e",

        class_box_fill: "#2a2a2a",
        class_box_stroke: "#888888",
        class_header_text: "#e0e0e0",
        class_member_text: "#bdbdbd",
        class_separator: "#555555",
        class_marker_fill: "#1e1e1e",

        er_box_fill: "#1a2d40",
        er_box_stroke: "#5b8db8",
        er_entity_text: "#e0e0e0",
        er_attribute_text: "#bdbdbd",

        c4_person_fill: "#1e3a5f",
        c4_person_stroke: "#5b9bd5",
        c4_system_fill: "#1a2d40",
        c4_system_stroke: "#4a7ab0",
        c4_container_fill: "#1e3040",
        c4_container_stroke: "#6a9fc0",
        c4_component_fill: "#1a2535",
        c4_component_stroke: "#8ab0d0",
        c4_external_fill: "#2a2a2a",
        c4_external_stroke: "#777777",
        c4_deployment_fill: "#222222",
        c4_deployment_stroke: "#aaaaaa",
        c4_text_on_dark: "#e0e0e0",

        c4_boundary_enterprise_fill: "#1e1e1e",
        c4_boundary_enterprise_stroke: "#c8860a",
        c4_boundary_system_fill: "#1e1e1e",
        c4_boundary_system_stroke: "#3a8a4a",
        c4_boundary_container_fill: "#1e1e1e",
        c4_boundary_container_stroke: "#3a6a9a",
        c4_boundary_deployment_fill: "#1e1e1e",
        c4_boundary_deployment_stroke: "#666666",

        subgraph_fill: ["#252525", "#2d2d2d", "#353535", "#3d3d3d"],
        subgraph_stroke: ["#555555", "#666666", "#777777", "#888888"],
        subgraph_label: ["#bdbdbd", "#c8c8c8", "#d0d0d0", "#e0e0e0"],
    };

    // ── D2 — midnight ─────────────────────────────────────────────────────────
    /// GitHub-dark (`#0d1117`) canvas.  Low eye-strain, cyan accents.
    pub static MIDNIGHT: Theme = Theme {
        background: "#0d1117",
        grid_dot: "#30363d",
        crossing_bg: "#0d1117",
        crossing_stroke: "#8b949e",

        shape_process_fill: "#0d2137",
        shape_process_stroke: "#58a6ff",
        shape_decision_fill: "#1a1000",
        shape_decision_stroke: "#e3b341",
        shape_terminal_fill: "#0d2618",
        shape_terminal_stroke: "#3fb950",
        shape_storage_fill: "#0d2137",
        shape_storage_top_fill: "#071526",
        shape_storage_stroke: "#58a6ff",
        shape_special_fill: "#1e0d2e",
        shape_special_stroke: "#bc8cff",
        node_text: "#c9d1d9",
        fallback_box_fill: "#161b22",
        fallback_box_stroke: "#30363d",

        edge_stroke: "#8b949e",
        edge_fallback_stroke: "#30363d",
        arrow_fill: "#8b949e",
        edge_label_text: "#c9d1d9",
        edge_label_bg: "#161b22",
        edge_label_border: "#30363d",
        multiplicity_text: "#8b949e",

        class_box_fill: "#161b22",
        class_box_stroke: "#30363d",
        class_header_text: "#c9d1d9",
        class_member_text: "#8b949e",
        class_separator: "#30363d",
        class_marker_fill: "#0d1117",

        er_box_fill: "#0d1e2e",
        er_box_stroke: "#388bfd",
        er_entity_text: "#c9d1d9",
        er_attribute_text: "#8b949e",

        c4_person_fill: "#0d2137",
        c4_person_stroke: "#58a6ff",
        c4_system_fill: "#0d1e2e",
        c4_system_stroke: "#388bfd",
        c4_container_fill: "#0d1e30",
        c4_container_stroke: "#58a6ff",
        c4_component_fill: "#0d1a25",
        c4_component_stroke: "#79c0ff",
        c4_external_fill: "#161b22",
        c4_external_stroke: "#6e7681",
        c4_deployment_fill: "#161b22",
        c4_deployment_stroke: "#8b949e",
        c4_text_on_dark: "#c9d1d9",

        c4_boundary_enterprise_fill: "#0d1117",
        c4_boundary_enterprise_stroke: "#e3b341",
        c4_boundary_system_fill: "#0d1117",
        c4_boundary_system_stroke: "#3fb950",
        c4_boundary_container_fill: "#0d1117",
        c4_boundary_container_stroke: "#388bfd",
        c4_boundary_deployment_fill: "#0d1117",
        c4_boundary_deployment_stroke: "#6e7681",

        subgraph_fill: ["#161b22", "#1c2128", "#212830", "#272e38"],
        subgraph_stroke: ["#30363d", "#3d444f", "#484f58", "#535b62"],
        subgraph_label: ["#8b949e", "#9ba3ab", "#adb5bc", "#c9d1d9"],
    };

    // ── D3 — forest ───────────────────────────────────────────────────────────
    /// Dark green canvas.  Earth tones, warm amber decisions, moss green terminals.
    pub static FOREST: Theme = Theme {
        background: "#0f1a0f",
        grid_dot: "#1e3a1e",
        crossing_bg: "#0f1a0f",
        crossing_stroke: "#86efac",

        shape_process_fill: "#0d2210",
        shape_process_stroke: "#4ade80",
        shape_decision_fill: "#1f1500",
        shape_decision_stroke: "#fbbf24",
        shape_terminal_fill: "#3057E1",
        shape_terminal_stroke: "#34d399",
        shape_storage_fill: "#0d2210",
        shape_storage_top_fill: "#081508",
        shape_storage_stroke: "#4ade80",
        shape_special_fill: "#1a0d22",
        shape_special_stroke: "#c084fc",
        node_text: "#d1fae5",
        fallback_box_fill: "#111a11",
        fallback_box_stroke: "#4ade80",

        edge_stroke: "#86efac",
        edge_fallback_stroke: "#1e3a1e",
        arrow_fill: "#86efac",
        edge_label_text: "#d1fae5",
        edge_label_bg: "#111a11",
        edge_label_border: "#1e3a1e",
        multiplicity_text: "#86efac",

        class_box_fill: "#111a11",
        class_box_stroke: "#4ade80",
        class_header_text: "#d1fae5",
        class_member_text: "#86efac",
        class_separator: "#1e3a1e",
        class_marker_fill: "#0f1a0f",

        er_box_fill: "#0d1a0d",
        er_box_stroke: "#22c55e",
        er_entity_text: "#d1fae5",
        er_attribute_text: "#86efac",

        c4_person_fill: "#0d2210",
        c4_person_stroke: "#4ade80",
        c4_system_fill: "#0d1a0d",
        c4_system_stroke: "#22c55e",
        c4_container_fill: "#0d2210",
        c4_container_stroke: "#4ade80",
        c4_component_fill: "#0a180a",
        c4_component_stroke: "#86efac",
        c4_external_fill: "#111a11",
        c4_external_stroke: "#6b7280",
        c4_deployment_fill: "#111a11",
        c4_deployment_stroke: "#9ca3af",
        c4_text_on_dark: "#d1fae5",

        c4_boundary_enterprise_fill: "#0f1a0f",
        c4_boundary_enterprise_stroke: "#fbbf24",
        c4_boundary_system_fill: "#0f1a0f",
        c4_boundary_system_stroke: "#4ade80",
        c4_boundary_container_fill: "#0f1a0f",
        c4_boundary_container_stroke: "#22c55e",
        c4_boundary_deployment_fill: "#0f1a0f",
        c4_boundary_deployment_stroke: "#6b7280",

        subgraph_fill: ["#111a11", "#162216", "#1a2a1a", "#1e321e"],
        subgraph_stroke: ["#1e3a1e", "#265226", "#2e6a2e", "#38823e"],
        subgraph_label: ["#86efac", "#a7f3c0", "#b8f5cc", "#d1fae5"],
    };

    // ── C1 — classic blueprint ───────────────────────────────────────────────────────────
    /// Dark green canvas.  Earth tones, warm amber decisions, moss green terminals.
    pub static CLASSIC: Theme = Theme {
        background: "#3057E1",
        grid_dot: "#CED8F7",
        crossing_bg: "#3057E1",
        crossing_stroke: "#CED8F7",

        shape_process_fill: "#3057E1",
        shape_process_stroke: "#CED8F7",
        shape_decision_fill: "#3057E1",
        shape_decision_stroke: "#CED8F7",
        shape_terminal_fill: "#3057E1",
        shape_terminal_stroke: "#CED8F7",
        shape_storage_fill: "#3057E1",
        shape_storage_top_fill: "#3057E1",
        shape_storage_stroke: "#CED8F7",
        shape_special_fill: "#3057E1",
        shape_special_stroke: "#CED8F7",
        node_text: "#CED8F7",
        fallback_box_fill: "#3057E1",
        fallback_box_stroke: "#CED8F7",

        edge_stroke: "#CED8F7",
        edge_fallback_stroke: "#CED8F7",
        arrow_fill: "#CED8F7",
        edge_label_text: "#CED8F7",
        edge_label_bg: "#3057E1",
        edge_label_border: "#CED8F7",
        multiplicity_text: "#CED8F7",

        class_box_fill: "#3057E1",
        class_box_stroke: "#CED8F7",
        class_header_text: "#CED8F7",
        class_member_text: "#CED8F7",
        class_separator: "#CED8F7",
        class_marker_fill: "#3057E1",

        er_box_fill: "#3057E1",
        er_box_stroke: "#CED8F7",
        er_entity_text: "#CED8F7",
        er_attribute_text: "#CED8F7",

        c4_person_fill: "#3057E1",
        c4_person_stroke: "#CED8F7",
        c4_system_fill: "#3057E1",
        c4_system_stroke: "#CED8F7",
        c4_container_fill: "#3057E1",
        c4_container_stroke: "#CED8F7",
        c4_component_fill: "#3057E1",
        c4_component_stroke: "#CED8F7",
        c4_external_fill: "#3057E1",
        c4_external_stroke: "#CED8F7",
        c4_deployment_fill: "#3057E1",
        c4_deployment_stroke: "#CED8F7",
        c4_text_on_dark: "#CED8F7",

        c4_boundary_enterprise_fill: "#3057E1",
        c4_boundary_enterprise_stroke: "#CED8F7",
        c4_boundary_system_fill: "#3057E1",
        c4_boundary_system_stroke: "#CED8F7",
        c4_boundary_container_fill: "#3057E1",
        c4_boundary_container_stroke: "#CED8F7",
        c4_boundary_deployment_fill: "#3057E1",
        c4_boundary_deployment_stroke: "#CED8F7",

        subgraph_fill: ["#3057E1", "#3057E1", "#3057E1", "#3057E1"],
        subgraph_stroke: ["#CED8F7", "#CED8F7", "#CED8F7", "#CED8F7"],
        subgraph_label: ["#CED8F7", "#CED8F7", "#CED8F7", "#CED8F7"],
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_name_default_is_default() {
        assert_eq!(ThemeName::default(), ThemeName::Default);
    }

    #[test]
    fn all_six_themes_resolve() {
        let names = [
            ThemeName::Default,
            ThemeName::Paper,
            ThemeName::Blueprint,
            ThemeName::Dark,
            ThemeName::Midnight,
            ThemeName::Forest,
            ThemeName::ClassicBlueprint,
        ];
        for name in names {
            let theme = name.resolve();
            // Background must be a non-empty string
            assert!(!theme.background.is_empty());
        }
    }

    #[test]
    fn theme_name_serde_roundtrip() {
        let json = serde_json::to_string(&ThemeName::Dark).unwrap();
        assert_eq!(json, "\"dark\"");
        let back: ThemeName = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ThemeName::Dark);
    }

    #[test]
    fn default_theme_background_is_white() {
        assert_eq!(ThemeName::Default.resolve().background, "white");
    }

    #[test]
    fn dark_theme_background_is_dark() {
        let bg = ThemeName::Dark.resolve().background;
        // Must not be "white"
        assert_ne!(bg, "white");
    }
}
