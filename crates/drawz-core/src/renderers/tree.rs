use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::{TreeDiagram, TreeNode};

/// Render tree with `├──` / `└──` connectors.
///
/// # Errors
///
/// Returns an error if neither `indent` nor `root` is provided.
pub fn render(diagram: &TreeDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    let lines = if let Some(indent) = &diagram.indent {
        render_indent(indent, ctx)
    } else if let Some(root) = &diagram.root {
        render_node(root, ctx)
    } else {
        return Err("tree requires 'indent' or 'root' field".to_string());
    };

    if lines.is_empty() {
        return Err("tree content is empty".to_string());
    }

    Ok(lines)
}

/// Parse indent-based text into tree lines with connectors.
fn render_indent(text: &str, ctx: &mut RenderContext) -> Vec<String> {
    let raw: Vec<&str> = text.lines().collect();
    if raw.is_empty() {
        return Vec::new();
    }

    let entries: Vec<(usize, &str)> = raw
        .iter()
        .map(|line| {
            let trimmed = line.trim_start();
            let spaces = line.len() - trimmed.len();
            (spaces / 2, trimmed)
        })
        .collect();

    let mut lines = Vec::new();
    render_indent_recursive(&entries, 0, entries.len(), "", ctx, &mut lines);
    lines
}

fn render_indent_recursive(
    entries: &[(usize, &str)],
    start: usize,
    end: usize,
    prefix: &str,
    ctx: &mut RenderContext,
    out: &mut Vec<String>,
) {
    if start >= end {
        return;
    }

    let base_level = entries[start].0;

    // Root of this subtree
    let line = format!("{prefix}{}", entries[start].1);
    out.push(fit_line(&line, ctx));

    // Collect direct children (entries at base_level + 1)
    let mut children: Vec<(usize, usize)> = Vec::new();
    let mut i = start + 1;
    while i < end {
        if entries[i].0 <= base_level {
            break;
        }
        if entries[i].0 == base_level + 1 {
            let cs = i;
            i += 1;
            while i < end && entries[i].0 > base_level + 1 {
                i += 1;
            }
            children.push((cs, i));
        } else {
            i += 1;
        }
    }

    for (idx, &(cs, ce)) in children.iter().enumerate() {
        let is_last = idx == children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let line = format!("{prefix}{connector}{}", entries[cs].1);
        out.push(fit_line(&line, ctx));

        let nested = format!("{prefix}{child_prefix}");
        render_indent_recursive(entries, cs + 1, ce, &nested, ctx, out);
    }
}

/// Render structured `TreeNode` recursively.
fn render_node(root: &TreeNode, ctx: &mut RenderContext) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(fit_line(&root.label, ctx));
    render_children(&root.children, "", ctx, &mut lines);
    lines
}

fn render_children(children: &[TreeNode], prefix: &str, ctx: &mut RenderContext, out: &mut Vec<String>) {
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let line = format!("{prefix}{connector}{}", child.label);
        out.push(fit_line(&line, ctx));

        let nested = format!("{prefix}{child_prefix}");
        render_children(&child.children, &nested, ctx, out);
    }
}

fn fit_line(line: &str, ctx: &mut RenderContext) -> String {
    let w = display_width(line);
    if w > ctx.inner_width {
        ctx.warnings.push("suggestion: some labels truncated to fit width".to_string());
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
    use crate::schema::{TreeDiagram, TreeNode};

    fn ctx(width: usize) -> RenderContext {
        RenderContext { inner_width: width, total_width: u16::try_from(width).unwrap(), warnings: Vec::new() }
    }

    #[test]
    fn should_render_connectors_when_indent_provided() {
        let d = TreeDiagram { title: None, root: None, indent: Some("root\n  a\n  b".into()) };
        let lines = render(&d, &mut ctx(40)).unwrap();
        assert!(lines.iter().any(|l| l.contains("├──")));
        assert!(lines.iter().any(|l| l.contains("└──")));
        for l in &lines { assert_eq!(display_width(l), 40); }
    }

    #[test]
    fn should_render_hierarchy_when_tree_node_provided() {
        let d = TreeDiagram {
            title: None,
            indent: None,
            root: Some(TreeNode {
                label: "r".into(),
                children: vec![
                    TreeNode { label: "a".into(), children: vec![] },
                    TreeNode { label: "b".into(), children: vec![
                        TreeNode { label: "c".into(), children: vec![] },
                    ] },
                ],
            }),
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert_eq!(lines[0].trim_end(), "r");
        assert!(lines.iter().any(|l| l.contains("└── c")));
        for l in &lines { assert_eq!(display_width(l), 30); }
    }

    #[test]
    fn should_return_error_when_no_indent_or_root() {
        let d = TreeDiagram { title: None, root: None, indent: None };
        assert!(render(&d, &mut ctx(40)).is_err());
    }

    #[test]
    fn should_truncate_and_warn_when_labels_exceed_width() {
        let d = TreeDiagram { title: None, root: None, indent: Some("root\n  a_very_long_label_that_exceeds_width".into()) };
        let mut c = ctx(15);
        let lines = render(&d, &mut c).unwrap();
        for l in &lines { assert_eq!(display_width(l), 15); }
        assert!(!c.warnings.is_empty());
    }
}
