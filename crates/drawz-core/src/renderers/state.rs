use std::collections::HashSet;

use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::StateDiagram;

/// Render state diagram with main path vertical and branch targets to the right.
///
/// # Errors
///
/// Returns an error if transitions are empty.
pub(crate) fn render(
    diagram: &StateDiagram,
    ctx: &mut RenderContext,
) -> Result<Vec<String>, String> {
    if diagram.transitions.is_empty() {
        return Err("state diagram requires at least one transition".to_string());
    }

    // Collect unique states in appearance order
    let mut states: Vec<&str> = Vec::new();
    for t in &diagram.transitions {
        for s in [t.from.as_str(), t.to.as_str()] {
            if !states.contains(&s) {
                states.push(s);
            }
        }
    }

    // Determine the main path: follow the first outgoing transition from each state
    let main_path = compute_main_path(&states, &diagram.transitions);

    // Get display labels (from explicit states if provided)
    let get_label = |state: &str| -> String {
        diagram
            .states
            .as_ref()
            .and_then(|nodes| {
                nodes
                    .iter()
                    .find(|n| n.label == state || n.id.as_deref() == Some(state))
            })
            .map_or_else(|| state.to_string(), |n| n.label.clone())
    };

    let mut lines = Vec::new();

    for (i, &state) in main_path.iter().enumerate() {
        let label = get_label(state);

        // Find branch transitions from this state (to states not next on main path)
        let next_main = main_path.get(i + 1).copied();
        let branches: Vec<(&str, &str)> = diagram
            .transitions
            .iter()
            .filter(|t| t.from == state && t.to != state && Some(t.to.as_str()) != next_main)
            .map(|t| (t.label.as_deref().unwrap_or(""), t.to.as_str()))
            .collect();

        if branches.is_empty() {
            // Simple: just the state box
            let box_lines = render_state_box(&label, ctx);
            lines.extend(box_lines);
        } else {
            // State box with horizontal branch arrow(s) to the right
            let branch_label = branches[0].0;
            let branch_target = get_label(branches[0].1);
            render_state_with_branch(&label, branch_label, &branch_target, ctx, &mut lines);

            // Additional branches as text annotations below
            for &(blabel, btarget) in &branches[1..] {
                let target_label = get_label(btarget);
                let annotation = if blabel.is_empty() {
                    format!("  │ → {target_label}")
                } else {
                    format!("  │ {blabel} → {target_label}")
                };
                lines.push(fit_line(&annotation, ctx));
            }
        }

        // Self-loop transitions
        for t in diagram
            .transitions
            .iter()
            .filter(|t| t.from == state && t.to == state)
        {
            let slabel = t.label.as_deref().unwrap_or("");
            lines.push(fit_line(&format!("  ↺ {slabel}"), ctx));
        }

        // Transition arrow to next main state
        if let Some(next) = next_main {
            let to_next = diagram
                .transitions
                .iter()
                .find(|t| t.from == state && t.to == next);
            if let Some(t) = to_next {
                if let Some(label) = &t.label {
                    lines.push(fit_line(&format!("  │ {label}"), ctx));
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

/// Compute the main path by following the first outgoing transition from each state.
fn compute_main_path<'a>(
    states: &[&'a str],
    transitions: &'a [crate::schema::Edge],
) -> Vec<&'a str> {
    let start = states[0];
    let mut path = vec![start];
    let mut current = start;
    let mut visited = HashSet::new();
    visited.insert(current);

    loop {
        // Follow the first non-self, non-visited outgoing transition
        let next = transitions
            .iter()
            .find(|t| t.from == current && t.to != current && !visited.contains(t.to.as_str()))
            .map(|t| t.to.as_str());

        match next {
            Some(n) if states.contains(&n) => {
                visited.insert(n);
                path.push(n);
                current = n;
            }
            _ => break,
        }
    }
    path
}

/// Render a state box with a horizontal branch arrow to a target on the right.
fn render_state_with_branch(
    state: &str,
    branch_label: &str,
    target: &str,
    ctx: &mut RenderContext,
    out: &mut Vec<String>,
) {
    let state_w = display_width(state) + 4; // "│ " + state + " │"
    let target_w = display_width(target) + 4;

    // Arrow: ──label──→ (or just ────→ if no label)
    let arrow_label = if branch_label.is_empty() {
        "────→".to_string()
    } else {
        format!("──{}──→", branch_label)
    };
    let arrow_w = display_width(&arrow_label);

    let gap = 1; // space between boxes and arrow

    // Check if it fits in width
    let total_w = state_w + gap + arrow_w + gap + target_w;
    if total_w > ctx.inner_width {
        // Doesn't fit — fall back to text annotation
        let box_lines = render_state_box(state, ctx);
        out.extend(box_lines);
        let annotation = if branch_label.is_empty() {
            format!("  │ → {target}")
        } else {
            format!("  │ {branch_label} → {target}")
        };
        out.push(fit_line(&annotation, ctx));
        return;
    }

    // Build the three lines side by side
    let state_top = format!("╭{}╮", "─".repeat(state_w - 2));
    let state_mid = format!("│ {} │", state);
    let state_bot = format!("╰{}╯", "─".repeat(state_w - 2));

    let target_top = format!("╭{}╮", "─".repeat(target_w - 2));
    let target_mid = format!("│ {} │", target);
    let target_bot = format!("╰{}╯", "─".repeat(target_w - 2));

    let spacer_top = " ".repeat(gap + arrow_w + gap);
    let spacer_bot = " ".repeat(gap + arrow_w + gap);
    let arrow_mid = format!("{}{}{}", " ".repeat(gap), arrow_label, " ".repeat(gap));

    out.push(fit_line(
        &format!("{state_top}{spacer_top}{target_top}"),
        ctx,
    ));
    out.push(fit_line(
        &format!("{state_mid}{arrow_mid}{target_mid}"),
        ctx,
    ));
    out.push(fit_line(
        &format!("{state_bot}{spacer_bot}{target_bot}"),
        ctx,
    ));
}

fn render_state_box(label: &str, ctx: &mut RenderContext) -> Vec<String> {
    let max_label_w = ctx.inner_width.saturating_sub(6);
    let fitted = if display_width(label) > max_label_w {
        ctx.warnings
            .push("suggestion: some state names truncated to fit width".to_string());
        truncate(label, max_label_w)
    } else {
        label.to_string()
    };

    let inner_w = display_width(&fitted) + 2;
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
        RenderContext {
            inner_width: width,
            total_width: u16::try_from(width).unwrap(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn should_render_rounded_boxes_when_transitions_provided() {
        let d = StateDiagram {
            title: None,
            states: None,
            transitions: vec![
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: Some("go".into()),
                },
                Edge {
                    from: "B".into(),
                    to: "C".into(),
                    label: None,
                },
            ],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains('A')));
        assert!(lines.iter().any(|l| l.contains("go")));
        assert!(lines.iter().any(|l| l.contains("╭")));
        for l in &lines {
            assert_eq!(display_width(l), 30);
        }
    }

    #[test]
    fn should_use_explicit_labels_when_states_provided() {
        let d = StateDiagram {
            title: None,
            states: Some(vec![
                Node {
                    id: None,
                    label: "Start".into(),
                },
                Node {
                    id: None,
                    label: "End".into(),
                },
            ]),
            transitions: vec![Edge {
                from: "Start".into(),
                to: "End".into(),
                label: None,
            }],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("Start")));
        assert!(lines.iter().any(|l| l.contains("End")));
        for l in &lines {
            assert_eq!(display_width(l), 30);
        }
    }

    #[test]
    fn should_return_error_when_transitions_empty() {
        let d = StateDiagram {
            title: None,
            states: None,
            transitions: vec![],
        };
        assert!(render(&d, &mut ctx(30)).is_err());
    }

    #[test]
    fn should_show_branching_transitions_with_horizontal_arrow() {
        let d = StateDiagram {
            title: None,
            states: None,
            transitions: vec![
                Edge {
                    from: "Running".into(),
                    to: "Done".into(),
                    label: Some("ok".into()),
                },
                Edge {
                    from: "Running".into(),
                    to: "Failed".into(),
                    label: Some("err".into()),
                },
            ],
        };
        let lines = render(&d, &mut ctx(50)).unwrap();
        // Branch target should appear on same line as source (horizontal arrow)
        let has_horizontal = lines
            .iter()
            .any(|l| l.contains("Running") && l.contains("Failed"));
        assert!(has_horizontal, "branch should render horizontally");
        for l in &lines {
            assert_eq!(display_width(l), 50);
        }
    }

    #[test]
    fn should_truncate_when_state_name_exceeds_width() {
        let d = StateDiagram {
            title: None,
            states: None,
            transitions: vec![Edge {
                from: "a_very_long_state_name_here".into(),
                to: "b".into(),
                label: None,
            }],
        };
        let mut c = ctx(15);
        let lines = render(&d, &mut c).unwrap();
        for l in &lines {
            assert_eq!(display_width(l), 15);
        }
    }

    #[test]
    fn should_render_self_loop() {
        let d = StateDiagram {
            title: None,
            states: None,
            transitions: vec![
                Edge {
                    from: "A".into(),
                    to: "A".into(),
                    label: Some("retry".into()),
                },
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                },
            ],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert!(lines.iter().any(|l| l.contains("↺") && l.contains("retry")));
    }

    #[test]
    fn should_compute_main_path_following_first_transitions() {
        let transitions = vec![
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
        ];
        let states = vec!["A", "B", "C", "D"];
        let path = compute_main_path(&states, &transitions);
        assert_eq!(path, vec!["A", "B", "D"]);
    }

    #[test]
    fn should_fall_back_to_text_when_branch_too_wide() {
        let d = StateDiagram {
            title: None,
            states: None,
            transitions: vec![
                Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: Some("go".into()),
                },
                Edge {
                    from: "A".into(),
                    to: "VeryLongStateName".into(),
                    label: Some("branch".into()),
                },
            ],
        };
        // Narrow width forces text fallback
        let lines = render(&d, &mut ctx(20)).unwrap();
        for l in &lines {
            assert_eq!(display_width(l), 20);
        }
    }
}
