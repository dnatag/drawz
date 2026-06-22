use drawz_core::measure::display_width;
use drawz_core::render;
use drawz_core::schema::*;

fn assert_aligned(result: &drawz_core::RenderResult, _width: u16) {
    assert!(
        result.errors.is_empty(),
        "unexpected errors: {:?}",
        result.errors
    );
    let output = result.output.as_ref().expect("expected output");
    for line in output.lines() {
        let first_w = output.lines().next().map(display_width).unwrap_or(0);
        assert_eq!(display_width(line), first_w, "misaligned: {line:?}");
    }
}

fn assert_and_print(label: &str, result: &drawz_core::RenderResult, width: u16) {
    assert_aligned(result, width);
    let output = result.output.as_ref().unwrap();
    let sep = "═".repeat(60);
    println!("\n{sep}");
    println!("  {label}  (width={width})");
    println!("{sep}");
    println!("{output}");
    println!();
}

#[test]
fn state_transitions_alignment() {
    let d = Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions: vec![
            Edge {
                from: "Idle".into(),
                to: "Running".into(),
                label: Some("start".into()),
            },
            Edge {
                from: "Running".into(),
                to: "Done".into(),
                label: None,
            },
        ],
    });
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    let expected_w = display_width(output.lines().next().unwrap());
    assert!(expected_w <= 30);
    for line in output.lines() {
        assert_eq!(display_width(line), expected_w, "misaligned: {line:?}");
    }
    assert!(output.contains("╭") && output.contains("╰"));
}

#[test]
fn state_empty_transitions_error() {
    let d = Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions: vec![],
    });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
}

#[test]
fn state_renders_rounded_boxes_with_labels() {
    let d = Diagram::State(StateDiagram {
        title: Some("Order Lifecycle".into()),
        states: None,
        transitions: vec![
            Edge {
                from: "Created".into(),
                to: "Paid".into(),
                label: Some("pay".into()),
            },
            Edge {
                from: "Paid".into(),
                to: "Shipped".into(),
                label: Some("ship".into()),
            },
            Edge {
                from: "Shipped".into(),
                to: "Delivered".into(),
                label: None,
            },
        ],
    });
    let result = render(&d, 40);
    assert_and_print("State: Order Lifecycle", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("╭"));
    assert!(output.contains("╰"));
    assert!(output.contains("Created"));
    assert!(output.contains("Delivered"));
    assert!(output.contains("pay"));
    assert!(output.contains("ship"));
    assert!(output.contains("Order Lifecycle"));
}

#[test]
fn complex_state_machine() {
    let d = Diagram::State(StateDiagram {
        title: Some("TCP Connection".into()),
        states: None,
        transitions: vec![
            Edge {
                from: "CLOSED".into(),
                to: "SYN_SENT".into(),
                label: Some("connect()".into()),
            },
            Edge {
                from: "SYN_SENT".into(),
                to: "ESTABLISHED".into(),
                label: Some("SYN+ACK".into()),
            },
            Edge {
                from: "ESTABLISHED".into(),
                to: "FIN_WAIT".into(),
                label: Some("close()".into()),
            },
            Edge {
                from: "FIN_WAIT".into(),
                to: "CLOSED".into(),
                label: Some("ACK".into()),
            },
        ],
    });
    let result = render(&d, 40);
    assert_and_print("Complex State: TCP Connection", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("TCP Connection"));
    assert!(output.contains("CLOSED"));
    assert!(output.contains("ESTABLISHED"));
    assert!(output.contains("connect()"));
}

#[test]
fn state_branch_renders_horizontally() {
    let d = Diagram::State(StateDiagram {
        title: Some("Error Handling".into()),
        states: None,
        transitions: vec![
            Edge {
                from: "Idle".into(),
                to: "Running".into(),
                label: Some("start".into()),
            },
            Edge {
                from: "Running".into(),
                to: "Done".into(),
                label: Some("complete".into()),
            },
            Edge {
                from: "Running".into(),
                to: "Failed".into(),
                label: Some("error".into()),
            },
            Edge {
                from: "Failed".into(),
                to: "Idle".into(),
                label: Some("retry".into()),
            },
        ],
    });
    let result = render(&d, 50);
    assert_and_print(
        "State: Branching (Running → Failed horizontal)",
        &result,
        50,
    );
    let output = result.output.unwrap();
    // Branch target should be on same line as source
    assert!(
        output
            .lines()
            .any(|l| l.contains("Running") && l.contains("Failed")),
        "Running→Failed should render horizontally"
    );
    assert!(output.contains("error"), "branch label should appear");
}

#[test]
fn state_self_loop_renders() {
    let d = Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions: vec![
            Edge {
                from: "Retry".into(),
                to: "Retry".into(),
                label: Some("again".into()),
            },
            Edge {
                from: "Retry".into(),
                to: "Done".into(),
                label: Some("success".into()),
            },
        ],
    });
    let result = render(&d, 40);
    assert_and_print("State: Self-loop", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("↺"), "self-loop should show ↺ symbol");
    assert!(output.contains("again"), "self-loop label should appear");
}

#[test]
fn state_multiple_branches_from_one_state() {
    let d = Diagram::State(StateDiagram {
        title: Some("Jira Workflow".into()),
        states: None,
        transitions: vec![
            Edge {
                from: "Open".into(),
                to: "InProgress".into(),
                label: Some("assign".into()),
            },
            Edge {
                from: "InProgress".into(),
                to: "Done".into(),
                label: Some("resolve".into()),
            },
            Edge {
                from: "InProgress".into(),
                to: "Blocked".into(),
                label: Some("block".into()),
            },
            Edge {
                from: "InProgress".into(),
                to: "Cancelled".into(),
                label: Some("cancel".into()),
            },
        ],
    });
    let result = render(&d, 60);
    assert_and_print("State: Multiple branches from InProgress", &result, 60);
    let output = result.output.unwrap();
    // First branch should be horizontal
    assert!(
        output
            .lines()
            .any(|l| l.contains("InProgress") && l.contains("Blocked")),
        "first branch should be horizontal"
    );
    // Additional branches should appear as text annotations
    assert!(
        output.contains("cancel") && output.contains("Cancelled"),
        "second branch should appear as text"
    );
}
