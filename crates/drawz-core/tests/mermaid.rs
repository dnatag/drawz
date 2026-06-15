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
fn mermaid_unsupported_type_returns_error() {
    let d = Diagram::Mermaid(MermaidDiagram { title: None, code: "pie\n  \"A\": 50\n  \"B\": 50".into() });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
    assert!(result.output.is_none());
}

#[test]
fn mermaid_flowchart_renders_as_flow() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR\nA[Request]-->B[Validate]\nB-->C[Process]\nC-->D[Response]".into(),
    });
    let result = render(&d, 40);
    assert_and_print("Mermaid: Flowchart", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Request"));
    assert!(output.contains("Response"));
}

#[test]
fn mermaid_sequence_renders_as_sequence() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi back".into(),
    });
    let result = render(&d, 50);
    assert_and_print("Mermaid: Sequence", &result, 50);
    let output = result.output.unwrap();
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
}

#[test]
fn mermaid_state_renders_as_state() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "stateDiagram-v2\nIdle --> Active : start\nActive --> Done : finish".into(),
    });
    let result = render(&d, 40);
    assert_and_print("Mermaid: State Diagram", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Idle"));
    assert!(output.contains("Active"));
    assert!(output.contains("Done"));
}

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

#[test]
fn mermaid_title_passthrough() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: Some("My Flow".into()),
        code: "graph LR\nA-->B".into(),
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("My Flow"));
}

#[test]
fn mermaid_escaped_newline_handling() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR\\nA-->B\\nB-->C".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains('C'));
}

#[test]
fn mermaid_chained_three_nodes() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph LR; A-->B-->C-->D".into(),
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
    let output = result.output.unwrap();
    assert!(output.contains('A'));
    assert!(output.contains('D'));
}

#[test]
fn mermaid_multiline_with_mixed_separators() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "graph TD\nA-->B;B-->C\nC-->D".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
}

#[test]
fn mermaid_sequence_colon_in_label() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "sequenceDiagram\nA->>B: GET /api/v1:8080/users".into(),
    });
    let result = render(&d, 60);
    assert_aligned(&result, 60);
}

#[test]
fn mermaid_state_spaces_around_arrow() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "stateDiagram-v2\nA-->B\nC --> D\nE  -->  F".into(),
    });
    let result = render(&d, 30);
    assert_aligned(&result, 30);
}

#[test]
fn mermaid_flowchart_keyword_variants() {
    for keyword in &["graph LR", "graph TD", "flowchart LR", "flowchart TD"] {
        let code = format!("{keyword}\nA-->B");
        let d = Diagram::Mermaid(MermaidDiagram { title: None, code });
        let result = render(&d, 30);
        assert_aligned(&result, 30);
    }
}
