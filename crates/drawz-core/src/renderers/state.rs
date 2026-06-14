use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::StateDiagram;

/// Render state diagram as vertical state boxes connected by labeled arrows.
///
/// # Errors
///
/// Returns an error if transitions are empty.
pub fn render(diagram: &StateDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    if diagram.transitions.is_empty() {
        return Err("state diagram requires at least one transition".to_string());
    }

    // Collect unique states in order of appearance
    let mut states: Vec<&str> = Vec::new();
    for t in &diagram.transitions {
        if !states.contains(&t.from.as_str()) {
            states.push(&t.from);
        }
        if !states.contains(&t.to.as_str()) {
            states.push(&t.to);
        }
    }

    // If explicit states provided, use their labels
    let state_labels: Vec<&str> = if let Some(explicit) = &diagram.states {
        explicit.iter().map(|n| n.label.as_str()).collect()
    } else {
        states.clone()
    };

    let mut lines = Vec::new();

    for (i, &state) in state_labels.iter().enumerate() {
        // Render state box
        let box_lines = render_state_box(state, ctx);
        lines.extend(box_lines);

        // Find transition from this state
        if i < state_labels.len() - 1 {
            let transition = diagram.transitions.iter().find(|t| {
                t.from == state && state_labels.get(i + 1).is_some_and(|&next| t.to == next)
            });

            if let Some(t) = transition {
                if let Some(label) = &t.label {
                    let arrow = format!("  │ {label}");
                    lines.push(fit_line(&arrow, ctx));
                } else {
                    lines.push(fit_line("  │", ctx));
                }
            } else {
                lines.push(fit_line("  │", ctx));
            }
            lines.push(fit_line("  ▼", ctx));
        }
    }

    Ok(lines)
}

fn render_state_box(label: &str, ctx: &mut RenderContext) -> Vec<String> {
    let max_label_w = ctx.inner_width.saturating_sub(6); // "( " + label + " )"
    let fitted = if display_width(label) > max_label_w {
        ctx.warnings.push("suggestion: some state names truncated to fit width".to_string());
        truncate(label, max_label_w)
    } else {
        label.to_string()
    };

    let inner_w = display_width(&fitted) + 2; // space + label + space
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let mid = format!("│ {fitted} │");
    let bot = format!("╰{}╯", "─".repeat(inner_w));

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
    use crate::schema::{Edge, Node, StateDiagram};

    fn ctx(width: usize) -> RenderContext {
        RenderContext { inner_width: width, total_width: u16::try_from(width).unwrap(), warnings: Vec::new() }
    }

    #[test]
    fn should_render_rounded_boxes_when_transitions_provided() {
        let d = StateDiagram {
            title: None, states: None,
            transitions: vec![
                Edge { from: "A".into(), to: "B".into(), label: Some("go".into()) },
                Edge { from: "B".into(), to: "C".into(), label: None },
            ],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains('A')));
        assert!(lines.iter().any(|l| l.contains("go")));
        assert!(lines.iter().any(|l| l.contains("╭")));
        for l in &lines { assert_eq!(display_width(l), 30); }
    }

    #[test]
    fn should_use_explicit_labels_when_states_provided() {
        let d = StateDiagram {
            title: None,
            states: Some(vec![
                Node { id: None, label: "Start".into() },
                Node { id: None, label: "End".into() },
            ]),
            transitions: vec![Edge { from: "Start".into(), to: "End".into(), label: None }],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Start")));
        assert!(lines.iter().any(|l| l.contains("End")));
        for l in &lines { assert_eq!(display_width(l), 30); }
    }

    #[test]
    fn should_return_error_when_transitions_empty() {
        let d = StateDiagram { title: None, states: None, transitions: vec![] };
        assert!(render(&d, &mut ctx(30)).is_err());
    }

    #[test]
    fn should_truncate_when_state_name_exceeds_width() {
        let d = StateDiagram {
            title: None, states: None,
            transitions: vec![Edge {
                from: "a_very_long_state_name_here".into(),
                to: "b".into(), label: None,
            }],
        };
        let mut c = ctx(15);
        let lines = render(&d, &mut c).unwrap();
        for l in &lines { assert_eq!(display_width(l), 15); }
    }
}
