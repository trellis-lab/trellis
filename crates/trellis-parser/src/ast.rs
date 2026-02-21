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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    // Class diagram specific fields
    pub stereotype: Option<String>,
    pub class_attributes: Vec<ClassAttribute>,
    pub class_methods: Vec<ClassMethod>,
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
    pub arrow_head: ArrowHead,
    // Class diagram specific fields
    pub class_edge_type: Option<ClassEdgeType>,
    pub source_multiplicity: Option<String>,
    pub target_multiplicity: Option<String>,
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
    Cylinder,
    Stadium,
    Subroutine,
    Asymmetric,
    Parallelogram,
    ParallelogramAlt,
    Trapezoid,
    TrapezoidAlt,
    DoubleCircle,
    /// Class diagram box (three-compartment)
    ClassBox,
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

// ── Class diagram types ──────────────────────────────────────────────

/// Visibility of a class member
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClassVisibility {
    #[default]
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
}

/// An attribute (field) of a class
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassAttribute {
    pub visibility: ClassVisibility,
    pub attr_type: String,
    pub name: String,
    pub is_static: bool,
    pub is_abstract: bool,
}

/// A method of a class
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassMethod {
    pub visibility: ClassVisibility,
    pub return_type: String,
    pub name: String,
    pub params: String,
    pub is_static: bool,
    pub is_abstract: bool,
}

/// Type of class diagram relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClassEdgeType {
    Inheritance,  // <|-- or --|>
    Composition,  // *-- or --*
    Aggregation,  // o-- or --o
    #[default]
    Association,  // --> or <-- or --
    Realization,  // <|.. or ..|>
    Dependency,   // ..> or <..
    Link,         // ..
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }
}
