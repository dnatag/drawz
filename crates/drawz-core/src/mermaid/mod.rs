//! Mermaid subset parser — converts flowchart, sequenceDiagram, stateDiagram
//! to internal diagram types.

use crate::schema::{
    Diagram, Edge, FlowDiagram, Message, Node, SequenceDiagram, StateDiagram,
};

/// Parse a Mermaid code block into an internal Diagram type.
///
/// # Errors
///
/// Returns an error if the Mermaid syntax is unrecognized or unsupported.
pub fn parse(code: &str) -> Result<Diagram, String> {
    // Handle literal \n (backslash-n) that arrives from JSON transport
    let normalized = code.replace("\\n", "\n");
    let trimmed = normalized.trim();

    if trimmed.starts_with("graph ") || trimmed.starts_with("flowchart ") {
        parse_flowchart(trimmed)
    } else if trimmed.starts_with("sequenceDiagram") {
        parse_sequence(trimmed)
    } else if trimmed.starts_with("stateDiagram") {
        parse_state(trimmed)
    } else {
        Err("unsupported mermaid diagram type. Supported: flowchart, sequenceDiagram, stateDiagram".to_string())
    }
}

/// Parse `graph LR; A-->B-->C` or multiline flowchart.
fn parse_flowchart(code: &str) -> Result<Diagram, String> {
    // Skip the first line (graph LR / flowchart TD etc.)
    let body = skip_first_line(code);

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();

    for segment in split_statements(body) {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        parse_flow_statement(segment, &mut nodes, &mut edges);
    }

    if nodes.is_empty() && edges.is_empty() {
        return Err("flowchart has no nodes or edges".to_string());
    }

    // If we only have edges, infer nodes
    if nodes.is_empty() {
        for e in &edges {
            if !nodes.iter().any(|n| n.id.as_deref() == Some(&e.from) || n.label == e.from) {
                nodes.push(Node { id: Some(e.from.clone()), label: e.from.clone() });
            }
            if !nodes.iter().any(|n| n.id.as_deref() == Some(&e.to) || n.label == e.to) {
                nodes.push(Node { id: Some(e.to.clone()), label: e.to.clone() });
            }
        }
    }

    Ok(Diagram::Flow(FlowDiagram {
        title: None,
        steps: None,
        nodes: Some(nodes),
        edges: Some(edges),
    }))
}

/// Parse flow statement like `A-->B`, `A-->|label|B`, `A[Label]`, `A-->B-->C`.
fn parse_flow_statement(stmt: &str, nodes: &mut Vec<Node>, edges: &mut Vec<Edge>) {
    // Try to match edge patterns: A-->B, A-->|label|B, A---B, A==>B
    let arrow_patterns = ["-->|", "==>|", "-.->", "-->", "---", "==>"];

    for pattern in &arrow_patterns {
        if let Some(pos) = stmt.find(pattern) {
            let left = stmt[..pos].trim();
            let right_start = pos + pattern.len();
            let rest = &stmt[right_start..];

            let (label, target) = if pattern.ends_with('|') {
                // A-->|label|B
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

            // Extract immediate target node (before any chained arrow)
            let immediate = extract_immediate_node(&target);
            let from_id = extract_node_id(left);
            let to_id = extract_node_id(&immediate);

            register_node(left, nodes);
            register_node(&immediate, nodes);
            edges.push(Edge { from: from_id, to: to_id, label });

            // Recursively parse chained remainder (e.g., B-->C from A-->B-->C)
            if immediate.len() < target.len() {
                parse_flow_statement(&target, nodes, edges);
            }
            return;
        }
    }

    // No arrow found — it's a node declaration like `A[Label]`
    register_node(stmt, nodes);
}

/// Extract the immediate node from a potentially chained string like `B-->C`.
fn extract_immediate_node(s: &str) -> String {
    let s = s.trim();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'[' | b'(' | b'{' => {
                let close = match bytes[i] { b'[' => b']', b'(' => b')', _ => b'}' };
                i += 1;
                while i < bytes.len() && bytes[i] != close { i += 1; }
                if i < bytes.len() { i += 1; }
            }
            b'-' | b'=' | b'.' => return s[..i].to_string(),
            _ => i += 1,
        }
    }
    s.to_string()
}

/// Extract node ID from `A[Label]` → `A`, or plain `A` → `A`.
fn extract_node_id(s: &str) -> String {
    let s = s.trim();
    if let Some(bracket) = s.find(['[', '(', '{']) {
        s[..bracket].trim().to_string()
    } else {
        s.to_string()
    }
}

/// Register a node with optional label from bracket syntax.
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
        nodes.push(Node { id: Some(id), label });
    }
}

/// Parse sequenceDiagram.
fn parse_sequence(code: &str) -> Result<Diagram, String> {
    let body = skip_first_line(code);
    let mut actors: Vec<String> = Vec::new();
    let mut messages: Vec<Message> = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("participant ").or_else(|| line.strip_prefix("actor ")) {
            let name = rest.trim().to_string();
            if !actors.contains(&name) {
                actors.push(name);
            }
        } else if let Some((from_to, label)) = line.split_once(':') {
            // A->>B: message or A-->>B: message
            let from_to = from_to.trim();
            let arrow_patterns = ["-->>", "->>", "-->", "->", "--x", "-x"];
            for pat in &arrow_patterns {
                if let Some(pos) = from_to.find(pat) {
                    let from = from_to[..pos].trim().to_string();
                    let to = from_to[pos + pat.len()..].trim().to_string();

                    if !actors.contains(&from) {
                        actors.push(from.clone());
                    }
                    if !actors.contains(&to) {
                        actors.push(to.clone());
                    }

                    messages.push(Message {
                        from,
                        to,
                        label: label.trim().to_string(),
                    });
                    break;
                }
            }
        }
    }

    if actors.is_empty() {
        return Err("sequenceDiagram has no actors".to_string());
    }

    Ok(Diagram::Sequence(SequenceDiagram {
        title: None,
        actors,
        messages,
    }))
}

/// Parse stateDiagram.
fn parse_state(code: &str) -> Result<Diagram, String> {
    let body = skip_first_line(code);
    let mut transitions: Vec<Edge> = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Pattern: StateA --> StateB : label
        if let Some(pos) = line.find("-->") {
            let from = line[..pos].trim().to_string();
            let rest = &line[pos + 3..];
            let (to, label) = if let Some((t, l)) = rest.split_once(':') {
                (t.trim().to_string(), Some(l.trim().to_string()))
            } else {
                (rest.trim().to_string(), None)
            };

            if !from.is_empty() && !to.is_empty() {
                transitions.push(Edge { from, to, label });
            }
        }
    }

    if transitions.is_empty() {
        return Err("stateDiagram has no transitions".to_string());
    }

    Ok(Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions,
    }))
}

fn skip_first_line(code: &str) -> &str {
    // Find end of the declaration keyword (e.g., "graph LR", "flowchart TD", "sequenceDiagram")
    // Everything after it (on the same line or subsequent lines) is body.
    // The declaration ends at the first newline or semicolon.
    if let Some(pos) = code.find(['\n', ';']) {
        &code[pos + 1..]
    } else {
        ""
    }
}

fn split_statements(body: &str) -> Vec<&str> {
    // Split on newlines and semicolons
    body.split(['\n', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("%%"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_flowchart_to_flow() {
        let d = parse("graph LR; A-->B-->C").unwrap();
        assert!(matches!(d, Diagram::Flow(_)));
    }

    #[test]
    fn should_parse_flowchart_with_labels() {
        let d = parse("flowchart TD\nA[Start]-->B[End]").unwrap();
        if let Diagram::Flow(f) = d {
            let nodes = f.nodes.unwrap();
            assert!(nodes.iter().any(|n| n.label == "Start"));
            assert!(nodes.iter().any(|n| n.label == "End"));
        } else {
            panic!("expected Flow");
        }
    }

    #[test]
    fn should_parse_sequence_diagram() {
        let d = parse("sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.actors.len(), 2);
            assert_eq!(s.messages.len(), 2);
            assert_eq!(s.messages[0].label, "Hello");
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_parse_state_diagram() {
        let d = parse("stateDiagram-v2\nIdle --> Running : start\nRunning --> Done").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions.len(), 2);
            assert_eq!(s.transitions[0].label.as_deref(), Some("start"));
        } else {
            panic!("expected State");
        }
    }

    #[test]
    fn should_return_error_for_unsupported_type() {
        assert!(parse("pie\n\"A\": 50").is_err());
    }

    #[test]
    fn should_parse_edge_labels_with_pipe_syntax() {
        let d = parse("graph LR; A-->|label|B").unwrap();
        if let Diagram::Flow(f) = d {
            let edges = f.edges.unwrap();
            assert_eq!(edges[0].label.as_deref(), Some("label"));
        } else {
            panic!("expected Flow");
        }
    }
}
