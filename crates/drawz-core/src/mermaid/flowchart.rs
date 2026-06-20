use crate::schema::{DagDiagram, Diagram, Edge, FlowDiagram, Node};

use super::helpers::{skip_first_line, split_statements};

/// Parse `graph LR; A-->B-->C` or multiline flowchart.
pub(super) fn parse_flowchart(code: &str) -> Result<Diagram, String> {
    // Extract direction from first line (e.g., "graph LR", "flowchart TD")
    let first_part = code.split(['\n', ';']).next().unwrap_or("").trim();
    let direction = first_part.split_whitespace().nth(1).map(String::from);

    let body = skip_first_line(code);

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();

    for segment in split_statements(body) {
        // Skip subgraph/end directives (not renderable as nodes)
        if segment.starts_with("subgraph") || segment == "end" {
            continue;
        }
        parse_flow_statement(segment, &mut nodes, &mut edges);
    }

    if nodes.is_empty() && edges.is_empty() {
        return Err("flowchart has no nodes or edges".to_string());
    }

    // Detect branching/merging: if any node has multiple outgoing or incoming edges, use DAG
    let has_branching = edges.iter().any(|e1| {
        edges.iter().filter(|e2| e2.from == e1.from).count() > 1
            || edges.iter().filter(|e2| e2.to == e1.to).count() > 1
    });

    if has_branching {
        Ok(Diagram::Dag(DagDiagram {
            title: None,
            nodes: Some(nodes),
            edges,
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

fn parse_flow_statement(stmt: &str, nodes: &mut Vec<Node>, edges: &mut Vec<Edge>) {
    let arrow_patterns = ["-->|", "==>|", "-.->", "-->", "---", "==>"];

    for pattern in &arrow_patterns {
        if let Some(pos) = stmt.find(pattern) {
            let left = stmt[..pos].trim();
            let right_start = pos + pattern.len();
            let rest = &stmt[right_start..];

            let (label, target) = if pattern.ends_with('|') {
                if let Some(end_pipe) = rest.find('|') {
                    let lbl = rest[..end_pipe].to_string();
                    let tgt = rest[end_pipe + 1..].trim().to_string();
                    (Some(lbl), tgt)
                } else {
                    (None, rest.trim().to_string())
                }
            } else {
                (None, rest.trim().to_string())
            };

            let immediate = extract_immediate_node(&target);
            let from_id = extract_node_id(left);
            let to_id = extract_node_id(&immediate);

            register_node(left, nodes);
            register_node(&immediate, nodes);
            edges.push(Edge {
                from: from_id,
                to: to_id,
                label,
            });

            if immediate.len() < target.len() {
                parse_flow_statement(&target, nodes, edges);
            }
            return;
        }
    }

    register_node(stmt, nodes);
}

fn extract_immediate_node(s: &str) -> String {
    let s = s.trim();
    let arrow_patterns = ["-->|", "==>|", "-.->", "-->", "---", "==>"];
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'[' | b'(' | b'{' => {
                let close = match bytes[i] {
                    b'[' => b']',
                    b'(' => b')',
                    _ => b'}',
                };
                i += 1;
                while i < bytes.len() && bytes[i] != close {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            b'-' | b'=' | b'.' => {
                // Only split if this is the start of a full arrow pattern
                let rest = &s[i..];
                if arrow_patterns.iter().any(|p| rest.starts_with(p)) {
                    return s[..i].to_string();
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    s.to_string()
}

fn extract_node_id(s: &str) -> String {
    let s = s.trim();
    if let Some(bracket) = s.find(['[', '(', '{']) {
        s[..bracket].trim().to_string()
    } else {
        s.to_string()
    }
}

fn register_node(s: &str, nodes: &mut Vec<Node>) {
    let s = s.trim();
    let id = extract_node_id(s);
    if id.is_empty() {
        return;
    }

    let label = if let Some(start) = s.find(['[', '(', '{']) {
        let end_char = match s.as_bytes()[start] {
            b'(' => ')',
            b'{' => '}',
            _ => ']',
        };
        s[start + 1..].trim_end_matches(end_char).to_string()
    } else {
        id.clone()
    };

    if !nodes.iter().any(|n| n.id.as_deref() == Some(&id)) {
        nodes.push(Node {
            id: Some(id),
            label,
        });
    }
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
}
