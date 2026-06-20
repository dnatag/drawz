use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::{FlowDiagram, FlowStep};

/// Render flow as a vertical pipeline with arrows.
///
/// # Errors
///
/// Returns an error if no steps or nodes are provided.
pub(crate) fn render(
    diagram: &FlowDiagram,
    ctx: &mut RenderContext,
) -> Result<Vec<String>, String> {
    let is_horizontal = diagram
        .direction
        .as_deref()
        .is_some_and(|d| d.eq_ignore_ascii_case("lr"));

    if let Some(steps) = &diagram.steps {
        if steps.is_empty() {
            return Err("flow requires at least one step".to_string());
        }
        if is_horizontal {
            let labels: Vec<&str> = steps
                .iter()
                .filter_map(|s| match s {
                    FlowStep::Label(l) => Some(l.as_str()),
                    _ => None,
                })
                .collect();
            Ok(render_horizontal(&labels, ctx))
        } else {
            render_steps(steps, ctx, 0)
        }
    } else if let Some(nodes) = &diagram.nodes {
        if nodes.is_empty() {
            return Err("flow requires at least one node".to_string());
        }
        let edges = diagram.edges.as_deref().unwrap_or(&[]);
        if is_horizontal {
            let labels: Vec<&str> = nodes.iter().map(|n| n.label.as_str()).collect();
            Ok(render_horizontal(&labels, ctx))
        } else {
            Ok(render_graph(nodes, edges, ctx))
        }
    } else {
        Err("flow requires 'steps' or 'nodes' field".to_string())
    }
}

fn render_horizontal(labels: &[&str], _ctx: &mut RenderContext) -> Vec<String> {
    let arrow = "───→";
    let arrow_w = display_width(arrow);

    // Render at natural size — each box is as wide as its label needs
    let widths: Vec<usize> = labels.iter().map(|l| display_width(l) + 4).collect();
    let total_w: usize = widths.iter().sum::<usize>() + (labels.len() - 1) * (arrow_w + 2);

    // If total natural width exceeds inner_width, fall back to vertical
    if total_w > _ctx.inner_width {
        let steps: Vec<FlowStep> = labels
            .iter()
            .map(|&l| FlowStep::Label(l.to_string()))
            .collect();
        return render_steps(&steps, _ctx, 0).unwrap_or_default();
    }

    // Build 3 lines: top borders, middle labels, bottom borders
    let mut top_parts = Vec::new();
    let mut mid_parts = Vec::new();
    let mut bot_parts = Vec::new();

    for (i, (&l, &w)) in labels.iter().zip(&widths).enumerate() {
        top_parts.push(format!("┌{}┐", "─".repeat(w - 2)));
        mid_parts.push(format!("│ {} │", l));
        bot_parts.push(format!("└{}┘", "─".repeat(w - 2)));

        if i < labels.len() - 1 {
            top_parts.push(format!("{:^w$}", "", w = arrow_w + 2));
            mid_parts.push(format!(" {arrow} "));
            bot_parts.push(format!("{:^w$}", "", w = arrow_w + 2));
        }
    }

    // Don't pad to ctx.inner_width — let natural size flow through
    vec![top_parts.join(""), mid_parts.join(""), bot_parts.join("")]
}

fn render_steps(
    steps: &[FlowStep],
    ctx: &mut RenderContext,
    indent: usize,
) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    let prefix = " ".repeat(indent);

    for (i, step) in steps.iter().enumerate() {
        match step {
            FlowStep::Label(label) => {
                let box_lines = render_step_box(label, &prefix, ctx);
                lines.extend(box_lines);
            }
            FlowStep::Sub(sub) => {
                let box_lines = render_step_box(&sub.label, &prefix, ctx);
                lines.extend(box_lines);

                // Render sub-steps inside a dashed frame
                if !sub.steps.is_empty() {
                    let arrow = format!("{prefix}  │");
                    lines.push(fit_line(&arrow, ctx));

                    let frame_prefix = format!("{prefix}  ");
                    let fp_w = display_width(&frame_prefix);
                    let frame_max = ctx.inner_width.saturating_sub(fp_w + 4); // 4 for ╎ + space + space + ╎

                    // Render sub-steps with reduced inner_width
                    let saved_inner = ctx.inner_width;
                    ctx.inner_width = frame_max;
                    let sub_result = render_steps(&sub.steps, ctx, 0);
                    ctx.inner_width = saved_inner;
                    let sub_lines = sub_result?;

                    // Size frame to actual content, not max available width
                    let content_w = sub_lines.iter()
                        .map(|l| display_width(l.trim_end()))
                        .max()
                        .unwrap_or(0);
                    let frame_inner = content_w.min(frame_max);

                    // Top border
                    let border_w = frame_inner + 2; // inner + 2 spaces
                    let top = format!("{frame_prefix}┌{}┐", "╌".repeat(border_w));
                    lines.push(fit_line(&top, ctx));
                    // Content rows
                    for sl in &sub_lines {
                        let padded = pad_right(sl.trim_end(), frame_inner);
                        let row = format!("{frame_prefix}╎ {padded} ╎");
                        lines.push(fit_line(&row, ctx));
                    }
                    // Bottom border
                    let bot = format!("{frame_prefix}└{}┘", "╌".repeat(border_w));
                    lines.push(fit_line(&bot, ctx));
                }
            }
        }

        // Arrow between steps (not after last)
        if i < steps.len() - 1 {
            let arrow = format!("{prefix}  │");
            lines.push(fit_line(&arrow, ctx));
            let arrow = format!("{prefix}  ▼");
            lines.push(fit_line(&arrow, ctx));
        }
    }

    Ok(lines)
}

fn render_step_box(label: &str, prefix: &str, ctx: &mut RenderContext) -> Vec<String> {
    let max_label_w = ctx.inner_width.saturating_sub(prefix.len() + 4); // "[ " + " ]"
    let fitted = if display_width(label) > max_label_w {
        ctx.warnings
            .push("suggestion: some labels truncated to fit width".to_string());
        truncate(label, max_label_w)
    } else {
        label.to_string()
    };

    let box_w = display_width(&fitted) + 4; // "[ " + label + " ]"
    let top = format!("{prefix}┌{}┐", "─".repeat(box_w.saturating_sub(2)));
    let mid = format!("{prefix}│ {fitted} │");
    let bot = format!("{prefix}└{}┘", "─".repeat(box_w.saturating_sub(2)));

    vec![
        fit_line(&top, ctx),
        fit_line(&mid, ctx),
        fit_line(&bot, ctx),
    ]
}

fn render_graph(
    nodes: &[crate::schema::Node],
    edges: &[crate::schema::Edge],
    ctx: &mut RenderContext,
) -> Vec<String> {
    // Render nodes in order with edges as arrows between them
    // Build adjacency: for each node, find outgoing edges
    let mut lines = Vec::new();

    for (i, node) in nodes.iter().enumerate() {
        let label = &node.label;
        let box_lines = render_step_box(label, "", ctx);
        lines.extend(box_lines);

        // Find edge from this node
        let node_id = node.id.as_deref().unwrap_or(&node.label);
        let outgoing: Vec<&str> = edges
            .iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.label.as_deref().unwrap_or(""))
            .collect();

        if i < nodes.len() - 1 {
            if let Some(edge_label) = outgoing.first().filter(|l| !l.is_empty()) {
                let arrow = format!("  │ {edge_label}");
                lines.push(fit_line(&arrow, ctx));
            } else {
                lines.push(fit_line("  │", ctx));
            }
            lines.push(fit_line("  ▼", ctx));
        }
    }

    lines
}

fn fit_line(line: &str, ctx: &mut RenderContext) -> String {
    let w = display_width(line);
    if w > ctx.inner_width {
        pad_right(&truncate(line, ctx.inner_width), ctx.inner_width)
    } else {
        pad_right(line, ctx.inner_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::display_width;
    use crate::result::RenderContext;
    use crate::schema::{Edge, FlowDiagram, FlowStep, Node, SubFlow};

    fn ctx(width: usize) -> RenderContext {
        RenderContext {
            inner_width: width,
            total_width: u16::try_from(width).unwrap(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn should_render_boxes_and_arrows_when_linear_steps() {
        let d = FlowDiagram {
            title: None,
            direction: None,
            steps: Some(vec![
                FlowStep::Label("A".into()),
                FlowStep::Label("B".into()),
            ]),
            nodes: None,
            edges: None,
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains('A')));
        assert!(lines.iter().any(|l| l.contains('▼')));
        for l in &lines {
            assert_eq!(display_width(l), 30);
        }
    }

    #[test]
    fn should_render_dashed_frame_when_nested_subflow() {
        let d = FlowDiagram {
            title: None,
            direction: None,
            steps: Some(vec![FlowStep::Sub(SubFlow {
                label: "Parent".into(),
                steps: vec![FlowStep::Label("Child".into())],
            })]),
            nodes: None,
            edges: None,
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Parent")));
        assert!(lines.iter().any(|l| l.contains("Child")));
        for l in &lines {
            assert_eq!(display_width(l), 30);
        }
    }

    #[test]
    fn should_render_edge_labels_when_graph_mode() {
        let d = FlowDiagram {
            title: None,
            direction: None,
            steps: None,
            nodes: Some(vec![
                Node {
                    id: Some("a".into()),
                    label: "Start".into(),
                },
                Node {
                    id: Some("b".into()),
                    label: "End".into(),
                },
            ]),
            edges: Some(vec![Edge {
                from: "a".into(),
                to: "b".into(),
                label: Some("go".into()),
            }]),
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("go")));
        for l in &lines {
            assert_eq!(display_width(l), 30);
        }
    }

    #[test]
    fn should_return_error_when_steps_empty() {
        let d = FlowDiagram {
            title: None,
            direction: None,
            steps: Some(vec![]),
            nodes: None,
            edges: None,
        };
        assert!(render(&d, &mut ctx(30)).is_err());
    }

    #[test]
    fn should_return_error_when_no_steps_or_nodes() {
        let d = FlowDiagram {
            title: None,
            direction: None,
            steps: None,
            nodes: None,
            edges: None,
        };
        assert!(render(&d, &mut ctx(30)).is_err());
    }
}
