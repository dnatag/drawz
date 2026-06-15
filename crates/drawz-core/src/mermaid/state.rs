use crate::schema::{Diagram, Edge, StateDiagram};

use super::helpers::skip_first_line;

/// Parse stateDiagram.
pub(super) fn parse_state(code: &str) -> Result<Diagram, String> {
    let body = skip_first_line(code);
    let mut transitions: Vec<Edge> = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_simple_transition() {
        let d = parse_state("stateDiagram-v2\nA --> B").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions.len(), 1);
            assert_eq!(s.transitions[0].from, "A");
            assert_eq!(s.transitions[0].to, "B");
            assert_eq!(s.transitions[0].label, None);
        } else {
            panic!("expected State");
        }
    }

    #[test]
    fn should_parse_transition_with_label() {
        let d = parse_state("stateDiagram-v2\nIdle --> Running : start").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions[0].label.as_deref(), Some("start"));
        } else {
            panic!("expected State");
        }
    }

    #[test]
    fn should_parse_multiple_transitions() {
        let d = parse_state("stateDiagram-v2\nA --> B\nB --> C\nC --> A").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions.len(), 3);
        } else {
            panic!("expected State");
        }
    }

    #[test]
    fn should_skip_comments() {
        let d = parse_state("stateDiagram-v2\n%% this is a comment\nA --> B").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions.len(), 1);
        } else {
            panic!("expected State");
        }
    }

    #[test]
    fn should_skip_empty_lines() {
        let d = parse_state("stateDiagram-v2\n\nA --> B\n\n").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions.len(), 1);
        } else {
            panic!("expected State");
        }
    }

    #[test]
    fn should_error_when_no_transitions() {
        let r = parse_state("stateDiagram-v2\n%% just a comment\n");
        assert!(r.is_err());
    }

    #[test]
    fn should_handle_star_notation_for_initial_state() {
        let d = parse_state("stateDiagram-v2\n[*] --> Idle").unwrap();
        if let Diagram::State(s) = d {
            assert_eq!(s.transitions[0].from, "[*]");
            assert_eq!(s.transitions[0].to, "Idle");
        } else {
            panic!("expected State");
        }
    }
}
