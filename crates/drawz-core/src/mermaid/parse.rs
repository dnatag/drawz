use crate::schema::Diagram;

use super::{flowchart, sequence, state};

/// Parse a Mermaid code block into an internal Diagram type.
///
/// # Errors
///
/// Returns an error if the Mermaid syntax is unrecognized or unsupported.
pub fn parse(code: &str) -> Result<Diagram, String> {
    let normalized = code.replace("\\n", "\n");
    let trimmed = normalized.trim();

    if trimmed.starts_with("graph ") || trimmed.starts_with("flowchart ") {
        flowchart::parse_flowchart(trimmed)
    } else if trimmed.starts_with("sequenceDiagram") {
        sequence::parse_sequence(trimmed)
    } else if trimmed.starts_with("stateDiagram") {
        state::parse_state(trimmed)
    } else {
        Err("unsupported mermaid diagram type. Supported: flowchart, sequenceDiagram, stateDiagram".to_string())
    }
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
