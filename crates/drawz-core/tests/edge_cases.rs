//! Edge case tests for features added during this development cycle.

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

// --- Flow direction: LR ---

#[test]
fn flow_lr_with_short_labels() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: Some("LR".into()),
        steps: Some(vec![
            FlowStep::Label("A".into()),
            FlowStep::Label("B".into()),
            FlowStep::Label("C".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 50);
    assert_aligned(&result, 50);
    let output = result.output.unwrap();
    // All on same line (horizontal)
    assert!(output
        .lines()
        .any(|l| l.contains('A') && l.contains('B') && l.contains('C')));
}

#[test]
fn flow_lr_renders_horizontally_even_with_long_labels() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: Some("LR".into()),
        steps: Some(vec![
            FlowStep::Label("First Very Long Step".into()),
            FlowStep::Label("Second Very Long Step".into()),
            FlowStep::Label("Third Very Long Step".into()),
        ]),
        nodes: None,
        edges: None,
    });
    // At wide width: horizontal
    let result = render(&d, 120);
    assert!(result.output.is_some());
    let output = result.output.unwrap();
    assert!(output
        .lines()
        .any(|l| l.contains("First") && l.contains("Second") && l.contains("Third")));

    // At narrow width: falls back to vertical
    let result2 = render(&d, 60);
    assert!(result2.output.is_some());
    let output2 = result2.output.unwrap();
    assert!(!output2
        .lines()
        .any(|l| l.contains("First") && l.contains("Second")));
}

#[test]
fn flow_lr_single_step() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: Some("LR".into()),
        steps: Some(vec![FlowStep::Label("Solo".into())]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    assert!(result.output.unwrap().contains("Solo"));
}

// --- Node string shorthand ---

#[test]
fn dag_nodes_as_strings() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: Some(vec![
            Node {
                id: None,
                label: "Alpha".into(),
            },
            Node {
                id: None,
                label: "Beta".into(),
            },
        ]),
        edges: vec![Edge {
            from: "Alpha".into(),
            to: "Beta".into(),
            label: None,
        }],
        subgraphs: None,
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Alpha"));
    assert!(output.contains("Beta"));
}

#[test]
fn dag_mixed_string_and_object_nodes() {
    // Deserialization test via JSON
    let json =
        r#"{"type":"dag","nodes":["A",{"id":"b","label":"Beta"}],"edges":[{"from":"A","to":"b"}]}"#;
    let input: DiagramInput = serde_json::from_str(json).unwrap();
    let result = render(&input.diagram, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains("Beta"));
}

// --- Tree 4-space indent ---

#[test]
fn tree_four_space_indent() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        root: None,
        indent: Some(
            "project\n    src/\n        main.rs\n        lib.rs\n    tests/\n        test.rs"
                .into(),
        ),
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("project"));
    assert!(output.contains("main.rs"));
    assert!(output.contains("test.rs"));
    assert!(output.contains("├──") || output.contains("└──"));
}

#[test]
fn tree_single_line_no_children() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        root: None,
        indent: Some("root".into()),
    });
    let result = render(&d, 20);
    assert_aligned(&result, 20);
    assert!(result.output.unwrap().contains("root"));
}

// --- Mermaid branching → DAG ---

#[test]
fn mermaid_branching_becomes_dag() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\nA-->B\nA-->C\nB-->D\nC-->D".into(),
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    // B and C should be on same line (DAG parallel layer)
    assert!(output.lines().any(|l| l.contains('B') && l.contains('C')));
}

#[test]
fn mermaid_linear_stays_flow() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\nA-->B\nB-->C".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    // Vertical flow — each on different line
    assert!(!output.lines().any(|l| l.contains('A') && l.contains('C')));
    assert!(output.contains("▼"));
}

// --- Table grid borders ---

#[test]
fn table_has_full_grid_borders() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["H1".into(), "H2".into()],
        rows: vec![vec!["a".into(), "b".into()], vec!["c".into(), "d".into()]],
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    assert!(output.contains('┌'));
    assert!(output.contains('┬'));
    assert!(output.contains('┐'));
    assert!(output.contains('├'));
    assert!(output.contains('┼'));
    assert!(output.contains('┤'));
    assert!(output.contains('└'));
    assert!(output.contains('┴'));
    assert!(output.contains('┘'));
}

#[test]
fn table_row_with_more_cells_than_headers() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into()],
        rows: vec![vec!["1".into(), "2".into(), "extra".into()]],
    });
    let result = render(&d, 30);
    // Should not panic — extra cells ignored
    assert!(result.output.is_some() || !result.errors.is_empty());
}

// --- Freeform box validation ---

#[test]
fn freeform_no_warning_on_aligned_boxes() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some("┌─────┐\n│ box │\n└─────┘".into()),
        lines: None,
    });
    let result = render(&d, 40);
    assert!(result.warnings.is_empty());
}

// --- DAG with ascii-dag layout ---

#[test]
fn dag_fan_out_parallel_nodes() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges: vec![
            Edge {
                from: "Root".into(),
                to: "A".into(),
                label: None,
            },
            Edge {
                from: "Root".into(),
                to: "B".into(),
                label: None,
            },
            Edge {
                from: "Root".into(),
                to: "C".into(),
                label: None,
            },
        ],
        subgraphs: None,
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    // A, B, C should be on the same line
    assert!(output
        .lines()
        .any(|l| l.contains('A') && l.contains('B') && l.contains('C')));
}

// --- Width boundaries ---

#[test]
fn all_types_at_width_5() {
    let diagrams: Vec<(&str, Diagram)> = vec![
        (
            "freeform",
            Diagram::Freeform(FreeformDiagram {
                title: None,
                content: Some("x".into()),
                lines: None,
            }),
        ),
        (
            "flow",
            Diagram::Flow(FlowDiagram {
                title: None,
                direction: None,
                steps: Some(vec![FlowStep::Label("A".into())]),
                nodes: None,
                edges: None,
            }),
        ),
        (
            "dag",
            Diagram::Dag(DagDiagram {
                title: None,
                nodes: None,
                edges: vec![Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                }],
                subgraphs: None,
            }),
        ),
        (
            "state",
            Diagram::State(StateDiagram {
                title: None,
                states: None,
                transitions: vec![Edge {
                    from: "A".into(),
                    to: "B".into(),
                    label: None,
                }],
            }),
        ),
    ];
    for (name, d) in &diagrams {
        let result = render(d, 5);
        // Should not panic — may error or produce tiny output
        assert!(
            result.output.is_some() || !result.errors.is_empty(),
            "{name} panicked at width=5"
        );
    }
}

// --- Sequence truncation warning ---

#[test]
fn sequence_narrow_width_does_not_panic() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
        messages: vec![Message {
            from: "A".into(),
            to: "B".into(),
            label: "this is a very long message".into(),
        }],
    });
    let result = render(&d, 30);
    // Should render (maybe with truncation) or error, but not panic
    assert!(result.output.is_some() || !result.errors.is_empty());
}

// --- Frame shrinks to content ---

#[test]
fn outer_frame_shrinks_to_content_not_requested_width() {
    // A short flow at width 120 should NOT produce 120-col output
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![
            FlowStep::Label("A".into()),
            FlowStep::Label("B".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 120);
    let output = result.output.unwrap();
    let w = display_width(output.lines().next().unwrap());
    assert!(w < 120, "frame should shrink: got {w}");
    assert!(w >= 40, "frame should respect 40-col minimum: got {w}");
    // All lines same width (alignment invariant)
    for line in output.lines() {
        assert_eq!(display_width(line), w, "misaligned: {line:?}");
    }
}

#[test]
fn outer_frame_respects_40_col_minimum() {
    // Very short content should still get 40-col frame
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some("hi".into()),
        lines: None,
    });
    let result = render(&d, 120);
    let output = result.output.unwrap();
    let w = display_width(output.lines().next().unwrap());
    assert_eq!(w, 40, "expected 40-col minimum frame, got {w}");
}

#[test]
fn outer_frame_does_not_exceed_requested_width() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![FlowStep::Label("short".into())]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 30);
    let output = result.output.unwrap();
    let w = display_width(output.lines().next().unwrap());
    assert!(w <= 30, "should not exceed requested width: got {w}");
}

#[test]
fn subflow_frame_hugs_content_not_outer_width() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![FlowStep::Sub(SubFlow {
            label: "Parent".into(),
            steps: vec![FlowStep::Label("x".into())],
        })]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 120);
    let output = result.output.unwrap();
    // The dashed sub-flow border should be much narrower than 120
    let dashed_line = output.lines().find(|l| l.contains('╌')).unwrap();
    let dashed_w = display_width(dashed_line.trim_end());
    assert!(
        dashed_w < 50,
        "sub-flow frame should hug content: got {dashed_w}"
    );
}

#[test]
fn wide_content_still_drives_frame_beyond_minimum() {
    // Content wider than 40 cols should drive the frame size
    let long_label = "this is a label that exceeds forty columns easily";
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![FlowStep::Label(long_label.into())]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 120);
    let output = result.output.unwrap();
    let w = display_width(output.lines().next().unwrap());
    assert!(
        w > 40,
        "wide content should push frame beyond minimum: got {w}"
    );
    assert!(w < 120, "should still shrink from requested 120: got {w}");
}
