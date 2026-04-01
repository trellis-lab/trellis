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
    // ER diagram specific fields
    pub er_attributes: Vec<ErAttribute>,
    // C4 diagram specific fields
    pub c4_type: Option<C4NodeType>,
    pub c4_description: Option<String>,
    pub c4_technology: Option<String>,
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
    // ER diagram specific fields
    pub er_source_card: Option<ErCardinality>,
    pub er_target_card: Option<ErCardinality>,
    pub er_identifying: Option<bool>,
    // C4 diagram specific fields
    pub c4_rel_type: Option<C4RelType>,
    pub c4_technology: Option<String>,
    pub c4_bidirectional: bool,
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
    /// C4 architecture diagrams (Context, Container, Component, Dynamic, Deployment)
    C4Diagram,
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
    /// ER diagram entity box
    ErBox,
    /// C4 diagram element box
    C4Box,
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
    Public, // +
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
    Inheritance, // <|-- or --|>
    Composition, // *-- or --*
    Aggregation, // o-- or --o
    #[default]
    Association, // --> or <-- or --
    Realization, // <|.. or ..|>
    Dependency,  // ..> or <..
    Link,        // ..
}

// ── ER diagram types ──────────────────────────────────────────────────

/// Key type for ER diagram attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    PK, // Primary Key
    FK, // Foreign Key
    UK, // Unique Key
}

/// An attribute in an ER entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErAttribute {
    pub attr_type: String,
    pub name: String,
    pub keys: Vec<KeyType>,
    pub comment: Option<String>,
}

/// Cardinality notation for ER diagram relationships (crow's foot)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErCardinality {
    ExactlyOne, // ||
    ZeroOrOne,  // |o or o|
    OneOrMore,  // }| or |{
    ZeroOrMore, // }o or o{
}

// ── C4 diagram types ──────────────────────────────────────────────────

/// C4 element type — determines rendering shape and boundary semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum C4NodeType {
    // C4Context / common
    Person,
    PersonExt,
    System,
    SystemDb,
    SystemQueue,
    SystemExt,
    SystemDbExt,
    SystemQueueExt,
    // C4Container
    Container,
    ContainerDb,
    ContainerQueue,
    ContainerExt,
    ContainerDbExt,
    ContainerQueueExt,
    // C4Component
    Component,
    ComponentDb,
    ComponentQueue,
    ComponentExt,
    ComponentDbExt,
    ComponentQueueExt,
    // Boundaries (rendered as group frames, not boxes)
    EnterpriseBoundary,
    SystemBoundary,
    ContainerBoundary,
    // C4Deployment
    DeploymentNode,
}

impl C4NodeType {
    /// Returns true if this element is external (dashed border).
    pub fn is_external(self) -> bool {
        matches!(
            self,
            C4NodeType::PersonExt
                | C4NodeType::SystemExt
                | C4NodeType::SystemDbExt
                | C4NodeType::SystemQueueExt
                | C4NodeType::ContainerExt
                | C4NodeType::ContainerDbExt
                | C4NodeType::ContainerQueueExt
                | C4NodeType::ComponentExt
                | C4NodeType::ComponentDbExt
                | C4NodeType::ComponentQueueExt
        )
    }

    /// Returns true if this element is a boundary or deployment node — rendered as a
    /// frame enclosing its contents rather than as an individual box.
    pub fn is_boundary(self) -> bool {
        matches!(
            self,
            C4NodeType::EnterpriseBoundary
                | C4NodeType::SystemBoundary
                | C4NodeType::ContainerBoundary
                | C4NodeType::DeploymentNode
        )
    }

    /// Returns true if this element uses a database cylinder shape.
    pub fn is_db(self) -> bool {
        matches!(
            self,
            C4NodeType::SystemDb
                | C4NodeType::SystemDbExt
                | C4NodeType::ContainerDb
                | C4NodeType::ContainerDbExt
                | C4NodeType::ComponentDb
                | C4NodeType::ComponentDbExt
        )
    }

    /// Returns true if this element uses a queue (double-frame) shape.
    pub fn is_queue(self) -> bool {
        matches!(
            self,
            C4NodeType::SystemQueue
                | C4NodeType::SystemQueueExt
                | C4NodeType::ContainerQueue
                | C4NodeType::ContainerQueueExt
                | C4NodeType::ComponentQueue
                | C4NodeType::ComponentQueueExt
        )
    }

    /// Returns true if this element uses the person (human figure) shape.
    pub fn is_person(self) -> bool {
        matches!(self, C4NodeType::Person | C4NodeType::PersonExt)
    }
}

/// C4 relationship type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum C4RelType {
    #[default]
    Rel,
    RelBack,
    RelU,
    RelD,
    RelL,
    RelR,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }
}
