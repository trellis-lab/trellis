//! Generic overlap detection and spiral-search placement helper.
//!
//! All coordinates in this module are **centre-based** (cx, cy are the centre
//! of a node, not the top-left corner).  Callers are responsible for
//! converting to/from top-left coordinates as needed.

use std::collections::HashMap;

use super::NODE_SPACING;

/// Return `true` if a node centred at (`cx`, `cy`) with dimensions (`w`, `h`)
/// overlaps any node already recorded in `coords` / `sizes`.
///
/// A small gap of 10 px is added around each node to avoid tight packing.
pub fn overlaps_any(
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    coords: &HashMap<String, (f64, f64)>,
    sizes: &HashMap<String, (f64, f64)>,
) -> bool {
    let gap = 10.0_f64;
    for (id, (ox, oy)) in coords {
        let (ow, oh) = sizes[id];
        let dx = (cx - ox).abs();
        let dy = (cy - oy).abs();
        if dx < (w + ow) / 2.0 + gap && dy < (h + oh) / 2.0 + gap {
            return true;
        }
    }
    false
}

/// Find a non-overlapping position near (`cx`, `cy`) using a ring-based spiral
/// search.
///
/// If the candidate position is already free it is returned immediately.
/// Otherwise the search expands outward in rings of increasing radius (up to
/// radius 40) and returns the first free position found.  As a last resort the
/// node is placed far to the right of all existing nodes.
pub fn find_free_position(
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    coords: &HashMap<String, (f64, f64)>,
    sizes: &HashMap<String, (f64, f64)>,
) -> (f64, f64) {
    if !overlaps_any(cx, cy, w, h, coords, sizes) {
        return (cx, cy);
    }

    let step = (w.max(h) / 2.0 + NODE_SPACING).max(NODE_SPACING);
    for radius in 1i32..=40 {
        let r = radius as f64 * step;
        let candidates = [
            (cx + r, cy),
            (cx - r, cy),
            (cx, cy + r),
            (cx, cy - r),
            (cx + r, cy + r),
            (cx - r, cy - r),
            (cx + r, cy - r),
            (cx - r, cy + r),
        ];
        for (tx, ty) in candidates {
            if !overlaps_any(tx, ty, w, h, coords, sizes) {
                return (tx, ty);
            }
        }
    }
    // Fallback: place far to the right
    let max_x = coords.values().map(|(x, _)| *x).fold(cx, f64::max);
    (max_x + w + NODE_SPACING, cy)
}
