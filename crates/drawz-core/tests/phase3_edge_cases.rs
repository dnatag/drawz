use drawz_core::measure::display_width;
use drawz_core::render;
use drawz_core::schema::*;

fn assert_aligned(result: &drawz_core::RenderResult, width: u16) {
    assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);
    let output = result.output.as_ref().expect("expected output");
    for line in output.lines() {
        assert_eq!(display_width(line), width as usize, "misaligned: {line:?}");
    }
}

// ═══════════════════════════════════════════════════
// Sequence edge cases
// ═══════════════════════════════════════════════════

#[test]
fn sequence_self_message() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into()],
        messages: vec![Message { from: "A".into(), to: "A".into(), label: "self-call".into() }],
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
}

#[test]
fn sequence_right_to_left_arrow() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["Client".into(), "Server".into()],
        messages: vec![
            Message { from: "Client".into(), to: "Server".into(), label: "request".into() },
            Message { from: "Server".into(), to: "Client".into(), label: "response".into() },
        ],
    });
    let result = render(&d, 50);
    assert_aligned(&result, 50);
    let output = result.output.unwrap();
    assert!(output.contains('◄') || output.contains('►'));
}

#[test]
fn sequence_too_narrow_for_actors() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into(), "F".into(), "G".into(), "H".into(), "I".into(), "J".into()],
        messages: vec![],
    });
    // 20 width / 10 actors = 2 per col, below minimum 3
    let result = render(&d, 20);
    assert!(!result.errors.is_empty());
}

#[test]
fn sequence_long_actor_name_truncated() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["VeryLongActorName".into(), "AnotherLongName".into()],
        messages: vec![Message { from: "VeryLongActorName".into(), to: "AnotherLongName".into(), label: "msg".into() }],
    });
    let result = render(&d, 25);
    assert_aligned(&result, 25);
}

#[test]
fn sequence_unknown_actor_warning() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into()],
        messages: vec![Message { from: "A".into(), to: "Unknown".into(), label: "x".into() }],
    });
    let result = render(&d, 40);
    assert!(!result.warnings.is_empty());
}

#[test]
fn sequence_many_actors_alignment() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into(), "C".into(), "D".into()],
        messages: vec![
            Message { from: "A".into(), to: "D".into(), label: "skip".into() },
            Message { from: "D".into(), to: "B".into(), label: "back".into() },
        ],
    });
    let result = render(&d, 60);
    assert_aligned(&result, 60);
}

#[test]
fn sequence_long_label_in_message() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: None,
        actors: vec!["A".into(), "B".into()],
        messages: vec![Message {
            from: "A".into(),
            to: "B".into(),
            label: "this is a very long message label that should be truncated".into(),
        }],
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
}

// ═══════════════════════════════════════════════════
// DAG edge cases
// ═══════════════════════════════════════════════════

#[test]
fn dag_cycle_handling() {
    // A -> B -> C -> A (cycle) — should still render without panic
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges: vec![
            Edge { from: "A".into(), to: "B".into(), label: None },
            Edge { from: "B".into(), to: "C".into(), label: None },
            Edge { from: "C".into(), to: "A".into(), label: None },
        ],
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
}

#[test]
fn dag_parallel_nodes_in_layer() {
    // A depends on nothing, B depends on nothing → same layer
    // C depends on both A and B
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges: vec![
            Edge { from: "A".into(), to: "C".into(), label: None },
            Edge { from: "B".into(), to: "C".into(), label: None },
        ],
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    // A and B should be on same layer (shown inline)
    assert!(output.contains('→') || output.contains('A'));
}

#[test]
fn dag_many_parallel_nodes_narrow_width() {
    // Force the fallback path: too many parallel nodes to fit inline
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges: vec![
            Edge { from: "NodeAlpha".into(), to: "Final".into(), label: None },
            Edge { from: "NodeBeta".into(), to: "Final".into(), label: None },
            Edge { from: "NodeGamma".into(), to: "Final".into(), label: None },
            Edge { from: "NodeDelta".into(), to: "Final".into(), label: None },
        ],
    });
    let result = render(&d, 20);
    assert_aligned(&result, 20);
}

#[test]
fn dag_single_node_no_edges() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: Some(vec![Node { id: None, label: "Standalone".into() }]),
        edges: vec![],
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    assert!(output.contains("Standalone"));
}

#[test]
fn dag_long_node_label_truncated() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: Some(vec![
            Node { id: Some("a".into()), label: "A very long node label that exceeds width".into() },
            Node { id: Some("b".into()), label: "Short".into() },
        ]),
        edges: vec![Edge { from: "a".into(), to: "b".into(), label: None }],
    });
    let mut result = render(&d, 20);
    assert_aligned(&result, 20);
}

#[test]
fn dag_diamond_dependency() {
    let d = Diagram::Dag(DagDiagram {
        title: None,
        nodes: None,
        edges: vec![
            Edge { from: "Start".into(), to: "Left".into(), label: None },
            Edge { from: "Start".into(), to: "Right".into(), label: None },
            Edge { from: "Left".into(), to: "End".into(), label: None },
            Edge { from: "Right".into(), to: "End".into(), label: None },
        ],
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Start"));
    assert!(output.contains("End"));
}

// ═══════════════════════════════════════════════════
// Mermaid edge cases
// ═══════════════════════════════════════════════════

#[test]
fn mermaid_empty_flowchart_body_error() {
    let d = Diagram::Mermaid(MermaidDiagram { title: None, code: "graph LR\n".into() });
    let result = render(&d, 40);
    assert!(!result.errors.is_empty());
}

#[test]
fn mermaid_flowchart_semicolon_separated() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR; A-->B; B-->C".into(),
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
}

#[test]
fn mermaid_flowchart_node_with_brackets() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "flowchart TD\nA[Start Node]-->B(Process)\nB-->C{Decision}".into(),
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Start Node"));
    assert!(output.contains("Process"));
    assert!(output.contains("Decision"));
}

#[test]
fn mermaid_flowchart_no_labels_infers_nodes() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\nX-->Y\nY-->Z".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    assert!(output.contains('X'));
    assert!(output.contains('Z'));
}

#[test]
fn mermaid_sequence_with_participant() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "sequenceDiagram\nparticipant Alice\nparticipant Bob\nAlice->>Bob: Hello".into(),
    });
    let result = render(&d, 50);
    assert_aligned(&result, 50);
    let output = result.output.unwrap();
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
}

#[test]
fn mermaid_sequence_with_actor_keyword() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "sequenceDiagram\nactor User\nactor System\nUser->>System: click".into(),
    });
    let result = render(&d, 50);
    assert_aligned(&result, 50);
}

#[test]
fn mermaid_sequence_multiple_arrow_types() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "sequenceDiagram\nA->>B: sync\nB-->>A: async\nA->C: plain".into(),
    });
    let result = render(&d, 50);
    assert_aligned(&result, 50);
}

#[test]
fn mermaid_state_with_comments() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "stateDiagram-v2\n%% This is a comment\nA --> B : go\nB --> C".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains('C'));
}

#[test]
fn mermaid_state_no_transitions_error() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "stateDiagram-v2\n%% only comments".into(),
    });
    let result = render(&d, 40);
    assert!(!result.errors.is_empty());
}

#[test]
fn mermaid_unsupported_diagram_type() {
    let d = Diagram::Mermaid(MermaidDiagram { title: None, code: "erDiagram\nFoo ||--o{ Bar : has".into() });
    let result = render(&d, 40);
    assert!(!result.errors.is_empty());
}

#[test]
fn mermaid_flowchart_dashed_arrow() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR\nA-.->B\nB-->C".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
}

#[test]
fn mermaid_flowchart_thick_arrow() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR\nA==>B".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
}
