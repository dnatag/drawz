use serde::Deserialize;

/// Top-level input wrapper. Extracts `width` before dispatching to diagram type.
#[derive(Debug, Deserialize)]
pub struct DiagramInput {
    /// Maximum output width in characters. Default: 80.
    #[serde(default = "default_width")]
    pub width: u16,
    #[serde(flatten)]
    pub diagram: Diagram,
}

fn default_width() -> u16 { 80 }

/// The diagram type, discriminated by the `type` field.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Diagram {
    Flow(FlowDiagram),
    State(StateDiagram),
    Tree(TreeDiagram),
    Sequence(SequenceDiagram),
    Table(TableDiagram),
    Dag(DagDiagram),
    Freeform(FreeformDiagram),
    Mermaid(MermaidDiagram),
}

/// Linear: `{ "steps": ["A", "B", "C"] }`
/// Nested: `{ "steps": ["A", {"label": "B", "steps": ["X", "Y"]}] }`
/// Full: `{ "nodes": [...], "edges": [...] }`
#[derive(Debug, Deserialize)]
pub struct FlowDiagram {
    pub title: Option<String>,
    pub steps: Option<Vec<FlowStep>>,
    pub nodes: Option<Vec<Node>>,
    pub edges: Option<Vec<Edge>>,
}

/// A step: plain label or nested sub-flow.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FlowStep {
    Sub(SubFlow),
    Label(String),
}

/// A named sub-pipeline within a flow.
#[derive(Debug, Deserialize)]
pub struct SubFlow {
    pub label: String,
    pub steps: Vec<FlowStep>,
}

/// Minimal: `{ "transitions": [{"from":"A","to":"B","label":"x"}] }`
/// States inferred from transitions if not provided.
#[derive(Debug, Deserialize)]
pub struct StateDiagram {
    pub title: Option<String>,
    pub states: Option<Vec<Node>>,
    pub transitions: Vec<Edge>,
}

/// Minimal: `{ "indent": "root\n  child1\n  child2" }`
/// Full: `{ "root": { "label": "root", "children": [...] } }`
#[derive(Debug, Deserialize)]
pub struct TreeDiagram {
    pub title: Option<String>,
    pub root: Option<TreeNode>,
    pub indent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TreeNode {
    pub label: String,
    #[serde(default)]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Deserialize)]
pub struct SequenceDiagram {
    pub title: Option<String>,
    pub actors: Vec<String>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct TableDiagram {
    pub title: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Minimal: `{ "edges": [{"from":"A","to":"B"}] }` — nodes inferred.
/// Full: `{ "nodes": [...], "edges": [...] }`
#[derive(Debug, Deserialize)]
pub struct DagDiagram {
    pub title: Option<String>,
    pub nodes: Option<Vec<Node>>,
    pub edges: Vec<Edge>,
}

/// Freeform text block.
/// `{ "content": "line1\nline2" }` or `{ "lines": ["a","b"] }`
#[derive(Debug, Deserialize)]
pub struct FreeformDiagram {
    pub title: Option<String>,
    pub content: Option<String>,
    pub lines: Option<Vec<String>>,
}

/// Mermaid DSL input — agents already know this format.
/// `{ "type": "mermaid", "code": "graph LR; A-->B-->C" }`
#[derive(Debug, Deserialize)]
pub struct MermaidDiagram {
    pub title: Option<String>,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: Option<String>,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}
