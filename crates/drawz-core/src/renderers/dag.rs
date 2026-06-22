//! DAG (directed acyclic graph) renderer — uses ascii-dag for layout, custom rendering.

use std::collections::HashMap;

use ascii_dag::Graph;

use crate::measure::{display_width, pad_right};
use crate::result::RenderContext;
use crate::schema::DagDiagram;

/// Horizontal spacing between boxes on the same level.
const BOX_SPACING: usize = 3;

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

    // Build ordered node IDs (preserving first-seen order)
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
        for id in [e.from.as_str(), e.to.as_str()] {
            if !node_ids.contains(&id) {
                node_ids.push(id);
            }
        }
    }

    // Map node ID → index for O(1) lookup
    let id_to_idx: HashMap<&str, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

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
            let from = *id_to_idx.get(e.from.as_str())?;
            let to = *id_to_idx.get(e.to.as_str())?;
            (from != to).then_some((from, to))
        })
        .collect();

    let dag = Graph::from_edges(&nodes_with_ids, &edges_with_ids);

    if dag.has_cycle() {
        return Err("cycle detected in dag".to_string());
    }

    // Group nodes by level using Sugiyama layout
    let ir = dag.compute_layout();
    let level_count = ir.level_count();
    let mut levels: Vec<Vec<&str>> = vec![Vec::new(); level_count];
    for node in ir.nodes() {
        levels[node.level].push(node.label);
    }

    // Build label → (level, position) index for O(1) edge matching
    let mut label_pos: HashMap<&str, (usize, usize)> = HashMap::new();
    for (level_idx, level) in levels.iter().enumerate() {
        for (pos, &label) in level.iter().enumerate() {
            label_pos.insert(label, (level_idx, pos));
        }
    }

    // Pre-compute level widths and centering offsets
    let level_widths: Vec<usize> = levels.iter().map(|level| level_width(level)).collect();
    let max_level_w = level_widths.iter().copied().max().unwrap_or(0);
    let offsets: Vec<usize> = level_widths
        .iter()
        .map(|&w| max_level_w.saturating_sub(w) / 2)
        .collect();

    // Render
    let mut lines = Vec::new();
    let mut level_line_starts: Vec<usize> = Vec::new(); // output line index where each level starts

    for (level_idx, level) in levels.iter().enumerate() {
        if level.is_empty() {
            continue;
        }

        level_line_starts.push(lines.len());
        render_level(level, offsets[level_idx], &mut lines);

        if level_idx >= level_count - 1 {
            continue;
        }

        let next_idx = level_idx + 1;
        let next_level = &levels[next_idx];
        if next_level.is_empty() {
            let center = offsets[level_idx] + level_widths[level_idx] / 2;
            lines.push(char_row(center, '│', ctx.inner_width));
            lines.push(char_row(center, '▼', ctx.inner_width));
            continue;
        }

        let cur_centers = box_centers(level, offsets[level_idx]);
        let next_centers = box_centers(next_level, offsets[next_idx]);

        // Resolve edges between these two levels via pre-built index
        let level_edges: Vec<(usize, usize)> = edges_with_ids
            .iter()
            .filter_map(|&(from_id, to_id)| {
                let from_label = nodes_with_ids[from_id].1;
                let to_label = nodes_with_ids[to_id].1;
                let &(fl, fp) = label_pos.get(from_label)?;
                let &(tl, tp) = label_pos.get(to_label)?;
                (fl == level_idx && tl == next_idx).then_some((fp, tp))
            })
            .collect();

        render_arrows(
            &cur_centers,
            &next_centers,
            &level_edges,
            ctx.inner_width,
            &mut lines,
        );
    }

    if lines.is_empty() {
        return Err("dag has no renderable content".to_string());
    }

    // Post-process: wrap subgraph regions with labeled frames
    if let Some(subgraphs) = &diagram.subgraphs {
        let id_to_label: HashMap<&str, &str> = node_ids
            .iter()
            .zip(nodes_with_ids.iter())
            .map(|(&id, &(_, lbl))| (id, lbl))
            .collect();
        lines = frame_subgraphs(
            &lines,
            subgraphs,
            &label_pos,
            &level_line_starts,
            &id_to_label,
        );
    }

    Ok(lines)
}

// --- Helpers ---

/// Total display width of a level's boxes including spacing.
fn level_width(labels: &[&str]) -> usize {
    if labels.is_empty() {
        return 0;
    }
    let box_widths: usize = labels.iter().map(|l| display_width(l) + 4).sum();
    box_widths + (labels.len() - 1) * BOX_SPACING
}

/// Compute center x-position of each box given an offset.
fn box_centers(labels: &[&str], offset: usize) -> Vec<usize> {
    let mut centers = Vec::with_capacity(labels.len());
    let mut x = offset;
    for &label in labels {
        let w = display_width(label) + 4;
        centers.push(x + w / 2);
        x += w + BOX_SPACING;
    }
    centers
}

/// Create a row with a single character at position `x`, rest spaces.
fn char_row(x: usize, ch: char, width: usize) -> String {
    let mut row = vec![' '; width];
    if x < width {
        row[x] = ch;
    }
    row.into_iter().collect()
}

/// Collect unique sorted values from an iterator.
fn sorted_unique(iter: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut v: Vec<usize> = iter.collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Render boxes for one level at the given x-offset.
fn render_level(labels: &[&str], offset: usize, out: &mut Vec<String>) {
    let widths: Vec<usize> = labels.iter().map(|l| display_width(l) + 4).collect();
    let sep = " ".repeat(BOX_SPACING);
    let prefix = " ".repeat(offset);

    let top: String = widths
        .iter()
        .map(|&w| format!("┌{}┐", "─".repeat(w - 2)))
        .collect::<Vec<_>>()
        .join(&sep);
    let mid: String = labels
        .iter()
        .zip(&widths)
        .map(|(&l, &w)| format!("│ {} │", pad_right(l, w - 4)))
        .collect::<Vec<_>>()
        .join(&sep);
    let bot: String = widths
        .iter()
        .map(|&w| format!("└{}┘", "─".repeat(w - 2)))
        .collect::<Vec<_>>()
        .join(&sep);

    out.push(format!("{prefix}{top}"));
    out.push(format!("{prefix}{mid}"));
    out.push(format!("{prefix}{bot}"));
}

/// Render arrows between two levels based on actual edge topology.
fn render_arrows(
    src_centers: &[usize],
    dst_centers: &[usize],
    edges: &[(usize, usize)],
    width: usize,
    out: &mut Vec<String>,
) {
    if edges.is_empty() {
        out.push(pad_right("  │", width));
        out.push(pad_right("  ▼", width));
        return;
    }

    let src_xs = sorted_unique(edges.iter().map(|&(s, _)| src_centers[s]));
    let dst_xs = sorted_unique(edges.iter().map(|&(_, d)| dst_centers[d]));

    // Straight-down case: sources and destinations align, or single 1:1 connection
    if src_xs == dst_xs || (src_xs.len() == 1 && dst_xs.len() == 1) {
        out.push(multi_char_row(&src_xs, '│', width));
        out.push(multi_char_row(&src_xs, '▼', width)); // arrow at source center
        return;
    }

    // Complex case: horizontal connector needed
    out.push(multi_char_row(&src_xs, '│', width));

    // Connector line spanning all involved positions
    let all_xs = sorted_unique(src_xs.iter().chain(dst_xs.iter()).copied());
    let left = all_xs[0];
    let right = *all_xs.last().unwrap();

    let mut row = vec![' '; width];
    let fill_end = right.min(width - 1);
    for ch in &mut row[left..=fill_end] {
        *ch = '─';
    }

    for &x in &all_xs {
        if x >= width {
            continue;
        }
        row[x] = junction_char(
            src_xs.contains(&x),
            dst_xs.contains(&x),
            x > left,
            x < right,
        );
    }
    out.push(row.into_iter().collect());

    out.push(multi_char_row(&dst_xs, '▼', width));
}

/// Place a character at multiple x-positions in a row.
fn multi_char_row(positions: &[usize], ch: char, width: usize) -> String {
    let mut row = vec![' '; width];
    for &x in positions {
        if x < width {
            row[x] = ch;
        }
    }
    row.into_iter().collect()
}

/// Select the correct box-drawing junction character.
fn junction_char(from_above: bool, to_below: bool, has_left: bool, has_right: bool) -> char {
    match (from_above, to_below, has_left, has_right) {
        (true, true, true, true) => '┼',
        (true, true, true, false) => '┤',
        (true, true, false, true) => '├',
        (true, true, false, false) => '│',
        (true, false, true, true) => '┴',
        (true, false, true, false) => '┘',
        (true, false, false, true) => '└',
        (true, false, false, false) => '│',
        (false, true, true, true) => '┬',
        (false, true, true, false) => '┐',
        (false, true, false, true) => '┌',
        (false, true, false, false) => '│',
        _ => '─',
    }
}

fn get_label<'a>(id: &'a str, diagram: &'a DagDiagram) -> &'a str {
    diagram
        .nodes
        .as_ref()
        .and_then(|nodes| {
            nodes
                .iter()
                .find(|n| n.id.as_deref().unwrap_or(&n.label) == id)
        })
        .map_or(id, |n| &n.label)
}

/// Output lines per level (top border + label + bottom border of box).
const LINES_PER_LEVEL: usize = 3;

/// A resolved subgraph's position in the rendered output.
struct SubgraphRegion<'a> {
    start_line: usize,
    end_line: usize,
    label: &'a str,
}

/// Wrap contiguous level regions belonging to subgraphs with labeled borders.
fn frame_subgraphs(
    lines: &[String],
    subgraphs: &[crate::schema::Subgraph],
    label_pos: &HashMap<&str, (usize, usize)>,
    level_line_starts: &[usize],
    id_to_label: &HashMap<&str, &str>,
) -> Vec<String> {
    let regions = resolve_subgraph_regions(
        subgraphs,
        label_pos,
        level_line_starts,
        id_to_label,
        lines.len(),
    );
    if regions.is_empty() {
        return lines.to_vec();
    }

    let mut result = Vec::with_capacity(lines.len() + regions.len() * 2);
    let mut cursor = 0;

    for region in &regions {
        if region.start_line < cursor {
            continue; // overlapping region, skip
        }

        // Emit unframed lines before this region
        result.extend_from_slice(&lines[cursor..region.start_line]);

        // Frame the region
        let content = &lines[region.start_line..region.end_line];
        frame_region(region.label, content, &mut result);

        cursor = region.end_line;
    }

    // Emit remaining lines after last region
    result.extend_from_slice(&lines[cursor..]);
    result
}

/// Resolve which output line ranges each subgraph occupies.
fn resolve_subgraph_regions<'a>(
    subgraphs: &'a [crate::schema::Subgraph],
    label_pos: &HashMap<&str, (usize, usize)>,
    level_line_starts: &[usize],
    id_to_label: &HashMap<&str, &str>,
    total_lines: usize,
) -> Vec<SubgraphRegion<'a>> {
    let mut regions: Vec<SubgraphRegion<'a>> = subgraphs
        .iter()
        .filter_map(|sg| {
            let (min_level, max_level) =
                sg.node_ids
                    .iter()
                    .fold((usize::MAX, 0usize), |(min, max), node_id| {
                        let label = id_to_label
                            .get(node_id.as_str())
                            .copied()
                            .unwrap_or(node_id.as_str());
                        label_pos
                            .get(label)
                            .map_or((min, max), |&(level, _)| (min.min(level), max.max(level)))
                    });
            if min_level == usize::MAX {
                return None;
            }

            let start_line = *level_line_starts.get(min_level)?;
            let end_line = level_line_starts
                .get(max_level)
                .map(|&s| s + LINES_PER_LEVEL)
                .unwrap_or(total_lines);

            Some(SubgraphRegion {
                start_line,
                end_line,
                label: &sg.label,
            })
        })
        .collect();

    regions.sort_by_key(|r| r.start_line);
    regions
}

/// Wrap a slice of content lines with a labeled border.
fn frame_region(label: &str, content: &[String], out: &mut Vec<String>) {
    let content_w = content
        .iter()
        .map(|l| display_width(l.trim_end()))
        .max()
        .unwrap_or(0);
    let label_w = display_width(label);
    let inner_w = content_w.max(label_w + 2);
    let frame_w = inner_w + 4; // "│ " + content + " │"

    // Top: ┌─ Label ──...──┐
    let dashes = frame_w.saturating_sub(label_w + 5);
    out.push(format!("┌─ {} {}┐", label, "─".repeat(dashes)));

    // Content rows
    for line in content {
        out.push(format!("│ {} │", pad_right(line.trim_end(), inner_w)));
    }

    // Bottom: └──...──┘
    out.push(format!("└{}┘", "─".repeat(frame_w - 2)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::RenderContext;
    use crate::schema::{DagDiagram, Edge, Node};

    fn ctx(width: usize) -> RenderContext {
        RenderContext {
            inner_width: width,
            total_width: u16::try_from(width).unwrap(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn should_render_layers_when_edges_provided() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                },
                Edge {
                    from: "A".into(),
                    to: "C".into(),
                    label: None,
                },
                Edge {
                    from: "B".into(),
                    to: "D".into(),
                    label: None,
                },
                Edge {
                    from: "C".into(),
                    to: "D".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        assert!(lines.iter().any(|l| l.contains('A')));
        assert!(lines.iter().any(|l| l.contains('D')));
    }

    #[test]
    fn should_return_error_when_no_edges_or_nodes() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![],
            subgraphs: None,
        };
        assert!(render(&d, &mut ctx(40)).is_err());
    }

    #[test]
    fn should_use_node_labels_when_provided() {
        let d = DagDiagram {
            title: None,
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
            edges: vec![Edge {
                from: "a".into(),
                to: "b".into(),
                label: None,
            }],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Start")));
        assert!(lines.iter().any(|l| l.contains("End")));
    }

    #[test]
    fn should_render_diamond_pattern() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                },
                Edge {
                    from: "A".into(),
                    to: "C".into(),
                    label: None,
                },
                Edge {
                    from: "B".into(),
                    to: "D".into(),
                    label: None,
                },
                Edge {
                    from: "C".into(),
                    to: "D".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        let has_bc_same_line = lines.iter().any(|l| l.contains('B') && l.contains('C'));
        assert!(has_bc_same_line, "B and C should be in same layer");
    }

    #[test]
    fn should_render_converging_arrows_when_fan_in() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "X".into(),
                    to: "Z".into(),
                    label: None,
                },
                Edge {
                    from: "Y".into(),
                    to: "Z".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        let has_merge = lines
            .iter()
            .any(|l| l.contains('┘') || l.contains('┴') || l.contains('└') || l.contains('┬'));
        assert!(has_merge, "fan-in should show converging arrows");
        assert!(lines.iter().any(|l| l.contains('▼')));
    }

    #[test]
    fn should_render_diverging_arrows_when_fan_out() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "Z".into(),
                    to: "X".into(),
                    label: None,
                },
                Edge {
                    from: "Z".into(),
                    to: "Y".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        let has_split = lines
            .iter()
            .any(|l| l.contains('┬') || l.contains('┌') || l.contains('┐'));
        assert!(has_split, "fan-out should show diverging arrows");
        let arrow_count: usize = lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == '▼').count())
            .sum();
        assert!(
            arrow_count >= 2,
            "fan-out should have 2+ arrows: got {arrow_count}"
        );
    }

    // --- Helper function tests ---

    #[test]
    fn level_width_single_node() {
        assert_eq!(level_width(&["A"]), 5); // "A" + 4 = 5
    }

    #[test]
    fn level_width_multiple_nodes() {
        // "A"(5) + spacing(3) + "B"(5) = 13
        assert_eq!(level_width(&["A", "B"]), 13);
    }

    #[test]
    fn level_width_empty() {
        assert_eq!(level_width(&[]), 0);
    }

    #[test]
    fn box_centers_single() {
        let c = box_centers(&["Hello"], 0);
        // box width = 9, center = 4
        assert_eq!(c, vec![4]);
    }

    #[test]
    fn box_centers_with_offset() {
        let c = box_centers(&["A"], 10);
        // box width = 5, center = 10 + 2 = 12
        assert_eq!(c, vec![12]);
    }

    #[test]
    fn box_centers_multiple() {
        let c = box_centers(&["A", "B"], 0);
        // A: w=5, center=2. B: x=5+3=8, w=5, center=10
        assert_eq!(c, vec![2, 10]);
    }

    #[test]
    fn junction_char_passthrough() {
        assert_eq!(junction_char(true, true, true, true), '┼');
    }

    #[test]
    fn junction_char_fan_in_left_end() {
        assert_eq!(junction_char(true, false, false, true), '└');
    }

    #[test]
    fn junction_char_fan_in_right_end() {
        assert_eq!(junction_char(true, false, true, false), '┘');
    }

    #[test]
    fn junction_char_fan_out_split() {
        assert_eq!(junction_char(false, true, true, true), '┬');
    }

    #[test]
    fn sorted_unique_deduplicates() {
        let v = sorted_unique([3, 1, 2, 1, 3].into_iter());
        assert_eq!(v, vec![1, 2, 3]);
    }

    // --- Arrow rendering edge cases ---

    #[test]
    fn should_render_straight_arrows_when_linear_chain() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                },
                Edge {
                    from: "B".into(),
                    to: "C".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        // No merge/split characters in a linear chain
        let has_junction = lines
            .iter()
            .any(|l| l.contains('┬') || l.contains('┴') || l.contains('┼'));
        assert!(!has_junction, "linear chain should have no junction chars");
        // Should have straight vertical arrows
        assert!(lines.iter().any(|l| l.contains('│')));
        assert!(lines.iter().any(|l| l.contains('▼')));
    }

    #[test]
    fn should_center_single_node_levels() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                },
                Edge {
                    from: "A".into(),
                    to: "C".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        // "A" should be centered (indented), not left-aligned
        let a_line = lines.iter().find(|l| l.contains('A')).unwrap();
        assert!(
            a_line.starts_with(' '),
            "single-node level should be centered"
        );
    }

    #[test]
    fn should_handle_wide_label_difference_between_levels() {
        let d = DagDiagram {
            title: None,
            nodes: Some(vec![
                Node {
                    id: Some("a".into()),
                    label: "Short".into(),
                },
                Node {
                    id: Some("b".into()),
                    label: "A Very Long Node Label".into(),
                },
            ]),
            edges: vec![Edge {
                from: "a".into(),
                to: "b".into(),
                label: None,
            }],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Short")));
        assert!(lines.iter().any(|l| l.contains("A Very Long Node Label")));
    }

    #[test]
    fn should_render_three_to_one_fan_in_with_centered_target() {
        let d = DagDiagram {
            title: None,
            nodes: None,
            edges: vec![
                Edge {
                    from: "X".into(),
                    to: "T".into(),
                    label: None,
                },
                Edge {
                    from: "Y".into(),
                    to: "T".into(),
                    label: None,
                },
                Edge {
                    from: "Z".into(),
                    to: "T".into(),
                    label: None,
                },
            ],
            subgraphs: None,
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        // All three sources should be on the same line
        let has_xyz = lines
            .iter()
            .any(|l| l.contains('X') && l.contains('Y') && l.contains('Z'));
        assert!(has_xyz, "all sources should be on same level");
        // Target should be centered (indented)
        let t_line = lines
            .iter()
            .find(|l| l.contains('T') && !l.contains('X'))
            .unwrap();
        assert!(t_line.starts_with(' '), "fan-in target should be centered");
    }

    // --- Additional helper tests ---

    #[test]
    fn char_row_places_char_at_position() {
        let row = char_row(3, '│', 10);
        assert_eq!(row, "   │      ");
    }

    #[test]
    fn char_row_out_of_bounds_safe() {
        let row = char_row(100, '│', 5);
        assert_eq!(row, "     ");
    }

    #[test]
    fn multi_char_row_multiple_positions() {
        let row = multi_char_row(&[1, 5, 8], '▼', 10);
        assert_eq!(row, " ▼   ▼  ▼ ");
    }

    #[test]
    fn multi_char_row_empty_positions() {
        let row = multi_char_row(&[], '▼', 5);
        assert_eq!(row, "     ");
    }

    #[test]
    fn render_level_produces_three_lines() {
        let mut out = Vec::new();
        render_level(&["A", "B"], 0, &mut out);
        assert_eq!(out.len(), 3);
        assert!(out[0].contains('┌'));
        assert!(out[1].contains('A'));
        assert!(out[1].contains('B'));
        assert!(out[2].contains('└'));
    }

    #[test]
    fn render_level_with_offset_indents() {
        let mut out = Vec::new();
        render_level(&["X"], 5, &mut out);
        assert!(out[0].starts_with("     ┌"));
    }

    #[test]
    fn render_arrows_straight_when_aligned() {
        // src and dst at same position → straight │▼
        let mut out = Vec::new();
        render_arrows(&[5], &[5], &[(0, 0)], 20, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out[0].contains('│'));
        assert!(out[1].contains('▼'));
        assert!(!out[0].contains('─'), "should not have horizontal line");
    }

    #[test]
    fn render_arrows_straight_for_single_edge_misaligned() {
        // Single 1:1 connection with different centers → still straight
        let mut out = Vec::new();
        render_arrows(&[3], &[7], &[(0, 0)], 20, &mut out);
        assert_eq!(out.len(), 2); // no connector line
        assert!(out[0].contains('│'));
        assert!(out[1].contains('▼'));
    }

    #[test]
    fn render_arrows_fan_in_has_connector() {
        let mut out = Vec::new();
        render_arrows(&[2, 10], &[6], &[(0, 0), (1, 0)], 20, &mut out);
        assert_eq!(out.len(), 3); // │ line, connector, ▼ line
        assert!(out[1].contains('─'), "fan-in needs horizontal connector");
    }

    #[test]
    fn frame_region_wraps_content() {
        let content = vec!["hello".to_string(), "world".to_string()];
        let mut out = Vec::new();
        frame_region("Test", &content, &mut out);
        assert!(out[0].contains("Test"), "top border should have label");
        assert!(out[0].starts_with("┌─"));
        assert!(out[1].starts_with("│ "));
        assert!(out.last().unwrap().starts_with("└"));
        assert_eq!(out.len(), 4); // top + 2 content + bottom
    }

    #[test]
    fn should_render_subgraph_via_json_input() {
        let d = DagDiagram {
            title: None,
            nodes: Some(vec![
                Node {
                    id: Some("a".into()),
                    label: "X".into(),
                },
                Node {
                    id: Some("b".into()),
                    label: "Y".into(),
                },
            ]),
            edges: vec![Edge {
                from: "a".into(),
                to: "b".into(),
                label: None,
            }],
            subgraphs: Some(vec![crate::schema::Subgraph {
                label: "Group".into(),
                node_ids: vec!["a".into()],
            }]),
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        let output = lines.join("\n");
        assert!(output.contains("Group"), "should have subgraph frame");
        assert!(output.contains("X"), "should render node");
    }
}
