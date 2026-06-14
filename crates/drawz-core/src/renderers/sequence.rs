//! Sequence diagram renderer — actors, lifelines, and messages.

use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::SequenceDiagram;

/// Render a sequence diagram with actor columns and message arrows.
///
/// # Errors
///
/// Returns an error if actors list is empty.
pub fn render(diagram: &SequenceDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    if diagram.actors.is_empty() {
        return Err("sequence diagram requires at least one actor".to_string());
    }

    let num_actors = diagram.actors.len();
    if num_actors == 1 && diagram.messages.is_empty() {
        // Single actor, no messages — just show the actor
        let lines = vec![fit_line(&diagram.actors[0], ctx)];
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

        let msg_line = render_message(from, to, &msg.label, num_actors, col_width, ctx);
        lines.push(msg_line);
        lines.push(lifeline.clone());
    }

    Ok(lines)
}

fn render_actor_row(actors: &[String], col_width: usize, ctx: &mut RenderContext) -> String {
    let row: String = actors
        .iter()
        .map(|a| {
            let max_w = col_width.saturating_sub(1);
            if display_width(a) > max_w {
                truncate(a, max_w)
            } else {
                pad_right(a, max_w)
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
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
) -> String {
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
        // Self-message
        let c = from_center;
        if c < row.len() {
            row[c] = '│';
        }
        let label_start = c + 2;
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
    } else {
        let (left, right) = if from < to {
            (from_center, to_center)
        } else {
            (to_center, from_center)
        };
        let arrow_right = from < to;

        // Draw the arrow line
        for c in left..=right {
            if c < row.len() {
                row[c] = '─';
            }
        }
        // Arrow head
        if arrow_right {
            if right < row.len() {
                row[right] = '►';
            }
            if left < row.len() {
                row[left] = '├';
            }
        } else {
            if left < row.len() {
                row[left] = '◄';
            }
            if right < row.len() {
                row[right] = '┤';
            }
        }

        // Place label in the middle
        let mid = left + (right - left) / 2;
        let lw = display_width(label);
        let label_start = mid.saturating_sub(lw / 2);
        let max_label = right.saturating_sub(left).saturating_sub(2);
        let fitted = if lw > max_label {
            truncate(label, max_label)
        } else {
            label.to_string()
        };
        for (j, ch) in fitted.chars().enumerate() {
            let pos = label_start + j;
            if pos < row.len() && pos > left && pos < right {
                row[pos] = ch;
            }
        }
    }

    let s: String = row.into_iter().collect();
    pad_right(s.trim_end(), ctx.inner_width)
}

fn fit_line(line: &str, ctx: &mut RenderContext) -> String {
    pad_right(line, ctx.inner_width)
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
}
