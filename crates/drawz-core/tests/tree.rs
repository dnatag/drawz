use drawz_core::measure::display_width;
use drawz_core::render;
use drawz_core::schema::*;

fn assert_aligned(result: &drawz_core::RenderResult, _width: u16) {
    assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);
    let output = result.output.as_ref().expect("expected output");
    for line in output.lines() {
        let first_w = output.lines().next().map(display_width).unwrap_or(0); assert_eq!(display_width(line), first_w, "misaligned: {line:?}");
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
fn tree_indent_alignment() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        root: None,
        indent: Some("src\n  main.rs\n  lib.rs\n  utils\n    helper.rs".into()),
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
    assert!(output.contains("├──") || output.contains("└──"));
}

#[test]
fn tree_node_alignment() {
    let d = Diagram::Tree(TreeDiagram {
        title: None,
        root: Some(TreeNode {
            label: "root".into(),
            children: vec![
                TreeNode { label: "child1".into(), children: vec![] },
                TreeNode { label: "child2".into(), children: vec![
                    TreeNode { label: "grandchild".into(), children: vec![] },
                ] },
            ],
        }),
        indent: None,
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
}

#[test]
fn tree_missing_input_error() {
    let d = Diagram::Tree(TreeDiagram { title: None, root: None, indent: None });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
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
