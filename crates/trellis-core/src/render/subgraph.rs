use std::collections::HashMap;

use crate::types::{BoundingBox, SubgraphTree};

const BORDER_RADIUS: f64 = 8.0;
const STROKE_WIDTH: f64 = 1.5;
const STROKE_DASHARRAY: &str = "5,3";
const LABEL_FONT_SIZE: f64 = 13.0;
const LABEL_OFFSET_X: f64 = 8.0;
const LABEL_OFFSET_Y: f64 = 16.0;

/// Background colors by depth (lighter = shallower)
const BG_COLORS: &[&str] = &["#f0f4f8", "#e2e8f0", "#cbd5e1", "#94a3b8"];

/// Stroke colors (pre-computed ~30% darker than bg)
const STROKE_COLORS: &[&str] = &["#b0bec5", "#90a4ae", "#78909c", "#607d8b"];

/// Label text colors (pre-computed ~60% darker than bg)
const LABEL_COLORS: &[&str] = &["#546e7a", "#455a64", "#37474f", "#263238"];

fn get_bg_color(depth: usize) -> &'static str {
    BG_COLORS[depth.min(BG_COLORS.len() - 1)]
}

fn get_stroke_color(depth: usize) -> &'static str {
    STROKE_COLORS[depth.min(STROKE_COLORS.len() - 1)]
}

fn get_label_color(depth: usize) -> &'static str {
    LABEL_COLORS[depth.min(LABEL_COLORS.len() - 1)]
}

/// Render all subgraph backgrounds as SVG elements.
///
/// Returns an SVG string with `<rect>` and `<text>` elements for each subgraph frame.
/// Parent frames are rendered before children to ensure correct z-order (parent behind child).
pub fn render_subgraph_backgrounds(
    tree: &SubgraphTree,
    boxes: &HashMap<String, BoundingBox>,
) -> String {
    let mut svg = String::new();

    if let Some(root) = tree.nodes.get(&tree.root_id) {
        for child_id in &root.children {
            render_subgraph_recursive(tree, child_id, boxes, &mut svg, 0);
        }
    }

    svg
}

fn render_subgraph_recursive(
    tree: &SubgraphTree,
    subtree_id: &str,
    boxes: &HashMap<String, BoundingBox>,
    svg: &mut String,
    depth: usize,
) {
    let bbox = match boxes.get(subtree_id) {
        Some(b) => b,
        None => return,
    };

    let bg = get_bg_color(depth);
    let stroke = get_stroke_color(depth);
    let label_color = get_label_color(depth);

    // Background rectangle with dashed border
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         rx=\"{:.0}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{:.1}\" \
         stroke-dasharray=\"{}\"/>\n",
        bbox.x,
        bbox.y,
        bbox.width,
        bbox.height,
        BORDER_RADIUS,
        bg,
        stroke,
        STROKE_WIDTH,
        STROKE_DASHARRAY,
    ));

    // Label text (top-left corner)
    if let Some(tree_node) = tree.nodes.get(subtree_id) {
        if let Some(label) = &tree_node.label {
            let escaped = escape_xml(label);
            svg.push_str(&format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" \
                 font-family=\"Arial, Helvetica, sans-serif\" font-size=\"{:.0}\" \
                 font-weight=\"bold\" fill=\"{}\">{}</text>\n",
                bbox.x + LABEL_OFFSET_X,
                bbox.y + LABEL_OFFSET_Y,
                LABEL_FONT_SIZE,
                label_color,
                escaped,
            ));
        }
    }

    // Recurse into children at depth+1 (rendered on top of parent)
    if let Some(tree_node) = tree.nodes.get(subtree_id) {
        for child_id in tree_node.children.clone() {
            render_subgraph_recursive(tree, &child_id, boxes, svg, depth + 1);
        }
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SubgraphTreeNode;

    fn make_tree_with_one_sg() -> (SubgraphTree, HashMap<String, BoundingBox>) {
        let mut nodes = HashMap::new();
        nodes.insert(
            "ROOT".to_string(),
            SubgraphTreeNode {
                id: "ROOT".to_string(),
                label: None,
                children: vec!["S1".to_string()],
                direct_node_ids: vec![],
            },
        );
        nodes.insert(
            "S1".to_string(),
            SubgraphTreeNode {
                id: "S1".to_string(),
                label: Some("My Subgraph".to_string()),
                children: vec![],
                direct_node_ids: vec!["A".to_string(), "B".to_string()],
            },
        );

        let tree = SubgraphTree {
            nodes,
            root_id: "ROOT".to_string(),
        };

        let mut boxes = HashMap::new();
        boxes.insert(
            "S1".to_string(),
            BoundingBox {
                x: 10.0,
                y: 20.0,
                width: 200.0,
                height: 150.0,
            },
        );

        (tree, boxes)
    }

    #[test]
    fn test_render_single_frame() {
        let (tree, boxes) = make_tree_with_one_sg();
        let svg = render_subgraph_backgrounds(&tree, &boxes);

        assert!(svg.contains("<rect"), "Should contain a rect element");
        assert!(
            svg.contains("stroke-dasharray=\"5,3\""),
            "Should have dashed stroke"
        );
        assert!(svg.contains("My Subgraph"), "Should contain the label");
        assert!(svg.contains("font-weight=\"bold\""), "Label should be bold");
    }

    #[test]
    fn test_depth_colors() {
        assert_eq!(get_bg_color(0), "#f0f4f8");
        assert_eq!(get_bg_color(1), "#e2e8f0");
        assert_eq!(get_bg_color(2), "#cbd5e1");
        assert_eq!(get_bg_color(3), "#94a3b8");
        assert_eq!(get_bg_color(99), "#94a3b8"); // clamped
    }

    #[test]
    fn test_z_order_parent_before_child() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "ROOT".to_string(),
            SubgraphTreeNode {
                id: "ROOT".to_string(),
                label: None,
                children: vec!["S1".to_string()],
                direct_node_ids: vec![],
            },
        );
        nodes.insert(
            "S1".to_string(),
            SubgraphTreeNode {
                id: "S1".to_string(),
                label: Some("Parent".to_string()),
                children: vec!["S2".to_string()],
                direct_node_ids: vec![],
            },
        );
        nodes.insert(
            "S2".to_string(),
            SubgraphTreeNode {
                id: "S2".to_string(),
                label: Some("Child".to_string()),
                children: vec![],
                direct_node_ids: vec![],
            },
        );

        let tree = SubgraphTree {
            nodes,
            root_id: "ROOT".to_string(),
        };

        let mut boxes = HashMap::new();
        boxes.insert(
            "S1".to_string(),
            BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 250.0,
            },
        );
        boxes.insert(
            "S2".to_string(),
            BoundingBox {
                x: 40.0,
                y: 65.0,
                width: 220.0,
                height: 145.0,
            },
        );

        let svg = render_subgraph_backgrounds(&tree, &boxes);

        // Parent rect should appear before child rect
        let parent_pos = svg.find("Parent").expect("Should contain Parent label");
        let child_pos = svg.find("Child").expect("Should contain Child label");
        assert!(
            parent_pos < child_pos,
            "Parent should be rendered before child for correct z-order"
        );

        // Parent should use depth-0 color, child should use depth-1 color
        assert!(
            svg.contains("#f0f4f8"),
            "Parent should have depth-0 bg color"
        );
        assert!(
            svg.contains("#e2e8f0"),
            "Child should have depth-1 bg color"
        );
    }
}
