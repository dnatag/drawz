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
fn dag_cycle_handling() {
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
    assert!(output.contains('→') || output.contains('A'));
}

#[test]
fn dag_many_parallel_nodes_narrow_width() {
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
    let result = render(&d, 20);
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
