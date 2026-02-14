use serde::{Deserialize, Serialize};

/// The main graph structure representing a parsed Mermaid diagram
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub subgraphs: Vec<Subgraph>,
    pub direction: Direction,
    pub diagram_type: DiagramType,
}

/// A node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
    pub arrow_head: ArrowHead,
}

/// A subgraph containing nodes and edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subgraph {
    pub id: String,
    pub label: Option<String>,
    pub nodes: Vec<String>,
    pub subgraphs: Vec<Subgraph>,
}

/// Direction of the graph layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Direction {
    #[default]
    TB, // Top to Bottom
    BT, // Bottom to Top
    LR, // Left to Right
    RL, // Right to Left
}

/// Type of diagram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiagramType {
    #[default]
    Flowchart,
    ClassDiagram,
    ErDiagram,
}

/// Shape of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NodeShape {
    #[default]
    Rectangle,
    RoundedRectangle,
    Diamond,
    Circle,
    Hexagon,
}

/// Style of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeStyle {
    #[default]
    Solid,
    Dotted,
    Thick,
}

/// Arrow head type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArrowHead {
    #[default]
    Arrow,
    None,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }
}
