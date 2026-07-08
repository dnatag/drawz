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
