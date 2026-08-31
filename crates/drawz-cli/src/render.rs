use std::io::Read;

use clap::Subcommand;
use drawz_core::schema::*;

#[derive(Subcommand)]
pub enum RenderType {
    /// Render a flow/pipeline diagram
    Flow {
        /// Comma-separated step labels
        #[arg(long)]
        steps: String,
        /// Direction: LR for horizontal, TD for vertical (default)
        #[arg(long)]
        direction: Option<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render a table
    Table {
        /// Comma-separated column headers
        #[arg(long)]
        headers: String,
        /// Table row (comma-separated values). Repeat for multiple rows.
        #[arg(long = "row")]
        rows: Vec<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render a tree from indented text
    Tree {
        /// Indented text (use \n for newlines or pass via stdin)
        #[arg(long)]
        indent: Option<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render a sequence diagram
    Sequence {
        /// Comma-separated actor names
        #[arg(long)]
        actors: String,
        /// Message in from:to:label format. Repeat for multiple.
        #[arg(long = "msg")]
        messages: Vec<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render a state machine diagram
    State {
        /// Transition in from:to or from:to:label format. Repeat for multiple.
        #[arg(long = "edge")]
        edges: Vec<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render a DAG (directed acyclic graph)
    Dag {
        /// Edge in from:to or from:to:label format. Repeat for multiple.
        #[arg(long = "edge")]
        edges: Vec<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render Mermaid code
    Mermaid {
        /// Mermaid diagram code (or reads from stdin if omitted)
        #[arg(long)]
        code: Option<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render freeform text with alignment guarantee
    Freeform {
        /// Text content (or reads from stdin if omitted)
        #[arg(long)]
        content: Option<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Render a component/architecture diagram
    Component {
        /// Group in label:node1,node2,... format. Repeat for multiple groups.
        #[arg(long = "group")]
        groups: Vec<String>,
        /// Connection in from:to or from:to:label format. Repeat for multiple.
        #[arg(long = "connect")]
        connections: Vec<String>,
        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
}

pub fn build_diagram(render_type: RenderType) -> Result<Diagram, String> {
    match render_type {
        RenderType::Flow {
            steps,
            direction,
            title,
        } => Ok(Diagram::Flow(FlowDiagram {
            title,
            direction,
            steps: Some(
                steps
                    .split(',')
                    .map(|s| FlowStep::Label(s.trim().to_string()))
                    .collect(),
            ),
            nodes: None,
            edges: None,
        })),

        RenderType::Table {
            headers,
            rows,
            title,
        } => {
            let headers: Vec<String> = headers.split(',').map(|s| s.trim().to_string()).collect();
            let rows: Vec<Vec<String>> = rows
                .iter()
                .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
                .collect();
            if rows.is_empty() {
                return Err("at least one --row is required".into());
            }
            Ok(Diagram::Table(TableDiagram {
                title,
                headers,
                rows,
            }))
        }

        RenderType::Tree { indent, title } => {
            let text = match indent {
                Some(t) => t.replace("\\n", "\n"),
                None => read_stdin()?,
            };
            Ok(Diagram::Tree(TreeDiagram {
                title,
                root: None,
                indent: Some(text),
            }))
        }

        RenderType::Sequence {
            actors,
            messages,
            title,
        } => {
            let actors: Vec<String> = actors.split(',').map(|s| s.trim().to_string()).collect();
            let messages: Vec<Message> = messages
                .iter()
                .map(|m| {
                    let parts: Vec<&str> = m.splitn(3, ':').collect();
                    if parts.len() < 3 {
                        return Err(format!(
                            "invalid message format '{m}', expected from:to:label"
                        ));
                    }
                    Ok(Message {
                        from: parts[0].to_string(),
                        to: parts[1].to_string(),
                        label: parts[2].to_string(),
                    })
                })
                .collect::<Result<_, _>>()?;
            if messages.is_empty() {
                return Err("at least one --msg is required".into());
            }
            Ok(Diagram::Sequence(SequenceDiagram {
                title,
                actors,
                messages,
            }))
        }

        RenderType::State { edges, title } => {
            let transitions: Vec<Edge> = edges
                .iter()
                .map(|e| {
                    let (from, to, label) = parse_edge(e)?;
                    Ok(Edge { from, to, label })
                })
                .collect::<Result<_, String>>()?;
            if transitions.is_empty() {
                return Err("at least one --edge is required".into());
            }
            Ok(Diagram::State(StateDiagram {
                title,
                states: None,
                transitions,
            }))
        }

        RenderType::Dag { edges, title } => {
            let parsed_edges: Vec<Edge> = edges
                .iter()
                .map(|e| {
                    let (from, to, label) = parse_edge(e)?;
                    Ok(Edge { from, to, label })
                })
                .collect::<Result<_, String>>()?;
            if parsed_edges.is_empty() {
                return Err("at least one --edge is required".into());
            }
            Ok(Diagram::Dag(DagDiagram {
                title,
                nodes: None,
                edges: parsed_edges,
                subgraphs: None,
            }))
        }

        RenderType::Mermaid { code, title } => {
            let code = match code {
                Some(c) => c,
                None => read_stdin()?,
            };
            Ok(Diagram::Mermaid(MermaidDiagram { title, code }))
        }

        RenderType::Freeform { content, title } => {
            let content = match content {
                Some(c) => c.replace("\\n", "\n"),
                None => read_stdin()?,
            };
            Ok(Diagram::Freeform(FreeformDiagram {
                title,
                content: Some(content),
                lines: None,
            }))
        }

        RenderType::Component {
            groups,
            connections,
            title,
        } => {
            let parsed_groups: Vec<ComponentGroup> = groups
                .iter()
                .map(|g| {
                    let (label, rest) = g.split_once(':').ok_or_else(|| {
                        format!("invalid group format '{g}', expected label:node1,node2,...")
                    })?;
                    let nodes: Vec<String> =
                        rest.split(',').map(|s| s.trim().to_string()).collect();
                    Ok(ComponentGroup {
                        label: label.trim().to_string(),
                        nodes,
                        chains: Vec::new(),
                        edges: Vec::new(),
                    })
                })
                .collect::<Result<_, String>>()?;
            if parsed_groups.is_empty() {
                return Err("at least one --group is required".into());
            }
            let parsed_connections: Vec<Connection> = connections
                .iter()
                .map(|c| {
                    let (from, to, label) = parse_edge(c)?;
                    Ok(Connection { from, to, label })
                })
                .collect::<Result<_, String>>()?;
            Ok(Diagram::Component(ComponentDiagram {
                title,
                groups: parsed_groups,
                connections: parsed_connections,
            }))
        }
    }
}

fn read_stdin() -> Result<String, String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    Ok(buf)
}

/// Parse "from:to" or "from:to:label" into (from, to, Option<label>)
fn parse_edge(s: &str) -> Result<(String, String, Option<String>), String> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    match parts.len() {
        2 => Ok((parts[0].to_string(), parts[1].to_string(), None)),
        3 => Ok((
            parts[0].to_string(),
            parts[1].to_string(),
            Some(parts[2].to_string()),
        )),
        _ => Err(format!(
            "invalid edge format '{s}', expected from:to or from:to:label"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_component_diagram_from_flags() {
        let diagram = build_diagram(RenderType::Component {
            groups: vec![
                "Frontend:UI,Router".to_string(),
                "Backend:API,DB".to_string(),
            ],
            connections: vec!["Router:API:REST".to_string()],
            title: Some("Architecture".to_string()),
        })
        .unwrap();

        match diagram {
            Diagram::Component(d) => {
                assert_eq!(d.title.as_deref(), Some("Architecture"));
                assert_eq!(d.groups.len(), 2);
                assert_eq!(d.groups[0].label, "Frontend");
                assert_eq!(d.groups[0].nodes, vec!["UI", "Router"]);
                assert_eq!(d.connections.len(), 1);
                assert_eq!(d.connections[0].from, "Router");
                assert_eq!(d.connections[0].to, "API");
                assert_eq!(d.connections[0].label.as_deref(), Some("REST"));
            }
            _ => panic!("expected Component diagram"),
        }
    }

    #[test]
    fn should_error_when_no_groups_given() {
        let result = build_diagram(RenderType::Component {
            groups: vec![],
            connections: vec!["A:B".to_string()],
            title: None,
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--group is required"));
    }

    #[test]
    fn should_error_on_malformed_group() {
        let result = build_diagram(RenderType::Component {
            groups: vec!["NoColonHere".to_string()],
            connections: vec![],
            title: None,
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid group format"));
    }

    #[test]
    fn should_build_component_diagram_without_connections() {
        let diagram = build_diagram(RenderType::Component {
            groups: vec!["Solo:A,B,C".to_string()],
            connections: vec![],
            title: None,
        })
        .unwrap();

        match diagram {
            Diagram::Component(d) => {
                assert_eq!(d.groups.len(), 1);
                assert!(d.connections.is_empty());
            }
            _ => panic!("expected Component diagram"),
        }
    }
}
