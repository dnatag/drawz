//! DAG (directed acyclic graph) renderer — uses ascii-dag for layout, custom rendering.

use ascii_dag::Graph;

use crate::measure::{display_width, pad_right};
use crate::result::RenderContext;
use crate::schema::DagDiagram;

/// Render a DAG using ascii-dag's Sugiyama layout for layer assignment,
/// with our own clean box-and-arrow rendering style.
///
/// # Errors
///
/// Returns an error if edges are empty and no nodes provided, or if a cycle is detected.
pub(crate) fn render(diagram: &DagDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    if diagram.edges.is_empty() && diagram.nodes.is_none() {
        return Err("dag requires at least one edge or node".to_string());
    }

    // Build node ID → label mapping
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

    // Build ascii-dag graph for layout computation
    let nodes_with_ids: Vec<(usize, &str)> = node_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (i, get_label(id, diagram)))
        .collect();

    let edges_with_ids: Vec<(usize, usize)> = diagram
        .edges
        .iter()
        .filter_map(|e| {
            let from = node_ids.iter().position(|&n| n == e.from)?;
            let to = node_ids.iter().position(|&n| n == e.to)?;
            if from == to { return None; }
            Some((from, to))
        })
        .collect();

    let dag = Graph::from_edges(&nodes_with_ids, &edges_with_ids);

    if dag.has_cycle() {
        return Err("cycle detected in dag".to_string());
    }

    // Use layout IR to get level assignments
    let ir = dag.compute_layout();
    let level_count = ir.level_count();

    // Group nodes by level
    let mut levels: Vec<Vec<&str>> = vec![Vec::new(); level_count];
    for node in ir.nodes() {
        levels[node.level].push(node.label);
    }

    // Render each level with our own clean style
    let mut lines = Vec::new();
    for (level_idx, level) in levels.iter().enumerate() {
        if level.is_empty() {
            continue;
        }

        render_level(level, ctx, &mut lines);

        // Arrow between levels (not after last)
        if level_idx < level_count - 1 {
            lines.push(pad_right("  │", ctx.inner_width));
            lines.push(pad_right("  ▼", ctx.inner_width));
        }
    }

    if lines.is_empty() {
        return Err("dag has no renderable content".to_string());
    }

    Ok(lines)
}

fn render_level(labels: &[&str], ctx: &mut RenderContext, out: &mut Vec<String>) {
    if labels.len() == 1 {
        render_box(labels[0], ctx, out);
        return;
    }

    // Render all nodes in one row at natural size — no truncation
    let spacing = 3;
    let widths: Vec<usize> = labels.iter().map(|l| display_width(l) + 4).collect();
    let sep = " ".repeat(spacing);

    let top: String = widths.iter().map(|&w| format!("┌{}┐", "─".repeat(w - 2))).collect::<Vec<_>>().join(&sep);
    let mid: String = labels.iter().zip(&widths).map(|(&l, &w)| format!("│ {} │", pad_right(l, w - 4))).collect::<Vec<_>>().join(&sep);
    let bot: String = widths.iter().map(|&w| format!("└{}┘", "─".repeat(w - 2))).collect::<Vec<_>>().join(&sep);

    out.push(top);
    out.push(mid);
    out.push(bot);
}


fn render_box(label: &str, _ctx: &mut RenderContext, out: &mut Vec<String>) {
    let box_w = display_width(label) + 4;
    out.push(format!("┌{}┐", "─".repeat(box_w - 2)));
    out.push(format!("│ {label} │"));
    out.push(format!("└{}┘", "─".repeat(box_w - 2)));
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
        
    }

    #[test]
    fn should_render_diamond_pattern() {
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
        // B and C should appear on the same line (parallel)
        let has_bc_same_line = lines.iter().any(|l| l.contains('B') && l.contains('C'));
        assert!(has_bc_same_line, "B and C should be in same layer");
    }
}
