//! Sequence diagram renderer — actors, lifelines, and messages.

use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::SequenceDiagram;

/// Render a sequence diagram with actor columns and message arrows.
///
/// # Errors
///
/// Returns an error if actors list is empty.
pub(crate) fn render(diagram: &SequenceDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    if diagram.actors.is_empty() {
        return Err("sequence diagram requires at least one actor".to_string());
    }

    let num_actors = diagram.actors.len();
    if num_actors == 1 && diagram.messages.is_empty() {
        // Single actor, no messages — just show the actor
        let lines = vec![pad_right(&diagram.actors[0], ctx.inner_width)];
        return Ok(lines);
    }

    // Compute column positions
    let col_width = ctx.inner_width / num_actors;
    if col_width < 3 {
        return Err("sequence diagram too narrow for actors".to_string());
    }

    let mut lines = Vec::new();

    // Actor header row
    let header = render_actor_row(&diagram.actors, col_width, ctx);
    lines.push(header);

    // Lifeline separator
    let lifeline = render_lifeline_row(num_actors, col_width, ctx);
    lines.push(lifeline.clone());

    // Messages
    for msg in &diagram.messages {
        let from_idx = diagram.actors.iter().position(|a| *a == msg.from);
        let to_idx = diagram.actors.iter().position(|a| *a == msg.to);

        let (Some(from), Some(to)) = (from_idx, to_idx) else {
            ctx.warnings.push(format!(
                "skipping message: unknown actor in '{} -> {}'",
                msg.from, msg.to
            ));
            continue;
        };

        let msg_lines = render_message(from, to, &msg.label, num_actors, col_width, ctx);
        lines.extend(msg_lines);
        if from == to {
            let ret_line = render_self_return(from, num_actors, col_width, ctx);
            lines.push(ret_line);
        }
        lines.push(lifeline.clone());
    }

    Ok(lines)
}

fn render_actor_row(actors: &[String], col_width: usize, ctx: &mut RenderContext) -> String {
    let mut truncated = false;
    let row: String = actors
        .iter()
        .map(|a| {
            let max_w = col_width.saturating_sub(1);
            if display_width(a) > max_w {
                truncated = true;
                truncate(a, max_w)
            } else {
                pad_right(a, max_w)
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if truncated {
        ctx.warnings.push("some actor names truncated — use fewer actors or wider width".into());
    }
    pad_right(&row, ctx.inner_width)
}

fn render_lifeline_row(num_actors: usize, col_width: usize, ctx: &mut RenderContext) -> String {
    let mut row = String::new();
    for _ in 0..num_actors {
        let center = col_width / 2;
        for j in 0..col_width {
            if j == center {
                row.push('│');
            } else {
                row.push(' ');
            }
        }
    }
    pad_right(&row, ctx.inner_width)
}

fn render_message(
    from: usize,
    to: usize,
    label: &str,
    num_actors: usize,
    col_width: usize,
    ctx: &mut RenderContext,
) -> Vec<String> {
    let from_center = from * col_width + col_width / 2;
    let to_center = to * col_width + col_width / 2;

    let mut row = vec![' '; ctx.inner_width];

    // Place lifeline markers for actors not involved
    for i in 0..num_actors {
        let c = i * col_width + col_width / 2;
        if c < row.len() && i != from && i != to {
            row[c] = '│';
        }
    }

    if from == to {
        // Self-message: ├─┐ label
        let c = from_center;
        if c < row.len() { row[c] = '├'; }
        if c + 1 < row.len() { row[c + 1] = '─'; }
        if c + 2 < row.len() { row[c + 2] = '┐'; }
        let label_start = c + 4;
        let max_label = ctx.inner_width.saturating_sub(label_start);
        let fitted = if display_width(label) > max_label {
            truncate(label, max_label)
        } else {
            label.to_string()
        };
        for (j, ch) in fitted.chars().enumerate() {
            if label_start + j < row.len() {
                row[label_start + j] = ch;
            }
        }
        let s: String = row.into_iter().collect();
        return vec![pad_right(s.trim_end(), ctx.inner_width)];
    }

    let (left, right) = if from < to {
        (from_center, to_center)
    } else {
        (to_center, from_center)
    };
    let arrow_right = from < to;
    let max_label = right.saturating_sub(left).saturating_sub(2);
    let lw = display_width(label);

    // If label fits inline, render on the arrow line
    if lw <= max_label {
        // Draw arrow
        for c in left..=right { if c < row.len() { row[c] = '─'; } }
        if arrow_right {
            if right < row.len() { row[right] = '►'; }
            if left < row.len() { row[left] = '├'; }
        } else {
            if left < row.len() { row[left] = '◄'; }
            if right < row.len() { row[right] = '┤'; }
        }
        // Place label centered on arrow
        let mid = left + (right - left) / 2;
        let label_start = mid.saturating_sub(lw / 2);
        for (j, ch) in label.chars().enumerate() {
            let pos = label_start + j;
            if pos < row.len() && pos > left && pos < right {
                row[pos] = ch;
            }
        }
        let s: String = row.into_iter().collect();
        return vec![pad_right(s.trim_end(), ctx.inner_width)];
    }

    // Label doesn't fit inline — put it on a separate line above the arrow
    let mut label_row = vec![' '; ctx.inner_width];
    // Place lifelines on label row too
    for i in 0..num_actors {
        let c = i * col_width + col_width / 2;
        if c < label_row.len() {
            label_row[c] = '│';
        }
    }
    // Center label between the two actors
    let mid = left + (right - left) / 2;
    let label_start = mid.saturating_sub(lw / 2);
    for (j, ch) in label.chars().enumerate() {
        let pos = label_start + j;
        if pos < label_row.len() {
            label_row[pos] = ch;
        }
    }

    // Draw the arrow line (no label on it)
    for c in left..=right { if c < row.len() { row[c] = '─'; } }
    if arrow_right {
        if right < row.len() { row[right] = '►'; }
        if left < row.len() { row[left] = '├'; }
    } else {
        if left < row.len() { row[left] = '◄'; }
        if right < row.len() { row[right] = '┤'; }
    }

    let label_s: String = label_row.into_iter().collect();
    let arrow_s: String = row.into_iter().collect();
    vec![
        pad_right(label_s.trim_end(), ctx.inner_width),
        pad_right(arrow_s.trim_end(), ctx.inner_width),
    ]
}

fn render_self_return(from: usize, num_actors: usize, col_width: usize, ctx: &mut RenderContext) -> String {
    let from_center = from * col_width + col_width / 2;
    let mut row = vec![' '; ctx.inner_width];
    for i in 0..num_actors {
        let c = i * col_width + col_width / 2;
        if c < row.len() && i != from {
            row[c] = '│';
        }
    }
    let c = from_center;
    if c < row.len() {
        row[c] = '◄';
    }
    if c + 1 < row.len() {
        row[c + 1] = '─';
    }
    if c + 2 < row.len() {
        row[c + 2] = '┘';
    }
    let s: String = row.into_iter().collect();
    pad_right(s.trim_end(), ctx.inner_width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::display_width;
    use crate::result::RenderContext;
    use crate::schema::{Message, SequenceDiagram};

    fn ctx(width: usize) -> RenderContext {
        RenderContext { inner_width: width, total_width: u16::try_from(width).unwrap(), warnings: Vec::new() }
    }

    #[test]
    fn should_render_actors_and_arrows_when_messages_provided() {
        let d = SequenceDiagram {
            title: None,
            actors: vec!["Client".into(), "Server".into()],
            messages: vec![Message { from: "Client".into(), to: "Server".into(), label: "GET".into() }],
        };
        let lines = render(&d, &mut ctx(40)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Client")));
        assert!(lines.iter().any(|l| l.contains("Server")));
        for l in &lines { assert_eq!(display_width(l), 40); }
    }

    #[test]
    fn should_return_error_when_actors_empty() {
        let d = SequenceDiagram { title: None, actors: vec![], messages: vec![] };
        assert!(render(&d, &mut ctx(40)).is_err());
    }

    #[test]
    fn should_warn_when_message_references_unknown_actor() {
        let d = SequenceDiagram {
            title: None,
            actors: vec!["A".into()],
            messages: vec![Message { from: "A".into(), to: "Unknown".into(), label: "x".into() }],
        };
        let mut c = ctx(40);
        let _ = render(&d, &mut c);
        assert!(!c.warnings.is_empty());
    }

    #[test]
    fn should_render_single_actor_when_no_messages() {
        let d = SequenceDiagram { title: None, actors: vec!["Solo".into()], messages: vec![] };
        let lines = render(&d, &mut ctx(20)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Solo")));
        for l in &lines { assert_eq!(display_width(l), 20); }
    }

    #[test]
    fn should_render_loopback_arrow_when_self_message() {
        let d = SequenceDiagram {
            title: None,
            actors: vec!["A".into(), "B".into()],
            messages: vec![Message { from: "A".into(), to: "A".into(), label: "tick".into() }],
        };
        let mut c = ctx(40);
        let lines = render(&d, &mut c).unwrap();
        assert!(lines.iter().any(|l| l.contains("├─┐")), "missing loopback top");
        assert!(lines.iter().any(|l| l.contains("◄─┘")), "missing loopback bottom");
        assert!(lines.iter().any(|l| l.contains("tick")), "missing label");
        for l in &lines { assert_eq!(display_width(l), 40); }
    }
}
