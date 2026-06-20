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
fn flow_linear_steps_alignment() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![
            FlowStep::Label("Build".into()),
            FlowStep::Label("Test".into()),
            FlowStep::Label("Deploy".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        let expected_w = output.lines().next().map(display_width).unwrap_or(0);
        assert_eq!(display_width(line), expected_w, "misaligned: {line:?}");
    }
}

#[test]
fn flow_nested_subflow_alignment() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![
            FlowStep::Label("Start".into()),
            FlowStep::Sub(SubFlow {
                label: "Processing".into(),
                steps: vec![
                    FlowStep::Label("Step A".into()),
                    FlowStep::Label("Step B".into()),
                ],
            }),
            FlowStep::Label("End".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    let expected_w = display_width(output.lines().next().unwrap());
    assert!(expected_w <= 40, "exceeds max width: {expected_w}");
    for line in output.lines() {
        assert_eq!(display_width(line), expected_w, "misaligned: {line:?}");
    }
}

#[test]
fn flow_empty_steps_error() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
}

#[test]
fn flow_linear_renders_boxes_and_arrows() {
    let d = Diagram::Flow(FlowDiagram {
        title: Some("Pipeline".into()),
        direction: None,
        steps: Some(vec![
            FlowStep::Label("Build".into()),
            FlowStep::Label("Test".into()),
            FlowStep::Label("Deploy".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 40);
    assert_and_print("Flow: Linear Pipeline", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Pipeline"));
    assert!(output.contains("│ Build │"));
    assert!(output.contains("│ Test │"));
    assert!(output.contains("│ Deploy │"));
    assert!(output.contains('▼'));
    assert!(output.starts_with('┌'));
    assert!(output.ends_with('┘'));
}

#[test]
fn flow_nested_shows_indented_children() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![
            FlowStep::Label("Init".into()),
            FlowStep::Sub(SubFlow {
                label: "Process".into(),
                steps: vec![
                    FlowStep::Label("Validate".into()),
                    FlowStep::Label("Transform".into()),
                ],
            }),
            FlowStep::Label("Done".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 40);
    assert_and_print("Flow: Nested Subflow", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Process"));
    assert!(output.contains("Validate"));
    assert!(output.contains("Transform"));
}

#[test]
fn complex_flow_with_many_steps() {
    let d = Diagram::Flow(FlowDiagram {
        title: Some("CI/CD Pipeline".into()),
        direction: None,
        steps: Some(vec![
            FlowStep::Label("Checkout".into()),
            FlowStep::Label("Install Deps".into()),
            FlowStep::Sub(SubFlow {
                label: "Quality Gates".into(),
                steps: vec![
                    FlowStep::Label("Lint".into()),
                    FlowStep::Label("Unit Tests".into()),
                    FlowStep::Label("Integration Tests".into()),
                ],
            }),
            FlowStep::Label("Build Artifact".into()),
            FlowStep::Label("Deploy to Staging".into()),
            FlowStep::Label("Smoke Tests".into()),
            FlowStep::Label("Deploy to Prod".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 50);
    assert_and_print("Complex Flow: CI/CD Pipeline", &result, 50);
    assert!(result.fit);
    let output = result.output.unwrap();
    assert!(output.contains("CI/CD Pipeline"));
    assert!(output.contains("Quality Gates"));
    assert!(output.contains("Integration Tests"));
    assert!(output.contains("Deploy to Prod"));
}

#[test]
fn flow_graph_mode_with_edges() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: None,
        nodes: Some(vec![
            Node {
                id: Some("req".into()),
                label: "Request".into(),
            },
            Node {
                id: Some("auth".into()),
                label: "Auth Check".into(),
            },
            Node {
                id: Some("handler".into()),
                label: "Handler".into(),
            },
            Node {
                id: Some("resp".into()),
                label: "Response".into(),
            },
        ]),
        edges: Some(vec![
            Edge {
                from: "req".into(),
                to: "auth".into(),
                label: Some("validate".into()),
            },
            Edge {
                from: "auth".into(),
                to: "handler".into(),
                label: Some("authorized".into()),
            },
            Edge {
                from: "handler".into(),
                to: "resp".into(),
                label: None,
            },
        ]),
    });
    let result = render(&d, 40);
    assert_and_print("Flow: Graph Mode with Edge Labels", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Request"));
    assert!(output.contains("Auth Check"));
    assert!(output.contains("validate"));
    assert!(output.contains("authorized"));
}

#[test]
fn narrow_width_all_types_still_align() {
    let width: u16 = 20;

    let cases: Vec<(&str, Diagram)> = vec![
        (
            "Freeform@20",
            Diagram::Freeform(FreeformDiagram {
                title: None,
                content: Some("short\nlines".into()),
                lines: None,
            }),
        ),
        (
            "Flow@20",
            Diagram::Flow(FlowDiagram {
                title: None,
                direction: None,
                steps: Some(vec![
                    FlowStep::Label("A".into()),
                    FlowStep::Label("B".into()),
                ]),
                nodes: None,
                edges: None,
            }),
        ),
        (
            "State@20",
            Diagram::State(StateDiagram {
                title: None,
                states: None,
                transitions: vec![Edge {
                    from: "X".into(),
                    to: "Y".into(),
                    label: None,
                }],
            }),
        ),
        (
            "Tree@20",
            Diagram::Tree(TreeDiagram {
                title: None,
                root: None,
                indent: Some("r\n  a\n  b".into()),
            }),
        ),
        (
            "Table@20",
            Diagram::Table(TableDiagram {
                title: None,
                headers: vec!["H".into()],
                rows: vec![vec!["v".into()]],
            }),
        ),
    ];

    for (label, d) in &cases {
        let result = render(d, width);
        assert_and_print(label, &result, width);
    }
}

#[test]
fn flow_empty_label_does_not_panic() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![
            FlowStep::Label("".into()),
            FlowStep::Label("B".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 30);
    assert!(result.output.is_some() || !result.errors.is_empty());
}
