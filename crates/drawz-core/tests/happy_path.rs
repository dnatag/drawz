use drawz_core::measure::display_width;
use drawz_core::render;
use drawz_core::schema::*;

/// Helper: assert every line of output has exactly `width` display width.
fn assert_aligned(result: &drawz_core::RenderResult, width: u16) {
    assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);
    let output = result.output.as_ref().expect("expected output");
    for line in output.lines() {
        assert_eq!(
            display_width(line),
            width as usize,
            "misaligned: {line:?}"
        );
    }
}

/// Helper: assert alignment and print the diagram for human review.
/// Run with `cargo test -- --nocapture` to see output.
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

// ═══════════════════════════════════════════════════
// Happy-path tests: verify rendered output content
// ═══════════════════════════════════════════════════

#[test]
fn flow_linear_renders_boxes_and_arrows() {
    let d = Diagram::Flow(FlowDiagram {
        title: Some("Pipeline".into()),
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
fn state_renders_rounded_boxes_with_labels() {
    let d = Diagram::State(StateDiagram {
        title: Some("Order Lifecycle".into()),
        states: None,
        transitions: vec![
            Edge { from: "Created".into(), to: "Paid".into(), label: Some("pay".into()) },
            Edge { from: "Paid".into(), to: "Shipped".into(), label: Some("ship".into()) },
            Edge { from: "Shipped".into(), to: "Delivered".into(), label: None },
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
fn tree_indent_renders_connectors() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        root: None,
        indent: Some("project\n  src\n    main.rs\n    lib.rs\n  tests\n    integration.rs\n  Cargo.toml".into()),
    });
    let result = render(&d, 40);
    assert_and_print("Tree: Indent-based", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("project"));
    assert!(output.contains("├──"));
    assert!(output.contains("└──"));
    assert!(output.contains("src"));
    assert!(output.contains("main.rs"));
    assert!(output.contains("Cargo.toml"));
}

#[test]
fn tree_node_renders_hierarchy() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        indent: None,
        root: Some(TreeNode {
            label: "app".into(),
            children: vec![
                TreeNode {
                    label: "components".into(),
                    children: vec![
                        TreeNode { label: "Button.tsx".into(), children: vec![] },
                        TreeNode { label: "Modal.tsx".into(), children: vec![] },
                    ],
                },
                TreeNode { label: "index.ts".into(), children: vec![] },
            ],
        }),
    });
    let result = render(&d, 35);
    assert_and_print("Tree: Structured Node", &result, 35);
    let output = result.output.unwrap();
    assert!(output.contains("app"));
    assert!(output.contains("├── components"));
    assert!(output.contains("└── index.ts"));
    assert!(output.contains("Button.tsx"));
}

#[test]
fn table_renders_headers_separator_rows() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["Name".into(), "Role".into(), "Status".into()],
        rows: vec![
            vec!["Alice".into(), "Engineer".into(), "Active".into()],
            vec!["Bob".into(), "Designer".into(), "Away".into()],
        ],
    });
    let result = render(&d, 50);
    assert_and_print("Table: Team Roster", &result, 50);
    let output = result.output.unwrap();
    assert!(output.contains("Name"));
    assert!(output.contains("─┼─"));
    assert!(output.contains("Alice"));
    assert!(output.contains("Engineer"));
}

#[test]
fn freeform_preserves_content_structure() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: Some("Architecture".into()),
        content: Some("┌─────────┐    ┌─────────┐\n│  Client │───►│  Server │\n└─────────┘    └─────────┘".into()),
        lines: None,
    });
    let result = render(&d, 50);
    assert_and_print("Freeform: Hand-drawn Architecture", &result, 50);
    let output = result.output.unwrap();
    assert!(output.contains("Architecture"));
    assert!(output.contains("Client"));
    assert!(output.contains("Server"));
}

// ═══════════════════════════════════════════════════
// Complex integration: multi-component ASCII diagrams
// ═══════════════════════════════════════════════════

#[test]
fn complex_flow_with_many_steps() {
    let d = Diagram::Flow(FlowDiagram {
        title: Some("CI/CD Pipeline".into()),
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
fn complex_state_machine() {
    let d = Diagram::State(StateDiagram {
        title: Some("TCP Connection".into()),
        states: None,
        transitions: vec![
            Edge { from: "CLOSED".into(), to: "SYN_SENT".into(), label: Some("connect()".into()) },
            Edge { from: "SYN_SENT".into(), to: "ESTABLISHED".into(), label: Some("SYN+ACK".into()) },
            Edge { from: "ESTABLISHED".into(), to: "FIN_WAIT".into(), label: Some("close()".into()) },
            Edge { from: "FIN_WAIT".into(), to: "CLOSED".into(), label: Some("ACK".into()) },
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
fn complex_deep_tree() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        indent: None,
        root: Some(TreeNode {
            label: "drawz".into(),
            children: vec![
                TreeNode {
                    label: "crates".into(),
                    children: vec![
                        TreeNode {
                            label: "drawz-core".into(),
                            children: vec![
                                TreeNode {
                                    label: "src".into(),
                                    children: vec![
                                        TreeNode { label: "lib.rs".into(), children: vec![] },
                                        TreeNode { label: "render.rs".into(), children: vec![] },
                                        TreeNode {
                                            label: "renderers".into(),
                                            children: vec![
                                                TreeNode { label: "flow.rs".into(), children: vec![] },
                                                TreeNode { label: "state.rs".into(), children: vec![] },
                                                TreeNode { label: "tree.rs".into(), children: vec![] },
                                            ],
                                        },
                                    ],
                                },
                            ],
                        },
                        TreeNode {
                            label: "drawz-cli".into(),
                            children: vec![
                                TreeNode { label: "src".into(), children: vec![
                                    TreeNode { label: "main.rs".into(), children: vec![] },
                                ] },
                            ],
                        },
                    ],
                },
                TreeNode { label: "Cargo.toml".into(), children: vec![] },
            ],
        }),
    });
    let result = render(&d, 50);
    assert_and_print("Complex Tree: Project Layout", &result, 50);
    let output = result.output.unwrap();
    assert!(output.contains("drawz"));
    assert!(output.contains("├── crates"));
    assert!(output.contains("└── Cargo.toml"));
    assert!(output.contains("flow.rs"));
    assert!(output.contains("tree.rs"));
}

#[test]
fn complex_table_many_columns() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec![
            "Renderer".into(), "Status".into(), "Framed".into(), "Tests".into(),
        ],
        rows: vec![
            vec!["freeform".into(), "✓".into(), "yes".into(), "5".into()],
            vec!["table".into(), "✓".into(), "no".into(), "5".into()],
            vec!["tree".into(), "✓".into(), "no".into(), "4".into()],
            vec!["flow".into(), "✓".into(), "yes".into(), "5".into()],
            vec!["state".into(), "✓".into(), "yes".into(), "4".into()],
            vec!["sequence".into(), "—".into(), "yes".into(), "0".into()],
            vec!["dag".into(), "—".into(), "yes".into(), "0".into()],
        ],
    });
    let result = render(&d, 60);
    assert_and_print("Complex Table: Renderer Status", &result, 60);
    assert!(result.fit);
    let output = result.output.unwrap();
    assert!(output.contains("freeform"));
    assert!(output.contains("sequence"));
    assert_eq!(output.lines().count(), 9); // header + sep + 7 rows
}

#[test]
fn flow_graph_mode_with_edges() {
    let d = Diagram::Flow(FlowDiagram {
        title: None,
        steps: None,
        nodes: Some(vec![
            Node { id: Some("req".into()), label: "Request".into() },
            Node { id: Some("auth".into()), label: "Auth Check".into() },
            Node { id: Some("handler".into()), label: "Handler".into() },
            Node { id: Some("resp".into()), label: "Response".into() },
        ]),
        edges: Some(vec![
            Edge { from: "req".into(), to: "auth".into(), label: Some("validate".into()) },
            Edge { from: "auth".into(), to: "handler".into(), label: Some("authorized".into()) },
            Edge { from: "handler".into(), to: "resp".into(), label: None },
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
        ("Freeform@20", Diagram::Freeform(FreeformDiagram {
            title: None,
            content: Some("short\nlines".into()),
            lines: None,
        })),
        ("Flow@20", Diagram::Flow(FlowDiagram {
            title: None,
            steps: Some(vec![FlowStep::Label("A".into()), FlowStep::Label("B".into())]),
            nodes: None,
            edges: None,
        })),
        ("State@20", Diagram::State(StateDiagram {
            title: None,
            states: None,
            transitions: vec![Edge { from: "X".into(), to: "Y".into(), label: None }],
        })),
        ("Tree@20", Diagram::Tree(TreeDiagram {
            title: None,
            root: None,
            indent: Some("r\n  a\n  b".into()),
        })),
        ("Table@20", Diagram::Table(TableDiagram {
            title: None,
            headers: vec!["H".into()],
            rows: vec![vec!["v".into()]],
        })),
    ];

    for (label, d) in &cases {
        let result = render(d, width);
        assert_and_print(label, &result, width);
    }
}

// ═══════════════════════════════════════════════════
// Phase 3: Sequence, DAG, Mermaid
// ═══════════════════════════════════════════════════

#[test]
fn sequence_diagram_renders_actors_and_messages() {
    let d = Diagram::Sequence(SequenceDiagram {
        title: Some("Auth Flow".into()),
        actors: vec!["Client".into(), "Auth".into(), "API".into()],
        messages: vec![
            Message { from: "Client".into(), to: "Auth".into(), label: "login".into() },
            Message { from: "Auth".into(), to: "API".into(), label: "token".into() },
            Message { from: "API".into(), to: "Client".into(), label: "data".into() },
        ],
    });
    let result = render(&d, 60);
    assert_and_print("Sequence: Auth Flow", &result, 60);
    let output = result.output.unwrap();
    assert!(output.contains("Client"));
    assert!(output.contains("Auth"));
    assert!(output.contains("API"));
}

#[test]
fn dag_renders_layered_dependencies() {
    let d = Diagram::Dag(DagDiagram {
        title: Some("Build Graph".into()),
        nodes: Some(vec![
            Node { id: Some("parse".into()), label: "Parse".into() },
            Node { id: Some("lint".into()), label: "Lint".into() },
            Node { id: Some("compile".into()), label: "Compile".into() },
            Node { id: Some("link".into()), label: "Link".into() },
        ]),
        edges: vec![
            Edge { from: "parse".into(), to: "lint".into(), label: None },
            Edge { from: "parse".into(), to: "compile".into(), label: None },
            Edge { from: "lint".into(), to: "link".into(), label: None },
            Edge { from: "compile".into(), to: "link".into(), label: None },
        ],
    });
    let result = render(&d, 40);
    assert_and_print("DAG: Build Graph", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Parse"));
    assert!(output.contains("Link"));
    assert!(output.contains('▼'));
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
