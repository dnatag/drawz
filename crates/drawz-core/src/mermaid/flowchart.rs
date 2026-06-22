use crate::schema::{DagDiagram, Diagram, Edge, FlowDiagram, Node, Subgraph};

use super::helpers::{skip_first_line, split_statements};

/// Arrow patterns in priority order (longer/more specific first).
const ARROW_PATTERNS: &[&str] = &["-->|", "==>|", "-.->", "-->", "---", "==>"];

/// Bracket pairs for node label extraction.
const BRACKET_PAIRS: &[(u8, u8)] = &[(b'[', b']'), (b'(', b')'), (b'{', b'}')];

/// Parse `graph LR; A-->B-->C` or multiline flowchart.
pub(super) fn parse_flowchart(code: &str) -> Result<Diagram, String> {
    let first_part = code.split(['\n', ';']).next().unwrap_or("").trim();
    let direction = first_part.split_whitespace().nth(1).map(String::from);

    let body = skip_first_line(code);

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut subgraphs: Vec<Subgraph> = Vec::new();
    let mut current_subgraph: Option<usize> = None;

    for segment in split_statements(body) {
        if let Some(label) = segment.strip_prefix("subgraph") {
            let sg_idx = subgraphs.len();
            subgraphs.push(Subgraph {
                label: label.trim().to_string(),
                node_ids: Vec::new(),
            });
            current_subgraph = Some(sg_idx);
            continue;
        }
        if segment == "end" {
            current_subgraph = None;
            continue;
        }

        let nodes_before = nodes.len();
        parse_flow_statement(segment, &mut nodes, &mut edges);

        // Assign newly registered nodes to the active subgraph
        if let Some(sg_idx) = current_subgraph {
            for node in &nodes[nodes_before..] {
                if let Some(id) = &node.id {
                    subgraphs[sg_idx].node_ids.push(id.clone());
                }
            }
        }
    }

    if nodes.is_empty() && edges.is_empty() {
        return Err("flowchart has no nodes or edges".to_string());
    }

    let has_branching = has_branching_or_merging(&edges);

    if has_branching || !subgraphs.is_empty() {
        Ok(Diagram::Dag(DagDiagram {
            title: None,
            nodes: Some(nodes),
            edges,
            subgraphs: (!subgraphs.is_empty()).then_some(subgraphs),
        }))
    } else {
        Ok(Diagram::Flow(FlowDiagram {
            title: None,
            direction,
            steps: None,
            nodes: Some(nodes),
            edges: Some(edges),
        }))
    }
}

/// Check if any node has multiple outgoing or incoming edges.
fn has_branching_or_merging(edges: &[Edge]) -> bool {
    // O(n) with HashSets instead of O(n²) nested iteration
    use std::collections::HashSet;
    let mut seen_from: HashSet<&str> = HashSet::new();
    let mut seen_to: HashSet<&str> = HashSet::new();
    for e in edges {
        if !seen_from.insert(&e.from) {
            return true; // duplicate source = fan-out
        }
        if !seen_to.insert(&e.to) {
            return true; // duplicate target = fan-in
        }
    }
    false
}

/// Parse a single statement, extracting nodes and edges.
/// Handles chained arrows recursively (e.g., `A-->B-->C`).
fn parse_flow_statement(stmt: &str, nodes: &mut Vec<Node>, edges: &mut Vec<Edge>) {
    for pattern in ARROW_PATTERNS {
        let Some(pos) = stmt.find(pattern) else {
            continue;
        };

        let left = stmt[..pos].trim();
        let rest = &stmt[pos + pattern.len()..];

        let (label, target) = extract_edge_label(pattern, rest);
        let immediate = extract_immediate_node(&target);

        register_node(left, nodes);
        register_node(&immediate, nodes);
        edges.push(Edge {
            from: extract_node_id(left),
            to: extract_node_id(&immediate),
            label,
        });

        // If there's more after the immediate node, it's a chained arrow
        if immediate.len() < target.len() {
            parse_flow_statement(&target, nodes, edges);
        }
        return;
    }

    // No arrow found — standalone node declaration
    register_node(stmt, nodes);
}

/// Extract edge label from pipe-delimited syntax: `-->|label|target`
fn extract_edge_label(pattern: &str, rest: &str) -> (Option<String>, String) {
    if pattern.ends_with('|') {
        if let Some(end_pipe) = rest.find('|') {
            let label = rest[..end_pipe].to_string();
            let target = rest[end_pipe + 1..].trim().to_string();
            return (Some(label), target);
        }
    }
    (None, rest.trim().to_string())
}

/// Extract the first node from a potentially chained string.
/// Stops at the next arrow pattern, respecting bracket pairs.
fn extract_immediate_node(s: &str) -> String {
    let s = s.trim();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip over bracketed content (labels)
        if let Some(&(_, close)) = BRACKET_PAIRS.iter().find(|&&(open, _)| bytes[i] == open) {
            i += 1;
            while i < bytes.len() && bytes[i] != close {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }

        // Check if we're at the start of an arrow pattern
        if matches!(bytes[i], b'-' | b'=' | b'.') {
            let rest = &s[i..];
            if ARROW_PATTERNS.iter().any(|p| rest.starts_with(p)) {
                return s[..i].to_string();
            }
        }

        i += 1;
    }
    s.to_string()
}

/// Extract the node ID (part before any bracket).
fn extract_node_id(s: &str) -> String {
    let s = s.trim();
    s.find(['[', '(', '{'])
        .map_or_else(|| s.to_string(), |pos| s[..pos].trim().to_string())
}

/// Register a node if not already present. Extracts label from brackets if present.
fn register_node(s: &str, nodes: &mut Vec<Node>) {
    let s = s.trim();
    let id = extract_node_id(s);
    if id.is_empty() {
        return;
    }

    // Already registered?
    if nodes.iter().any(|n| n.id.as_deref() == Some(&id)) {
        return;
    }

    let label = extract_bracket_content(s).unwrap_or_else(|| id.clone());
    nodes.push(Node {
        id: Some(id),
        label,
    });
}

/// Extract content between the first bracket pair, if any.
fn extract_bracket_content(s: &str) -> Option<String> {
    let start = s.find(['[', '(', '{'])?;
    let close = match s.as_bytes()[start] {
        b'(' => ')',
        b'{' => '}',
        _ => ']',
    };
    Some(s[start + 1..].trim_end_matches(close).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_simple_chain() {
        let d = parse_flowchart("graph LR\nA-->B-->C").unwrap();
        if let Diagram::Flow(f) = d {
            let edges = f.edges.unwrap();
            assert_eq!(edges.len(), 2);
            assert_eq!(edges[0].from, "A");
            assert_eq!(edges[0].to, "B");
            assert_eq!(edges[1].from, "B");
            assert_eq!(edges[1].to, "C");
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_extract_node_labels_from_brackets() {
        let d = parse_flowchart("flowchart TD\nA[Start]-->B[End]").unwrap();
        if let Diagram::Flow(f) = d {
            let nodes = f.nodes.unwrap();
            assert_eq!(nodes[0].id.as_deref(), Some("A"));
            assert_eq!(nodes[0].label, "Start");
            assert_eq!(nodes[1].id.as_deref(), Some("B"));
            assert_eq!(nodes[1].label, "End");
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_parse_edge_with_pipe_label() {
        let d = parse_flowchart("graph LR\nA-->|yes|B").unwrap();
        if let Diagram::Flow(f) = d {
            let edges = f.edges.unwrap();
            assert_eq!(edges[0].label.as_deref(), Some("yes"));
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_parse_thick_arrow() {
        let d = parse_flowchart("graph LR\nA==>B").unwrap();
        if let Diagram::Flow(f) = d {
            let edges = f.edges.unwrap();
            assert_eq!(edges[0].from, "A");
            assert_eq!(edges[0].to, "B");
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_parse_dashed_arrow() {
        let d = parse_flowchart("graph LR\nA-.->B").unwrap();
        if let Diagram::Flow(f) = d {
            let edges = f.edges.unwrap();
            assert_eq!(edges[0].from, "A");
            assert_eq!(edges[0].to, "B");
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_parse_node_with_parens() {
        let d = parse_flowchart("flowchart TD\nA(Round)-->B").unwrap();
        if let Diagram::Flow(f) = d {
            let nodes = f.nodes.unwrap();
            assert!(nodes.iter().any(|n| n.label == "Round"));
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_error_when_empty_body() {
        let r = parse_flowchart("graph LR\n");
        assert!(r.is_err());
    }

    #[test]
    fn should_infer_nodes_from_edges_when_no_declarations() {
        let d = parse_flowchart("graph LR;X-->Y").unwrap();
        if let Diagram::Flow(f) = d {
            let nodes = f.nodes.unwrap();
            assert!(nodes.iter().any(|n| n.id.as_deref() == Some("X")));
            assert!(nodes.iter().any(|n| n.id.as_deref() == Some("Y")));
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_not_duplicate_nodes() {
        let d = parse_flowchart("graph LR\nA-->B\nA-->C").unwrap();
        if let Diagram::Dag(dag) = d {
            let nodes = dag.nodes.unwrap();
            let a_count = nodes
                .iter()
                .filter(|n| n.id.as_deref() == Some("A"))
                .count();
            assert_eq!(a_count, 1);
        } else {
            panic!("expected Dag for branching graph");
        }
    }

    #[test]
    fn should_parse_subgraph_into_dag_with_groups() {
        let code = "flowchart TD\n  subgraph Frontend\n    A-->B\n  end\n  subgraph Backend\n    C-->D\n  end\n  B-->C";
        let d = parse_flowchart(code).unwrap();
        if let Diagram::Dag(dag) = d {
            let sgs = dag.subgraphs.unwrap();
            assert_eq!(sgs.len(), 2);
            assert_eq!(sgs[0].label, "Frontend");
            assert!(sgs[0].node_ids.contains(&"A".to_string()));
            assert!(sgs[0].node_ids.contains(&"B".to_string()));
            assert_eq!(sgs[1].label, "Backend");
            assert!(sgs[1].node_ids.contains(&"C".to_string()));
            assert!(sgs[1].node_ids.contains(&"D".to_string()));
        } else {
            panic!("expected Dag");
        }
    }

    #[test]
    fn should_use_dag_when_subgraphs_present_even_without_branching() {
        let d = parse_flowchart("flowchart TD\n  subgraph Group\n    A-->B\n  end").unwrap();
        assert!(matches!(d, Diagram::Dag(_)));
    }

    // --- Helper function unit tests ---

    #[test]
    fn has_branching_detects_fan_out() {
        let edges = vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                label: None,
            },
            Edge {
                from: "A".into(),
                to: "C".into(),
                label: None,
            },
        ];
        assert!(has_branching_or_merging(&edges));
    }

    #[test]
    fn has_branching_detects_fan_in() {
        let edges = vec![
            Edge {
                from: "A".into(),
                to: "C".into(),
                label: None,
            },
            Edge {
                from: "B".into(),
                to: "C".into(),
                label: None,
            },
        ];
        assert!(has_branching_or_merging(&edges));
    }

    #[test]
    fn has_branching_false_for_linear() {
        let edges = vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                label: None,
            },
            Edge {
                from: "B".into(),
                to: "C".into(),
                label: None,
            },
        ];
        assert!(!has_branching_or_merging(&edges));
    }

    #[test]
    fn extract_edge_label_with_pipe() {
        let (label, target) = extract_edge_label("-->|", "yes|B");
        assert_eq!(label.as_deref(), Some("yes"));
        assert_eq!(target, "B");
    }

    #[test]
    fn extract_edge_label_no_pipe() {
        let (label, target) = extract_edge_label("-->", "B");
        assert_eq!(label, None);
        assert_eq!(target, "B");
    }

    #[test]
    fn extract_node_id_plain() {
        assert_eq!(extract_node_id("MyNode"), "MyNode");
    }

    #[test]
    fn extract_node_id_with_brackets() {
        assert_eq!(extract_node_id("A[Label]"), "A");
        assert_eq!(extract_node_id("B(Round)"), "B");
        assert_eq!(extract_node_id("C{Diamond}"), "C");
    }

    #[test]
    fn extract_bracket_content_square() {
        assert_eq!(extract_bracket_content("A[Hello]"), Some("Hello".into()));
    }

    #[test]
    fn extract_bracket_content_parens() {
        assert_eq!(extract_bracket_content("B(World)"), Some("World".into()));
    }

    #[test]
    fn extract_bracket_content_none() {
        assert_eq!(extract_bracket_content("Plain"), None);
    }

    #[test]
    fn extract_immediate_node_stops_at_arrow() {
        assert_eq!(extract_immediate_node("B-->C"), "B");
        assert_eq!(extract_immediate_node("X[Label]-->Y"), "X[Label]");
    }

    #[test]
    fn extract_immediate_node_no_arrow() {
        assert_eq!(extract_immediate_node("Standalone"), "Standalone");
    }
}
