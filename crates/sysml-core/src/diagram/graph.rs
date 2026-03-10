/// Format-agnostic diagram intermediate representation.

/// The kind of diagram being generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramKind {
    /// Block Definition Diagram — definitions and their relationships.
    Bdd,
    /// Internal Block Diagram — internal structure of a single definition.
    Ibd,
    /// State Machine Diagram — states and transitions.
    Stm,
    /// Activity Diagram — action flow.
    Act,
    /// Requirements Diagram — requirements and trace relationships.
    Req,
    /// Package Diagram — packages and containment.
    Pkg,
    /// Parametric Diagram — constraints and parameters.
    Par,
    /// Traceability Diagram — V-model: requirements → design → verification.
    Trace,
    /// Allocation Diagram — logical-to-physical mapping (actions/use-cases → parts).
    Alloc,
    /// Use Case Diagram — use case definitions and actors.
    Ucd,
}

impl DiagramKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Bdd => "Block Definition Diagram",
            Self::Ibd => "Internal Block Diagram",
            Self::Stm => "State Machine Diagram",
            Self::Act => "Activity Diagram",
            Self::Req => "Requirements Diagram",
            Self::Pkg => "Package Diagram",
            Self::Par => "Parametric Diagram",
            Self::Trace => "Traceability Diagram",
            Self::Alloc => "Allocation Diagram",
            Self::Ucd => "Use Case Diagram",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bdd" => Some(Self::Bdd),
            "ibd" => Some(Self::Ibd),
            "stm" | "state" => Some(Self::Stm),
            "act" | "activity" => Some(Self::Act),
            "req" | "requirements" => Some(Self::Req),
            "pkg" | "package" => Some(Self::Pkg),
            "par" | "parametric" => Some(Self::Par),
            "trace" | "traceability" => Some(Self::Trace),
            "alloc" | "allocation" => Some(Self::Alloc),
            "ucd" | "usecase" | "use-case" => Some(Self::Ucd),
            _ => None,
        }
    }
}

/// Layout direction for the diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    TopBottom,
    LeftRight,
    BottomTop,
    RightLeft,
}

impl Default for LayoutDirection {
    fn default() -> Self {
        Self::TopBottom
    }
}

impl LayoutDirection {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TB" | "TD" | "TOP-BOTTOM" | "TOP-DOWN" => Some(Self::TopBottom),
            "LR" | "LEFT-RIGHT" => Some(Self::LeftRight),
            "BT" | "BOTTOM-TOP" | "BOTTOM-UP" => Some(Self::BottomTop),
            "RL" | "RIGHT-LEFT" => Some(Self::RightLeft),
            _ => None,
        }
    }

    pub fn mermaid_code(&self) -> &'static str {
        match self {
            Self::TopBottom => "TD",
            Self::LeftRight => "LR",
            Self::BottomTop => "BT",
            Self::RightLeft => "RL",
        }
    }

    pub fn dot_code(&self) -> &'static str {
        match self {
            Self::TopBottom => "TB",
            Self::LeftRight => "LR",
            Self::BottomTop => "BT",
            Self::RightLeft => "RL",
        }
    }

    pub fn d2_code(&self) -> &'static str {
        match self {
            Self::TopBottom => "down",
            Self::LeftRight => "right",
            Self::BottomTop => "up",
            Self::RightLeft => "left",
        }
    }
}

/// A node in the diagram graph.
#[derive(Debug, Clone)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub stereotype: Option<String>,
    pub attributes: Vec<(String, String)>,
}

/// Classification of diagram nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Block,
    Port,
    State,
    Action,
    Decision,
    Fork,
    Join,
    Requirement,
    Constraint,
    Package,
    InitialState,
    FinalState,
    Note,
    UseCase,
    Actor,
}

/// An edge in the diagram graph.
#[derive(Debug, Clone)]
pub struct DiagramEdge {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub kind: EdgeKind,
}

/// Classification of diagram edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Composition,
    Specialization,
    Connection,
    Flow,
    Transition,
    Satisfy,
    Verify,
    Allocate,
    Dependency,
    Containment,
    ConstraintBinding,
}

/// A subgraph (containment group).
#[derive(Debug, Clone)]
pub struct Subgraph {
    pub id: String,
    pub label: String,
    pub node_ids: Vec<String>,
}

/// The complete diagram graph.
#[derive(Debug, Clone)]
pub struct DiagramGraph {
    pub title: String,
    pub kind: DiagramKind,
    pub nodes: Vec<DiagramNode>,
    pub edges: Vec<DiagramEdge>,
    pub subgraphs: Vec<Subgraph>,
    pub direction: LayoutDirection,
    pub max_depth: Option<usize>,
}

impl DiagramGraph {
    pub fn new(title: String, kind: DiagramKind) -> Self {
        Self {
            title,
            kind,
            nodes: Vec::new(),
            edges: Vec::new(),
            subgraphs: Vec::new(),
            direction: LayoutDirection::default(),
            max_depth: None,
        }
    }

    pub fn add_node(&mut self, node: DiagramNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: DiagramEdge) {
        self.edges.push(edge);
    }

    pub fn add_subgraph(&mut self, subgraph: Subgraph) {
        self.subgraphs.push(subgraph);
    }

    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    /// Remove nodes and edges that don't match the allowed set of names.
    /// Pseudo-nodes (initial/final states) and structural nodes (forks/joins/decisions)
    /// are always kept. Edges are kept only if both endpoints remain.
    pub fn filter_by_names(&mut self, allowed: &std::collections::HashSet<&str>) {
        self.nodes.retain(|n| {
            matches!(
                n.kind,
                NodeKind::InitialState
                    | NodeKind::FinalState
                    | NodeKind::Fork
                    | NodeKind::Join
                    | NodeKind::Decision
            ) || allowed.contains(n.id.as_str())
        });
        let remaining: std::collections::HashSet<&str> =
            self.nodes.iter().map(|n| n.id.as_str()).collect();
        self.edges
            .retain(|e| remaining.contains(e.source.as_str()) || remaining.contains(e.target.as_str()));
        self.subgraphs.iter_mut().for_each(|sg| {
            sg.node_ids.retain(|id| remaining.contains(id.as_str()));
        });
        self.subgraphs.retain(|sg| !sg.node_ids.is_empty());
    }
}
