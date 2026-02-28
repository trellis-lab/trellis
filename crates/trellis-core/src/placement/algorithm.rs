//! Core placement algorithm trait.
//!
//! [`LayoutAlgorithm`] is the single interface that every self-contained
//! placement algorithm must implement.  Diagram-specific orchestrators
//! (`er.rs`, `class.rs`, `c4.rs`) are **not** covered by this trait — they
//! remain plain functions that prepare diagram-specific data before delegating
//! to a concrete [`LayoutAlgorithm`].

use trellis_parser::Graph;

/// A self-contained placement algorithm that assigns `(x, y)` coordinates
/// to every node in `graph`.
///
/// After `layout` returns, every `graph.nodes[i].x` and `.y` must be set to
/// a valid finite value.  The algorithm may assume that node dimensions
/// (`width`, `height`) have already been set by the caller.
pub trait LayoutAlgorithm {
    fn layout(&self, graph: &mut Graph);
}
