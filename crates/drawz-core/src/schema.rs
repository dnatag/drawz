//! Diagram schema types — the JSON-to-Rust mapping.
//!
//! Each struct maps 1:1 to a JSON input shape. The `Diagram` enum is
//! discriminated by the `"type"` field via serde's tagged enum.
//! Fields are self-documenting via their names and map directly to
//! the JSON input format documented in the README.

#![allow(missing_docs)]

use serde::Deserialize;

/// Top-level input wrapper. Extracts `width` before dispatching to diagram type.
#[derive(Debug, Deserialize)]
pub struct DiagramInput {
    /// Maximum output width in characters. Default: 120.
    #[serde(default = "default_width")]
    pub width: u16,
    #[serde(flatten)]
    pub diagram: Diagram,
}

fn default_width() -> u16 {
    120
}

/// The diagram type, discriminated by the `type` field.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Diagram {
    Flow(FlowDiagram),
    State(StateDiagram),
    Tree(TreeDiagram),
    Sequence(SequenceDiagram),
    Table(TableDiagram),
    Dag(DagDiagram),
    Component(ComponentDiagram),
    Freeform(FreeformDiagram),
    Mermaid(MermaidDiagram),
}

/// Linear: `{ "steps": ["A", "B", "C"] }`
/// Nested: `{ "steps": ["A", {"label": "B", "steps": ["X", "Y"]}] }`
/// Full: `{ "nodes": [...], "edges": [...] }`
#[derive(Debug, Clone, Deserialize)]
pub struct FlowDiagram {
    pub title: Option<String>,
    /// Direction: "LR" for horizontal, "TD"/"TB" for vertical (default)
    pub direction: Option<String>,
    pub steps: Option<Vec<FlowStep>>,
    #[serde(default, deserialize_with = "deserialize_nodes")]
    pub nodes: Option<Vec<Node>>,
    pub edges: Option<Vec<Edge>>,
}

/// A step: plain label or nested sub-flow.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FlowStep {
    Sub(SubFlow),
    Label(String),
}

/// A named sub-pipeline within a flow.
#[derive(Debug, Clone, Deserialize)]
pub struct SubFlow {
    pub label: String,
    pub steps: Vec<FlowStep>,
}

/// Minimal: `{ "transitions": [{"from":"A","to":"B","label":"x"}] }`
/// States inferred from transitions if not provided.
#[derive(Debug, Clone, Deserialize)]
pub struct StateDiagram {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nodes")]
    pub states: Option<Vec<Node>>,
    pub transitions: Vec<Edge>,
}

/// Minimal: `{ "indent": "root\n  child1\n  child2" }`
/// Full: `{ "root": { "label": "root", "children": [...] } }`
#[derive(Debug, Clone, Deserialize)]
pub struct TreeDiagram {
    pub title: Option<String>,
    pub root: Option<TreeNode>,
    pub indent: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeNode {
    pub label: String,
    #[serde(default)]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SequenceDiagram {
    pub title: Option<String>,
    pub actors: Vec<String>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableDiagram {
    pub title: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Minimal: `{ "edges": [{"from":"A","to":"B"}] }` — nodes inferred.
/// Full: `{ "nodes": [...], "edges": [...] }`
#[derive(Debug, Clone, Deserialize)]
pub struct DagDiagram {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nodes")]
    pub nodes: Option<Vec<Node>>,
    pub edges: Vec<Edge>,
    /// Named clusters grouping nodes visually.
    pub subgraphs: Option<Vec<Subgraph>>,
}

/// A named group of nodes rendered as a framed cluster.
#[derive(Debug, Clone, Deserialize)]
pub struct Subgraph {
    pub label: String,
    pub node_ids: Vec<String>,
}

/// Architecture diagram: groups of nodes with labeled connections between them.
/// `{ "type": "component", "groups": [...], "connections": [...] }`
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentDiagram {
    pub title: Option<String>,
    pub groups: Vec<ComponentGroup>,
    pub connections: Vec<Connection>,
}

/// A named subsystem containing nodes.
/// Nodes can be a flat list, or organized into chains (horizontal pipelines).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ComponentGroup {
    pub label: String,
    /// Flat node list (rendered vertically). Used when no chains provided.
    #[serde(default)]
    pub nodes: Vec<String>,
    /// Horizontal pipelines: each chain renders as A → B → C on one row.
    #[serde(default)]
    pub chains: Vec<Vec<String>>,
    /// Internal connections between nodes (rendered as vertical/L-shaped arrows between chain rows).
    #[serde(default)]
    pub edges: Vec<Connection>,
}

/// A labeled connection between two nodes (which may be in different groups).
#[derive(Debug, Clone, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// Freeform text block.
/// `{ "content": "line1\nline2" }` or `{ "lines": ["a","b"] }`
#[derive(Debug, Clone, Deserialize)]
pub struct FreeformDiagram {
    pub title: Option<String>,
    pub content: Option<String>,
    pub lines: Option<Vec<String>>,
}

/// Mermaid DSL input — agents already know this format.
/// `{ "type": "mermaid", "code": "graph LR; A-->B-->C" }`
#[derive(Debug, Clone, Deserialize)]
pub struct MermaidDiagram {
    pub title: Option<String>,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NodeInput {
    Object { id: Option<String>, label: String },
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Option<String>,
    pub label: String,
}

impl From<NodeInput> for Node {
    fn from(input: NodeInput) -> Self {
        match input {
            NodeInput::Object { id, label } => Node { id, label },
            NodeInput::Label(s) => Node { id: None, label: s },
        }
    }
}

fn deserialize_nodes<'de, D>(deserializer: D) -> Result<Option<Vec<Node>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<Vec<NodeInput>> = Option::deserialize(deserializer)?;
    Ok(opt.map(|v| v.into_iter().map(Node::from).collect()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// Strip embedded newlines/carriage-returns/tabs from every label-like
/// field, collapsing runs of whitespace left behind.
///
/// Renderers build box-drawing frames by measuring a label's display width
/// and interpolating it directly into a single output line
/// (`format!("│ {label} │")`). A label containing a literal `\n` splits
/// that line in two when printed, breaking the frame open and violating the
/// "every line has the same display width" alignment guarantee — with no
/// error or warning, since nothing about the string is otherwise invalid.
///
/// Call this on every `Diagram` built from untrusted (agent-supplied) JSON,
/// before rendering. Deliberately does NOT touch `freeform.content`/`lines`,
/// `tree.indent`, or `mermaid.code` — those fields are multi-line by design.
pub fn sanitize(diagram: &mut Diagram) {
    fn sanitize_str(s: &mut String) {
        if s.contains(['\n', '\r', '\t']) {
            *s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        }
    }
    fn sanitize_opt(s: &mut Option<String>) {
        if let Some(inner) = s {
            sanitize_str(inner);
        }
    }
    fn sanitize_vec(v: &mut [String]) {
        v.iter_mut().for_each(sanitize_str);
    }
    fn sanitize_node(n: &mut Node) {
        sanitize_str(&mut n.label);
        sanitize_opt(&mut n.id);
    }
    fn sanitize_edge(e: &mut Edge) {
        sanitize_str(&mut e.from);
        sanitize_str(&mut e.to);
        sanitize_opt(&mut e.label);
    }
    fn sanitize_connection(c: &mut Connection) {
        sanitize_str(&mut c.from);
        sanitize_str(&mut c.to);
        sanitize_opt(&mut c.label);
    }
    fn sanitize_tree_node(n: &mut TreeNode) {
        sanitize_str(&mut n.label);
        n.children.iter_mut().for_each(sanitize_tree_node);
    }
    fn sanitize_steps(steps: &mut [FlowStep]) {
        for s in steps {
            match s {
                FlowStep::Label(l) => sanitize_str(l),
                FlowStep::Sub(sub) => {
                    sanitize_str(&mut sub.label);
                    sanitize_steps(&mut sub.steps);
                }
            }
        }
    }

    match diagram {
        Diagram::Flow(d) => {
            sanitize_opt(&mut d.title);
            if let Some(steps) = &mut d.steps {
                sanitize_steps(steps);
            }
            if let Some(nodes) = &mut d.nodes {
                nodes.iter_mut().for_each(sanitize_node);
            }
            if let Some(edges) = &mut d.edges {
                edges.iter_mut().for_each(sanitize_edge);
            }
        }
        Diagram::State(d) => {
            sanitize_opt(&mut d.title);
            if let Some(states) = &mut d.states {
                states.iter_mut().for_each(sanitize_node);
            }
            d.transitions.iter_mut().for_each(sanitize_edge);
        }
        Diagram::Tree(d) => {
            sanitize_opt(&mut d.title);
            if let Some(root) = &mut d.root {
                sanitize_tree_node(root);
            }
            // d.indent intentionally untouched — multi-line by design.
        }
        Diagram::Sequence(d) => {
            sanitize_opt(&mut d.title);
            sanitize_vec(&mut d.actors);
            for m in &mut d.messages {
                sanitize_str(&mut m.from);
                sanitize_str(&mut m.to);
                sanitize_str(&mut m.label);
            }
        }
        Diagram::Table(d) => {
            sanitize_opt(&mut d.title);
            sanitize_vec(&mut d.headers);
            d.rows.iter_mut().for_each(|row| sanitize_vec(row));
        }
        Diagram::Dag(d) => {
            sanitize_opt(&mut d.title);
            if let Some(nodes) = &mut d.nodes {
                nodes.iter_mut().for_each(sanitize_node);
            }
            d.edges.iter_mut().for_each(sanitize_edge);
            if let Some(subgraphs) = &mut d.subgraphs {
                for sg in subgraphs {
                    sanitize_str(&mut sg.label);
                    sanitize_vec(&mut sg.node_ids);
                }
            }
        }
        Diagram::Component(d) => {
            sanitize_opt(&mut d.title);
            for g in &mut d.groups {
                sanitize_str(&mut g.label);
                sanitize_vec(&mut g.nodes);
                g.chains.iter_mut().for_each(|chain| sanitize_vec(chain));
                g.edges.iter_mut().for_each(sanitize_connection);
            }
            d.connections.iter_mut().for_each(sanitize_connection);
        }
        Diagram::Freeform(d) => {
            sanitize_opt(&mut d.title);
            // content/lines intentionally untouched — multi-line by design.
        }
        Diagram::Mermaid(d) => {
            sanitize_opt(&mut d.title);
            // code intentionally untouched — multi-line DSL.
        }
    }
}
