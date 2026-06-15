//! DAG (directed acyclic graph) renderer — topological layers with dependency arrows.

use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::DagDiagram;

/// Render a DAG as layered nodes with vertical dependency arrows.
///
/// # Errors
///
/// Returns an error if edges are empty and no nodes provided.
pub(crate) fn render(diagram: &DagDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
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

    let components = connected_components(&node_ids, &diagram.edges);

    let mut lines = Vec::new();

    for (comp_idx, component) in components.iter().enumerate() {
        let comp_edges: Vec<&crate::schema::Edge> = diagram.edges.iter()
            .filter(|e| component.contains(&e.from.as_str()) || component.contains(&e.to.as_str()))
            .collect();

        let (layers, cyclic) = topo_layers(component, &comp_edges);

        if !cyclic.is_empty() {
            let names = cyclic.join(", ");
            return Err(format!("cycle detected among nodes: {names}"));
        }

        if layers.is_empty() {
            continue;
        }

        if comp_idx > 0 {
            lines.push(fit_line("", ctx));
        }

        for (layer_idx, layer) in layers.iter().enumerate() {
            let labels: Vec<&str> = layer
                .iter()
                .map(|&id| get_label(id, diagram))
                .collect();
            let layer_lines = render_layer(&labels, ctx);
            lines.extend(layer_lines);

            if layer_idx < layers.len() - 1 {
                lines.push(fit_line("  │", ctx));
                lines.push(fit_line("  ▼", ctx));
            }
        }
    }

    if lines.is_empty() {
        return Err("dag has no renderable layers".to_string());
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

/// Group nodes into connected components (treating edges as undirected).
fn connected_components<'a>(nodes: &[&'a str], edges: &[crate::schema::Edge]) -> Vec<Vec<&'a str>> {
    fn find(parent: &mut Vec<usize>, i: usize) -> usize {
        if parent[i] != i { parent[i] = find(parent, parent[i]); }
        parent[i]
    }

    let n = nodes.len();
    let mut parent: Vec<usize> = (0..n).collect();

    for e in edges {
        let Some(a) = nodes.iter().position(|&nd| nd == e.from) else { continue };
        let Some(b) = nodes.iter().position(|&nd| nd == e.to) else { continue };
        let ra = find(&mut parent, a);
        let rb = find(&mut parent, b);
        if ra != rb { parent[ra] = rb; }
    }

    let mut groups: std::collections::HashMap<usize, Vec<&'a str>> = std::collections::HashMap::new();
    for (i, &node) in nodes.iter().enumerate() {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(node);
    }

    // Stable order: sort by the index of the first node in each group
    let mut components: Vec<Vec<&'a str>> = groups.into_values().collect();
    components.sort_by_key(|c| nodes.iter().position(|&n| n == c[0]).unwrap_or(0));
    components
}

/// Topological layering: nodes with no incoming edges go to layer 0, etc.
/// Returns (layers, `cyclic_nodes`).
fn topo_layers<'a>(nodes: &[&'a str], edges: &[&crate::schema::Edge]) -> (Vec<Vec<&'a str>>, Vec<&'a str>) {
    let mut in_degree: Vec<usize> = vec![0; nodes.len()];
    for e in edges {
        if e.from != e.to {
            if let Some(to_idx) = nodes.iter().position(|&n| n == e.to) {
                in_degree[to_idx] += 1;
            }
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
            // Remaining nodes have cycles — this is not a valid DAG
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

    let cyclic: Vec<&'a str> = nodes
        .iter()
        .enumerate()
        .filter(|&(i, _)| remaining[i])
        .map(|(_, &n)| n)
        .collect();

    (layers, cyclic)
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

    #[test]
    fn should_render_disconnected_components_separately() {
        let d = DagDiagram {
            title: None, nodes: None,
            edges: vec![
                Edge { from: "A".into(), to: "B".into(), label: None },
                Edge { from: "C".into(), to: "D".into(), label: None },
                Edge { from: "E".into(), to: "F".into(), label: None },
            ],
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        // Each component should be rendered separately — A and C should NOT be in the same box
        let has_a_c_together = lines.iter().any(|l| l.contains('A') && l.contains('C'));
        assert!(!has_a_c_together, "A and C should not be in the same box");
        // Each source should appear with its own sink
        assert!(lines.iter().any(|l| l.contains('A')));
        assert!(lines.iter().any(|l| l.contains('B')));
        assert!(lines.iter().any(|l| l.contains('C')));
        assert!(lines.iter().any(|l| l.contains('D')));
        assert!(lines.iter().any(|l| l.contains('E')));
        assert!(lines.iter().any(|l| l.contains('F')));
        for l in &lines { assert_eq!(display_width(l), 40); }
    }
}
