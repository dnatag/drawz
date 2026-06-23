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
    let first_w = output.lines().next().map(display_width).unwrap_or(0);
    for line in output.lines() {
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
fn component_two_groups_with_connections() {
    let d = Diagram::Component(ComponentDiagram {
        title: Some("Process Architecture".into()),
        groups: vec![
            ComponentGroup {
                label: "Parent Process".into(),
                nodes: vec!["Scheduler".into(), "DAG Engine".into()],
                chains: vec![],
                edges: vec![],
            },
            ComponentGroup {
                label: "Child Process".into(),
                nodes: vec!["Sandbox".into(), "Runtime".into()],
                chains: vec![],
                edges: vec![],
            },
        ],
        connections: vec![
            Connection {
                from: "Scheduler".into(),
                to: "Sandbox".into(),
                label: Some("spawn".into()),
            },
            Connection {
                from: "DAG Engine".into(),
                to: "Runtime".into(),
                label: Some("pipe".into()),
            },
        ],
    });
    let result = render(&d, 60);
    assert_and_print("Component: Parent/Child Architecture", &result, 60);
    let output = result.output.unwrap();
    assert!(output.contains("Parent Process"));
    assert!(output.contains("Child Process"));
    assert!(output.contains("Scheduler"));
    assert!(output.contains("Sandbox"));
    assert!(output.contains("spawn"));
    assert!(output.contains("pipe"));
    assert!(output.contains("→"));
}

#[test]
fn component_single_group() {
    let d = Diagram::Component(ComponentDiagram {
        title: None,
        groups: vec![ComponentGroup {
            label: "Services".into(),
            nodes: vec!["API".into(), "Worker".into(), "DB".into()],
            chains: vec![],
            edges: vec![],
        }],
        connections: vec![],
    });
    let result = render(&d, 40);
    assert_and_print("Component: Single Group", &result, 40);
    let output = result.output.unwrap();
    assert!(output.contains("Services"));
    assert!(output.contains("API"));
    assert!(output.contains("Worker"));
    assert!(output.contains("DB"));
}

#[test]
fn component_mermaid_subgraph_becomes_component() {
    let d = Diagram::Mermaid(MermaidDiagram {
        title: None,
        code: "flowchart TD\n  subgraph Frontend\n    A[App]\n    B[Client]\n  end\n  subgraph Backend\n    C[API]\n    D[DB]\n  end\n  B-->|REST|C".into(),
    });
    let result = render(&d, 60);
    assert_and_print("Component: Mermaid subgraph bridge", &result, 60);
    let output = result.output.unwrap();
    assert!(output.contains("Frontend"));
    assert!(output.contains("Backend"));
    assert!(output.contains("REST"));
}

#[test]
fn component_empty_groups_error() {
    let d = Diagram::Component(ComponentDiagram {
        title: None,
        groups: vec![],
        connections: vec![],
    });
    let result = render(&d, 40);
    assert!(!result.errors.is_empty());
}

#[test]
fn component_alignment() {
    let d = Diagram::Component(ComponentDiagram {
        title: None,
        groups: vec![
            ComponentGroup {
                label: "Left".into(),
                nodes: vec!["Short".into(), "A Longer Name".into()],
                chains: vec![],
                edges: vec![],
            },
            ComponentGroup {
                label: "Right".into(),
                nodes: vec!["X".into(), "Y".into()],
                chains: vec![],
                edges: vec![],
            },
        ],
        connections: vec![Connection {
            from: "Short".into(),
            to: "X".into(),
            label: Some("link".into()),
        }],
    });
    let result = render(&d, 60);
    assert_and_print("Component: Alignment with mixed widths", &result, 60);
}

#[test]
fn component_chains_single_group() {
    let d = Diagram::Component(ComponentDiagram {
        title: None,
        groups: vec![ComponentGroup {
            label: "drawz-core".into(),
            nodes: vec![],
            chains: vec![
                vec!["schema.rs".into(), "render.rs".into(), "frame.rs".into()],
                vec![
                    "mermaid/parse.rs".into(),
                    "Diagram".into(),
                    "render.rs".into(),
                ],
            ],
            edges: vec![],
        }],
        connections: vec![],
    });
    let result = render(&d, 60);
    assert_and_print("Component: Chains single group", &result, 60);
    let output = result.output.unwrap();
    assert!(output.contains("schema.rs → render.rs → frame.rs"));
    assert!(output.contains("mermaid/parse.rs → Diagram → render.rs"));
    assert!(output.contains("drawz-core"));
}

#[test]
fn component_chains_with_cross_edges() {
    let d = Diagram::Component(ComponentDiagram {
        title: None,
        groups: vec![ComponentGroup {
            label: "Pipeline".into(),
            nodes: vec![],
            chains: vec![
                vec!["Parse".into(), "Transform".into(), "Emit".into()],
                vec!["Validate".into(), "Optimize".into()],
            ],
            edges: vec![Connection {
                from: "Transform".into(),
                to: "Validate".into(),
                label: Some("check".into()),
            }],
        }],
        connections: vec![],
    });
    let result = render(&d, 60);
    assert_and_print("Component: Chains with cross-edge", &result, 60);
    let output = result.output.unwrap();
    assert!(output.contains("Parse → Transform → Emit"));
    assert!(output.contains("Validate → Optimize"));
    assert!(output.contains("├─check─→"));
}

#[test]
fn component_chains_vertical_connector() {
    let d = Diagram::Component(ComponentDiagram {
        title: None,
        groups: vec![ComponentGroup {
            label: "Layers".into(),
            nodes: vec![],
            chains: vec![
                vec!["HTTP".into(), "Router".into()],
                vec!["Service".into(), "DB".into()],
            ],
            edges: vec![Connection {
                from: "Router".into(),
                to: "Service".into(),
                label: None,
            }],
        }],
        connections: vec![],
    });
    let result = render(&d, 50);
    assert_and_print("Component: Chains with vertical connector", &result, 50);
    let output = result.output.unwrap();
    assert!(output.contains("HTTP → Router"));
    assert!(output.contains("Service → DB"));
    assert!(output.contains('│'));
}

#[test]
fn component_stacked_groups_with_chains() {
    let d = Diagram::Component(ComponentDiagram {
        title: Some("Architecture".into()),
        groups: vec![
            ComponentGroup {
                label: "CLI".into(),
                nodes: vec![],
                chains: vec![vec!["main.rs".into(), "core".into()]],
                edges: vec![],
            },
            ComponentGroup {
                label: "Core".into(),
                nodes: vec![],
                chains: vec![
                    vec!["schema".into(), "render".into(), "frame".into()],
                    vec!["measure".into(), "pad_right".into()],
                ],
                edges: vec![Connection {
                    from: "render".into(),
                    to: "measure".into(),
                    label: Some("uses".into()),
                }],
            },
        ],
        connections: vec![Connection {
            from: "core".into(),
            to: "schema".into(),
            label: Some("deserialize".into()),
        }],
    });
    let result = render(&d, 60);
    assert_and_print("Component: Stacked groups with chains", &result, 60);
    let output = result.output.unwrap();
    assert!(output.contains("CLI"));
    assert!(output.contains("Core"));
    assert!(output.contains("schema → render → frame"));
    assert!(output.contains("├─uses─→"));
    assert!(output.contains("deserialize"));
}
