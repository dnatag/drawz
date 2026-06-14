//! DAG (directed acyclic graph) renderer — topological layers with dependency arrows.

use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::DagDiagram;

/// Render a DAG as layered nodes with vertical dependency arrows.
///
/// # Errors
///
/// Returns an error if edges are empty and no nodes provided.
pub fn render(diagram: &DagDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    if diagram.edges.is_empty() && diagram.nodes.is_none() {
        return Err("dag requires at least one edge or node".to_string());
    }

    // Collect all node labels
    let mut node_ids: Vec<&str> = Vec::new();
    if let Some(nodes) = &diagram.nodes {
        for n in nodes {
            let id = n.id.as_deref().unwrap_or(&n.label);
            if !node_ids.contains(&id) {
                node_ids.push(id);
            }
        }
    }
    for e in &diagram.edges {
        if !node_ids.contains(&e.from.as_str()) {
            node_ids.push(&e.from);
        }
        if !node_ids.contains(&e.to.as_str()) {
            node_ids.push(&e.to);
        }
    }

    // Topological sort via Kahn's algorithm
    let layers = topo_layers(&node_ids, &diagram.edges);

    let mut lines = Vec::new();

    for (layer_idx, layer) in layers.iter().enumerate() {
        // Render nodes in this layer as a row of boxes
        let labels: Vec<&str> = layer
            .iter()
            .map(|&id| get_label(id, diagram))
            .collect();
        let layer_lines = render_layer(&labels, ctx);
        lines.extend(layer_lines);

        // Arrow to next layer (if not last)
        if layer_idx < layers.len() - 1 {
            lines.push(fit_line("  │", ctx));
            lines.push(fit_line("  ▼", ctx));
        }
    }

    Ok(lines)
}

fn get_label<'a>(id: &'a str, diagram: &'a DagDiagram) -> &'a str {
    if let Some(nodes) = &diagram.nodes {
        nodes
            .iter()
            .find(|n| n.id.as_deref().unwrap_or(&n.label) == id)
            .map_or(id, |n| &n.label)
    } else {
        id
    }
}

/// Topological layering: nodes with no incoming edges go to layer 0, etc.
fn topo_layers<'a>(nodes: &[&'a str], edges: &[crate::schema::Edge]) -> Vec<Vec<&'a str>> {
    let mut in_degree: Vec<usize> = vec![0; nodes.len()];
    for e in edges {
        if let Some(to_idx) = nodes.iter().position(|&n| n == e.to) {
            in_degree[to_idx] += 1;
        }
    }

    let mut layers = Vec::new();
    let mut remaining: Vec<bool> = vec![true; nodes.len()];

    loop {
        let layer: Vec<&'a str> = nodes
            .iter()
            .enumerate()
            .filter(|&(i, _)| remaining[i] && in_degree[i] == 0)
            .map(|(_, &n)| n)
            .collect();

        if layer.is_empty() {
            // Remaining nodes have cycles — emit them as final layer
            let cyclic: Vec<&'a str> = nodes
                .iter()
                .enumerate()
                .filter(|&(i, _)| remaining[i])
                .map(|(_, &n)| n)
                .collect();
            if !cyclic.is_empty() {
                layers.push(cyclic);
            }
            break;
        }

        // Mark layer nodes as processed, reduce in-degrees
        for &node in &layer {
            let Some(idx) = nodes.iter().position(|&n| n == node) else { continue };
            remaining[idx] = false;
            for e in edges {
                if e.from == node {
                    if let Some(to_idx) = nodes.iter().position(|&n| n == e.to) {
                        in_degree[to_idx] = in_degree[to_idx].saturating_sub(1);
                    }
                }
            }
        }

        layers.push(layer);
    }

    layers
}

fn render_layer(
    labels: &[&str],
    ctx: &mut RenderContext,
) -> Vec<String> {
    if labels.len() == 1 {
        return render_node_box(labels[0], ctx);
    }

    // Multiple nodes in one layer — render inline
    let joined = labels.join("  →  ");

    if display_width(&joined) + 4 <= ctx.inner_width {
        render_node_box(&joined, ctx)
    } else {
        // Fall back to one box per line
        let mut lines = Vec::new();
        for (i, &label) in labels.iter().enumerate() {
            lines.extend(render_node_box(label, ctx));
            if i < labels.len() - 1 {
                lines.push(fit_line("", ctx));
            }
        }
        lines
    }
}

fn render_node_box(label: &str, ctx: &mut RenderContext) -> Vec<String> {
    let max_w = ctx.inner_width.saturating_sub(4);
    let fitted = if display_width(label) > max_w {
        ctx.warnings
            .push("suggestion: some node labels truncated to fit width".to_string());
        truncate(label, max_w)
    } else {
        label.to_string()
    };

    let box_w = display_width(&fitted) + 4;
    let top = format!("┌{}┐", "─".repeat(box_w.saturating_sub(2)));
    let mid = format!("│ {fitted} │");
    let bot = format!("└{}┘", "─".repeat(box_w.saturating_sub(2)));

    vec![
        fit_line(&top, ctx),
        fit_line(&mid, ctx),
        fit_line(&bot, ctx),
    ]
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
    use crate::schema::{DagDiagram, Edge, Node};

    fn ctx(width: usize) -> RenderContext {
        RenderContext { inner_width: width, total_width: u16::try_from(width).unwrap(), warnings: Vec::new() }
    }

    #[test]
    fn should_render_layers_when_edges_provided() {
        let d = DagDiagram {
            title: None, nodes: None,
            edges: vec![
                Edge { from: "A".into(), to: "B".into(), label: None },
                Edge { from: "A".into(), to: "C".into(), label: None },
                Edge { from: "B".into(), to: "D".into(), label: None },
                Edge { from: "C".into(), to: "D".into(), label: None },
            ],
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        assert!(lines.iter().any(|l| l.contains('A')));
        assert!(lines.iter().any(|l| l.contains('D')));
        assert!(lines.iter().any(|l| l.contains('▼')));
        for l in &lines { assert_eq!(display_width(l), 40); }
    }

    #[test]
    fn should_return_error_when_no_edges_or_nodes() {
        let d = DagDiagram { title: None, nodes: None, edges: vec![] };
        assert!(render(&d, &mut ctx(40)).is_err());
    }

    #[test]
    fn should_use_node_labels_when_provided() {
        let d = DagDiagram {
            title: None,
            nodes: Some(vec![
                Node { id: Some("a".into()), label: "Start".into() },
                Node { id: Some("b".into()), label: "End".into() },
            ]),
            edges: vec![Edge { from: "a".into(), to: "b".into(), label: None }],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Start")));
        assert!(lines.iter().any(|l| l.contains("End")));
        for l in &lines { assert_eq!(display_width(l), 30); }
    }
}
