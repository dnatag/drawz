//! Component architecture diagram renderer.
//!
//! Renders groups of nodes side-by-side with labeled connections between them.
//! Connected nodes are vertically aligned so arrows route straight across.
//! Nodes within each group stack vertically inside a labeled frame.

use crate::measure::{display_width, pad_right};
use crate::result::RenderContext;
use crate::schema::ComponentDiagram;

/// Spacing for the arrow column between groups.
const ARROW_COL_MIN: usize = 5;

/// Render a component diagram with side-by-side groups and labeled connections.
///
/// # Errors
///
/// Returns an error if groups are empty.
pub(crate) fn render(
    diagram: &ComponentDiagram,
    ctx: &mut RenderContext,
) -> Result<Vec<String>, String> {
    if diagram.groups.is_empty() {
        return Err("component diagram requires at least one group".to_string());
    }

    if diagram.groups.len() == 1 {
        let mut lines = Vec::new();
        render_standalone_group(&diagram.groups[0], &mut lines);
        return Ok(lines);
    }

    // Render pairs of groups side-by-side
    let mut lines = Vec::new();

    // If any group has chains, render all groups stacked (chains need more width)
    let has_chains = diagram.groups.iter().any(|g| !g.chains.is_empty());
    if has_chains {
        for (i, group) in diagram.groups.iter().enumerate() {
            render_standalone_group(group, &mut lines);
            if i < diagram.groups.len() - 1 {
                // Render inter-group connection if any
                let conn_label = diagram.connections.first().and_then(|c| c.label.as_deref());
                if let Some(label) = conn_label {
                    lines.push(format!("     │ {label}"));
                } else {
                    lines.push("     │".to_string());
                }
                lines.push("     ▼".to_string());
            }
        }
    } else {
        // Side-by-side mode for flat node groups
        let chunks: Vec<_> = diagram.groups.chunks(2).collect();
        let chunk_count = chunks.len();

        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            if chunk.len() == 2 {
                let left = &chunk[0];
                let right = &chunk[1];

                let left_nodes: Vec<&str> = left.nodes.iter().map(String::as_str).collect();
                let right_nodes: Vec<&str> = right.nodes.iter().map(String::as_str).collect();
                let connections: Vec<_> = diagram
                    .connections
                    .iter()
                    .filter_map(|c| {
                        let (from, to) = (c.from.as_str(), c.to.as_str());
                        let label = c.label.as_deref().unwrap_or("");
                        (left_nodes.contains(&from) && right_nodes.contains(&to))
                            .then_some((from, to, label))
                    })
                    .collect();

                render_pair(left, right, &connections, &mut lines);
            } else {
                render_standalone_group(&chunk[0], &mut lines);
            }

            // Vertical connector between chunks
            if chunk_idx < chunk_count - 1 {
                lines.push("  │".to_string());
                lines.push("  ▼".to_string());
            }
        }
    } // end else (side-by-side mode)

    // Pad all lines to inner_width
    let max_w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
    let target_w = max_w.min(ctx.inner_width);
    Ok(lines.iter().map(|l| pad_right(l, target_w)).collect())
}

/// Render two groups side-by-side. Connected nodes share a row so arrows go straight.
fn render_pair(
    left: &crate::schema::ComponentGroup,
    right: &crate::schema::ComponentGroup,
    connections: &[(&str, &str, &str)],
    out: &mut Vec<String>,
) {
    // Frame dimensions
    let left_inner = compute_inner_width(&left.label, &left.nodes);
    let right_inner = compute_inner_width(&right.label, &right.nodes);
    let left_frame_w = left_inner + 4;
    let right_frame_w = right_inner + 4;

    // Arrow column width
    let max_label_w = connections
        .iter()
        .map(|&(_, _, lbl)| display_width(lbl))
        .max()
        .unwrap_or(0);
    let arrow_col_w = (max_label_w + 4).max(ARROW_COL_MIN);

    // Build slots: each slot has (Option<left_node>, Option<right_node>, arrow_label)
    let slots = build_slots(left, right, connections);

    // Top borders
    let left_top = frame_top(&left.label, left_frame_w);
    let right_top = frame_top(&right.label, right_frame_w);
    out.push(format!(
        "{} {} {}",
        left_top,
        " ".repeat(arrow_col_w),
        right_top
    ));

    // Render each slot as 3 lines (box_top, box_mid+arrow, box_bot)
    for (slot_idx, (ln, rn, label)) in slots.iter().enumerate() {
        for row in 0..3 {
            let left_cell = node_cell(*ln, row);
            let right_cell = node_cell(*rn, row);

            let arrow = if row == 1 && !label.is_empty() {
                format_arrow(label, arrow_col_w)
            } else if row == 1 && ln.is_some() && rn.is_some() && label.is_empty() {
                // Connected pair without explicit label in this slot
                // Check if there's actually a connection for this pair
                let has_conn = connections
                    .iter()
                    .any(|&(f, t, _)| ln.is_some_and(|l| l == f) && rn.is_some_and(|r| r == t));
                if has_conn {
                    format!("{}→", "─".repeat(arrow_col_w - 1))
                } else {
                    " ".repeat(arrow_col_w)
                }
            } else {
                " ".repeat(arrow_col_w)
            };

            out.push(format!(
                "│ {} │ {} │ {} │",
                pad_right(&left_cell, left_inner),
                pad_right(&arrow, arrow_col_w),
                pad_right(&right_cell, right_inner),
            ));
        }

        // Spacer between slots
        if slot_idx < slots.len() - 1 {
            out.push(format!(
                "│ {} │ {} │ {} │",
                " ".repeat(left_inner),
                " ".repeat(arrow_col_w),
                " ".repeat(right_inner),
            ));
        }
    }

    // Bottom borders
    let left_bot = format!("└{}┘", "─".repeat(left_frame_w - 2));
    let right_bot = format!("└{}┘", "─".repeat(right_frame_w - 2));
    out.push(format!(
        "{} {} {}",
        left_bot,
        " ".repeat(arrow_col_w),
        right_bot
    ));
}

/// Determine slot assignment: connected pairs first, then remaining nodes.
fn build_slots<'a>(
    left: &'a crate::schema::ComponentGroup,
    right: &'a crate::schema::ComponentGroup,
    connections: &[(&'a str, &'a str, &'a str)],
) -> Vec<(Option<&'a str>, Option<&'a str>, &'a str)> {
    let mut slots: Vec<(Option<&str>, Option<&str>, &str)> = Vec::new();
    let mut left_used = vec![false; left.nodes.len()];
    let mut right_used = vec![false; right.nodes.len()];

    // Connected pairs get their own aligned slots
    for &(from, to, label) in connections {
        if let Some(li) = left.nodes.iter().position(|n| n == from) {
            if let Some(ri) = right.nodes.iter().position(|n| n == to) {
                if !left_used[li] && !right_used[ri] {
                    left_used[li] = true;
                    right_used[ri] = true;
                    slots.push((Some(from), Some(to), label));
                }
            }
        }
    }

    // Remaining unconnected nodes: pair leftovers or add solo
    let mut left_remaining = left
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| !left_used[*i])
        .map(|(_, n)| n.as_str());
    let mut right_remaining = right
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| !right_used[*i])
        .map(|(_, n)| n.as_str());

    loop {
        let l = left_remaining.next();
        let r = right_remaining.next();
        if l.is_none() && r.is_none() {
            break;
        }
        slots.push((l, r, ""));
    }

    slots
}

/// Compute inner width for a group frame (max of node boxes and label).
fn compute_inner_width(label: &str, nodes: &[String]) -> usize {
    let nodes_w = nodes
        .iter()
        .map(|n| display_width(n) + 4)
        .max()
        .unwrap_or(4);
    let label_w = display_width(label) + 2;
    nodes_w.max(label_w)
}

/// Render one row of a node box (0=top, 1=mid, 2=bot), or empty space.
fn node_cell(node: Option<&str>, row: usize) -> String {
    match node {
        Some(name) => {
            let w = display_width(name) + 4;
            match row {
                0 => format!("┌{}┐", "─".repeat(w - 2)),
                1 => format!("│ {name} │"),
                2 => format!("└{}┘", "─".repeat(w - 2)),
                _ => String::new(),
            }
        }
        None => String::new(),
    }
}

/// Format a labeled arrow for the arrow column.
fn format_arrow(label: &str, width: usize) -> String {
    let lw = display_width(label);
    let dashes = width.saturating_sub(lw + 3); // "─" + label + dashes + "→"
    format!("─{}{}→", label, "─".repeat(dashes))
}

/// Top border with label: ┌─ Label ──┐
fn frame_top(label: &str, frame_w: usize) -> String {
    let lw = display_width(label);
    let dashes = frame_w.saturating_sub(lw + 5);
    format!("┌─ {} {}┐", label, "─".repeat(dashes))
}

/// Render a standalone group (no paired partner).
fn render_standalone_group(group: &crate::schema::ComponentGroup, out: &mut Vec<String>) {
    if !group.chains.is_empty() {
        render_group_with_chains(group, out);
    } else {
        render_group_flat_nodes(group, out);
    }
}

/// Render a group with chains: each chain as a horizontal A → B → C row.
fn render_group_with_chains(group: &crate::schema::ComponentGroup, out: &mut Vec<String>) {
    let arrow = " → ";
    let arrow_w = display_width(arrow);

    // Compute content width: widest chain
    let max_chain_w = group
        .chains
        .iter()
        .map(|chain| {
            let nodes_w: usize = chain.iter().map(|n| display_width(n)).sum();
            nodes_w + chain.len().saturating_sub(1) * arrow_w
        })
        .max()
        .unwrap_or(0);
    let label_w = display_width(&group.label) + 2;
    let inner_w = max_chain_w.max(label_w);
    let frame_w = inner_w + 4;

    // Top border
    out.push(frame_top(&group.label, frame_w));

    // Render each chain as a row
    for (chain_idx, chain) in group.chains.iter().enumerate() {
        let chain_str: String = chain.join(arrow);
        out.push(format!("│ {} │", pad_right(&chain_str, inner_w)));

        // Find cross-chain edges that originate from nodes in this chain
        // and render vertical connectors
        let has_downward = group.edges.iter().any(|e| {
            chain.contains(&e.from)
                && group
                    .chains
                    .get(chain_idx + 1..)
                    .is_some_and(|later| later.iter().any(|c| c.contains(&e.to)))
        });

        if has_downward && chain_idx < group.chains.len() - 1 {
            // Find the node that has a downward edge and its position
            for edge in &group.edges {
                if !chain.contains(&edge.from) {
                    continue;
                }
                // Compute the x-position of the source node in this chain
                let node_x = chain
                    .iter()
                    .take_while(|n| n.as_str() != edge.from)
                    .map(|n| display_width(n) + arrow_w)
                    .sum::<usize>();
                let node_center = node_x + display_width(&edge.from) / 2;

                // Render connector: spaces + │ + optional label
                let label = edge.label.as_deref().unwrap_or("");
                let mut line = vec![' '; inner_w];
                if node_center < inner_w {
                    if label.is_empty() {
                        line[node_center] = '│';
                    } else {
                        // ├─label──→ starting at node_center
                        let annotation: Vec<char> = format!("├─{}─→", label).chars().collect();
                        for (j, &ch) in annotation.iter().enumerate() {
                            if node_center + j < inner_w {
                                line[node_center + j] = ch;
                            }
                        }
                    }
                }
                out.push(format!("│ {} │", line.into_iter().collect::<String>()));
            }
        } else if chain_idx < group.chains.len() - 1 {
            // Empty spacer between chains
            out.push(format!("│ {} │", " ".repeat(inner_w)));
        }
    }

    // Bottom border
    out.push(format!("└{}┘", "─".repeat(frame_w - 2)));
}

/// Render a group with a flat node list (vertical stack, original behavior).
fn render_group_flat_nodes(group: &crate::schema::ComponentGroup, out: &mut Vec<String>) {
    let inner_w = compute_inner_width(&group.label, &group.nodes);
    let frame_w = inner_w + 4;

    out.push(frame_top(&group.label, frame_w));
    for (i, node) in group.nodes.iter().enumerate() {
        let w = display_width(node) + 4;
        out.push(format!(
            "│ {} │",
            pad_right(&format!("┌{}┐", "─".repeat(w - 2)), inner_w)
        ));
        out.push(format!(
            "│ {} │",
            pad_right(&format!("│ {node} │"), inner_w)
        ));
        out.push(format!(
            "│ {} │",
            pad_right(&format!("└{}┘", "─".repeat(w - 2)), inner_w)
        ));
        if i < group.nodes.len() - 1 {
            out.push(format!("│ {} │", " ".repeat(inner_w)));
        }
    }
    out.push(format!("└{}┘", "─".repeat(frame_w - 2)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::RenderContext;
    use crate::schema::{ComponentDiagram, ComponentGroup, Connection};

    fn ctx(width: usize) -> RenderContext {
        RenderContext {
            inner_width: width,
            total_width: u16::try_from(width).unwrap(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn should_return_error_when_no_groups() {
        let d = ComponentDiagram {
            title: None,
            groups: vec![],
            connections: vec![],
        };
        assert!(render(&d, &mut ctx(80)).is_err());
    }

    #[test]
    fn should_render_single_group() {
        let d = ComponentDiagram {
            title: None,
            groups: vec![ComponentGroup {
                label: "Service".into(),
                nodes: vec!["API".into(), "DB".into()],
                chains: vec![],
                edges: vec![],
            }],
            connections: vec![],
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        let output = lines.join("\n");
        assert!(output.contains("Service"));
        assert!(output.contains("API"));
        assert!(output.contains("DB"));
    }

    #[test]
    fn should_render_two_groups_side_by_side_with_arrow() {
        let d = ComponentDiagram {
            title: None,
            groups: vec![
                ComponentGroup {
                    label: "Frontend".into(),
                    nodes: vec!["App".into()],
                    chains: vec![],
                    edges: vec![],
                },
                ComponentGroup {
                    label: "Backend".into(),
                    nodes: vec!["API".into()],
                    chains: vec![],
                    edges: vec![],
                },
            ],
            connections: vec![Connection {
                from: "App".into(),
                to: "API".into(),
                label: Some("REST".into()),
            }],
        };
        let lines = render(&d, &mut ctx(60)).unwrap();
        let output = lines.join("\n");
        assert!(output.contains("Frontend"));
        assert!(output.contains("Backend"));
        assert!(output.contains("REST"));
        assert!(output.contains("→"));
    }

    #[test]
    fn should_align_connected_nodes_on_same_row() {
        let d = ComponentDiagram {
            title: None,
            groups: vec![
                ComponentGroup {
                    label: "Left".into(),
                    nodes: vec!["A".into(), "B".into()],
                    chains: vec![],
                    edges: vec![],
                },
                ComponentGroup {
                    label: "Right".into(),
                    nodes: vec!["X".into(), "Y".into()],
                    chains: vec![],
                    edges: vec![],
                },
            ],
            connections: vec![
                Connection {
                    from: "A".into(),
                    to: "X".into(),
                    label: Some("one".into()),
                },
                Connection {
                    from: "B".into(),
                    to: "Y".into(),
                    label: Some("two".into()),
                },
            ],
        };
        let lines = render(&d, &mut ctx(60)).unwrap();
        // Each connection should have its arrow on the same line as both nodes
        let one_line = lines.iter().find(|l| l.contains("one")).unwrap();
        assert!(
            one_line.contains("A") && one_line.contains("X"),
            "A and X should be on same row as 'one'"
        );
        let two_line = lines.iter().find(|l| l.contains("two")).unwrap();
        assert!(
            two_line.contains("B") && two_line.contains("Y"),
            "B and Y should be on same row as 'two'"
        );
    }

    #[test]
    fn should_render_multiple_connections_architecture() {
        let d = ComponentDiagram {
            title: None,
            groups: vec![
                ComponentGroup {
                    label: "Parent".into(),
                    nodes: vec!["Scheduler".into(), "Engine".into()],
                    chains: vec![],
                    edges: vec![],
                },
                ComponentGroup {
                    label: "Child".into(),
                    nodes: vec!["Sandbox".into(), "Runtime".into()],
                    chains: vec![],
                    edges: vec![],
                },
            ],
            connections: vec![
                Connection {
                    from: "Scheduler".into(),
                    to: "Sandbox".into(),
                    label: Some("spawn".into()),
                },
                Connection {
                    from: "Engine".into(),
                    to: "Runtime".into(),
                    label: Some("pipe".into()),
                },
            ],
        };
        let lines = render(&d, &mut ctx(70)).unwrap();
        let output = lines.join("\n");
        assert!(output.contains("spawn"));
        assert!(output.contains("pipe"));
        assert!(output.contains("Scheduler"));
        assert!(output.contains("Runtime"));
    }
}
