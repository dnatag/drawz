use crate::schema::{Diagram, Message, SequenceDiagram};

use super::helpers::skip_first_line;

/// Parse sequenceDiagram.
pub(super) fn parse_sequence(code: &str) -> Result<Diagram, String> {
    let body = skip_first_line(code);
    let mut actors: Vec<String> = Vec::new();
    let mut messages: Vec<Message> = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line
            .strip_prefix("participant ")
            .or_else(|| line.strip_prefix("actor "))
        {
            let name = rest.trim().to_string();
            if !actors.contains(&name) {
                actors.push(name);
            }
        } else if let Some((from_to, label)) = line.split_once(':') {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_basic_messages() {
        let d = parse_sequence("sequenceDiagram\nAlice->>Bob: Hello").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.actors, vec!["Alice", "Bob"]);
            assert_eq!(s.messages.len(), 1);
            assert_eq!(s.messages[0].label, "Hello");
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_parse_participant_declarations() {
        let d =
            parse_sequence("sequenceDiagram\nparticipant A\nparticipant B\nA->>B: msg").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.actors[0], "A");
            assert_eq!(s.actors[1], "B");
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_parse_actor_keyword() {
        let d = parse_sequence("sequenceDiagram\nactor User\nUser->>Server: req").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.actors[0], "User");
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_parse_dashed_reply_arrow() {
        let d = parse_sequence("sequenceDiagram\nA-->>B: reply").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.messages[0].from, "A");
            assert_eq!(s.messages[0].to, "B");
            assert_eq!(s.messages[0].label, "reply");
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_auto_discover_actors_from_messages() {
        let d = parse_sequence("sequenceDiagram\nX->>Y: go\nY->>Z: forward").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.actors, vec!["X", "Y", "Z"]);
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_not_duplicate_actors() {
        let d = parse_sequence("sequenceDiagram\nA->>B: one\nA->>B: two").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.actors, vec!["A", "B"]);
        } else {
            panic!("expected Sequence");
        }
    }

    #[test]
    fn should_error_when_no_actors() {
        let r = parse_sequence("sequenceDiagram\n");
        assert!(r.is_err());
    }

    #[test]
    fn should_skip_empty_lines() {
        let d = parse_sequence("sequenceDiagram\n\n\nA->>B: hi\n\n").unwrap();
        if let Diagram::Sequence(s) = d {
            assert_eq!(s.messages.len(), 1);
        } else {
            panic!("expected Sequence");
        }
    }
}
