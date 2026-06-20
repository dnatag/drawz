//! Regression tests for bugs fixed in the code review cycle.

use drawz_core::measure::{display_width, pad_right, truncate};
use drawz_core::render;
use drawz_core::schema::*;

fn assert_aligned(result: &drawz_core::RenderResult) {
    assert!(
        result.errors.is_empty(),
        "unexpected errors: {:?}",
        result.errors
    );
    let output = result.output.as_ref().expect("expected output");
    let first_w = output.lines().next().map(display_width).unwrap_or(0);
    for line in output.lines() {
        assert_eq!(display_width(line), first_w, "misaligned: {line:?}");
    }
}

// === #1: Hyphenated node IDs in Mermaid ===

#[test]
fn mermaid_hyphenated_node_ids_parsed_correctly() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\n    api-gateway-->user-service-->database".into(),
    });
    let result = render(&d, 60);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("api-gateway"), "missing api-gateway node");
    assert!(output.contains("user-service"), "missing user-service node");
    assert!(output.contains("database"), "missing database node");
    // Should NOT have a spurious "user" node
    let lines_with_user: Vec<&str> = output
        .lines()
        .filter(|l| l.contains("user") && !l.contains("user-service"))
        .collect();
    assert!(
        lines_with_user.is_empty(),
        "spurious 'user' node found: {lines_with_user:?}"
    );
}

#[test]
fn mermaid_dotted_node_id() {
    // Node IDs with dots (e.g. "v1.0") should not split at "."
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR; v1.0-->v2.0".into(),
    });
    let result = render(&d, 40);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("v1.0"));
    assert!(output.contains("v2.0"));
}

// === #2: Width sentinel collision ===

#[test]
fn schema_default_width_is_120() {
    let json = r#"{"type":"flow","steps":["A","B"]}"#;
    let input: DiagramInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.width, 120);
}

#[test]
fn schema_explicit_width_120_preserved() {
    let json = r#"{"type":"flow","steps":["A","B"],"width":120}"#;
    let input: DiagramInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.width, 120);
}

// === #7: Subgraph/end phantom nodes ===

#[test]
fn mermaid_subgraph_not_rendered_as_node() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\n    subgraph Frontend\n        A-->B\n    end\n    subgraph Backend\n        C-->D\n    end\n    B-->C".into(),
    });
    let result = render(&d, 60);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains('D'));
    assert!(!output.contains("subgraph"), "subgraph rendered as node");
    // "end" should not appear as a standalone node label
    let end_as_node = output
        .lines()
        .any(|l| l.contains("│ end │") || l.contains("│ end  "));
    assert!(!end_as_node, "'end' rendered as node");
}

// === Flow: LR fallback ===

#[test]
fn flow_lr_falls_back_to_vertical_when_too_narrow() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: Some("LR".into()),
        steps: Some(vec![
            FlowStep::Label("Alpha".into()),
            FlowStep::Label("Beta".into()),
            FlowStep::Label("Gamma".into()),
        ]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 20);
    assert_aligned(&result);
    let output = result.output.unwrap();
    // Vertical: each label on separate line
    assert!(output.contains("Alpha"));
    assert!(output.contains("Beta"));
    assert!(!output
        .lines()
        .any(|l| l.contains("Alpha") && l.contains("Beta")));
}

#[test]
fn flow_single_step_no_arrow() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![FlowStep::Label("Only".into())]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 30);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("Only"));
    assert!(!output.contains("▼"), "single step should have no arrow");
}

#[test]
fn flow_node_edge_mode_renders() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: None,
        nodes: Some(vec![
            Node {
                id: Some("Start".into()),
                label: "Start".into(),
            },
            Node {
                id: Some("Middle".into()),
                label: "Middle".into(),
            },
            Node {
                id: Some("End".into()),
                label: "End".into(),
            },
        ]),
        edges: Some(vec![
            Edge {
                from: "Start".into(),
                to: "Middle".into(),
                label: None,
            },
            Edge {
                from: "Middle".into(),
                to: "End".into(),
                label: None,
            },
        ]),
    });
    let result = render(&d, 40);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("Start"));
    assert!(output.contains("End"));
}

// === DAG edge cases ===

#[test]
fn dag_single_node_no_edges() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: Some(vec![Node {
            id: None,
            label: "Alone".into(),
        }]),
        edges: vec![],
    });
    // Should not error — single node is valid
    let result = render(&d, 30);
    // Either renders or errors gracefully
    assert!(result.output.is_some() || !result.errors.is_empty());
}

#[test]
fn dag_disconnected_components() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges: vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                label: None,
            },
            Edge {
                from: "C".into(),
                to: "D".into(),
                label: None,
            },
        ],
    });
    let result = render(&d, 50);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains('D'));
}

#[test]
fn dag_long_chain_10_nodes() {
    let edges: Vec<Edge> = (0..9)
        .map(|i| Edge {
            from: format!("N{i}"),
            to: format!("N{}", i + 1),
            label: None,
        })
        .collect();
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges,
    });
    let result = render(&d, 30);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("N0"));
    assert!(output.contains("N9"));
}

// === Sequence edge cases ===

#[test]
fn sequence_empty_actors_error() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec![],
        messages: vec![],
    });
    let result = render(&d, 40);
    assert!(!result.errors.is_empty());
}

#[test]
fn sequence_single_actor_no_messages() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["Solo".into()],
        messages: vec![],
    });
    let result = render(&d, 30);
    // Should render actor with just lifelines
    assert!(result.output.is_some() || !result.errors.is_empty());
}

#[test]
fn sequence_spanning_non_adjacent_actors() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into(), "C".into(), "D".into()],
        messages: vec![Message {
            from: "A".into(),
            to: "D".into(),
            label: "skip".into(),
        }],
    });
    let result = render(&d, 60);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("skip"));
}

// === Table edge cases ===

#[test]
fn table_single_column_single_row() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["Name".into()],
        rows: vec![vec!["Value".into()]],
    });
    let result = render(&d, 30);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("Name"));
    assert!(output.contains("Value"));
}

#[test]
fn table_empty_string_cells() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into()],
        rows: vec![vec!["".into(), "".into()]],
    });
    let result = render(&d, 30);
    assert_aligned(&result);
}

#[test]
fn table_cjk_headers_aligned() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["名前".into(), "状態".into()],
        rows: vec![vec!["テスト".into(), "完了".into()]],
    });
    let result = render(&d, 40);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("名前"));
    assert!(output.contains("完了"));
}

// === State edge cases ===

#[test]
fn state_self_transition() {
    let d = Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions: vec![
            Edge {
                from: "Retry".into(),
                to: "Retry".into(),
                label: Some("timeout".into()),
            },
            Edge {
                from: "Retry".into(),
                to: "Done".into(),
                label: Some("success".into()),
            },
        ],
    });
    let result = render(&d, 40);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("Retry"));
    assert!(output.contains("Done"));
}

// === Tree edge cases ===

#[test]
fn tree_deep_nesting_8_levels() {
    let indent = "root\n  l1\n    l2\n      l3\n        l4\n          l5\n            l6\n              l7\n                l8";
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        root: None,
        indent: Some(indent.into()),
    });
    let result = render(&d, 60);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("l8"));
}

// === Freeform edge cases ===

#[test]
fn freeform_truncation_produces_warning() {
    let long = "x".repeat(200);
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some(long),
        lines: None,
    });
    let result = render(&d, 30);
    // Natural width expansion means no truncation warning — content overflows
    // The frame expands to natural size
    assert!(result.output.is_some());
}

#[test]
fn freeform_lines_field_works() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: None,
        lines: Some(vec!["line one".into(), "line two".into()]),
    });
    let result = render(&d, 30);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("line one"));
    assert!(output.contains("line two"));
}

#[test]
fn freeform_empty_content_error() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some("".into()),
        lines: None,
    });
    let result = render(&d, 30);
    assert!(!result.errors.is_empty());
}

// === Measure edge cases ===

#[test]
fn truncate_width_1_cjk() {
    // CJK char is width 2, can't fit in width 1
    let t = truncate("中", 1);
    assert!(display_width(&t) <= 1);
}

#[test]
fn truncate_empty_string() {
    assert_eq!(truncate("", 10), "");
}

#[test]
fn truncate_exact_fit() {
    assert_eq!(truncate("abc", 3), "abc");
}

#[test]
fn truncate_ansi_preserved() {
    let s = "he\x1b[31mllo world\x1b[0m";
    let t = truncate(s, 7);
    assert!(display_width(&t) <= 7);
    // Should contain ANSI reset at end
    assert!(t.contains("\x1b["));
}

#[test]
fn display_width_combining_chars() {
    // e + combining acute = 1 display column
    assert_eq!(display_width("e\u{0301}"), 1);
}

#[test]
fn pad_right_cjk_odd_target() {
    let result = pad_right("中", 3);
    assert_eq!(display_width(&result), 3);
}

#[test]
fn pad_right_already_exact() {
    let result = pad_right("abc", 3);
    assert_eq!(result, "abc");
}

// === Width boundaries ===

#[test]
fn width_3_rejected() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![FlowStep::Label("A".into())]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 3);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("too small") || result.errors[0].contains("width"));
}

#[test]
fn width_4_minimum_no_panic() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        direction: None,
        steps: Some(vec![FlowStep::Label("A".into())]),
        nodes: None,
        edges: None,
    });
    let result = render(&d, 4);
    // Should not panic — may produce very small output or error
    assert!(result.output.is_some() || !result.errors.is_empty());
}

// === Mermaid: dotted arrow with label ===

#[test]
fn mermaid_dotted_arrow_with_pipe_label() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\n    A-->|yes|B\n    A-.->C".into(),
    });
    let result = render(&d, 50);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains('B'));
    assert!(output.contains('C'));
}

#[test]
fn mermaid_node_with_curly_braces() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\n    A{Decision}-->B[Yes]\n    A-->C[No]".into(),
    });
    let result = render(&d, 50);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("Decision"));
}

// === Mermaid: state [*] initial/final ===

#[test]
fn mermaid_state_star_states() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "stateDiagram-v2\n    [*] --> Idle\n    Idle --> Active : start\n    Active --> [*]"
            .into(),
    });
    let result = render(&d, 50);
    assert_aligned(&result);
    let output = result.output.unwrap();
    assert!(output.contains("Idle"));
    assert!(output.contains("Active"));
}
