use drawz_core::measure::display_width;
use drawz_core::render;
use drawz_core::schema::{Diagram, FreeformDiagram, TableDiagram};

// === freeform integration ===

#[test]
fn freeform_missing_content_and_lines() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: None,
        lines: None,
    });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
    assert!(result.output.is_none());
}

#[test]
fn freeform_empty_content() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some(String::new()),
        lines: None,
    });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
}

#[test]
fn freeform_single_line_alignment() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some("hello".to_string()),
        lines: None,
    });
    let result = render(&d, 20);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 20, "misaligned: {line:?}");
    }
}

#[test]
fn freeform_unicode_content_alignment() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: Some("A ──► B\n  │\n  ▼\n  C".to_string()),
        lines: None,
    });
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 30, "misaligned: {line:?}");
    }
}

#[test]
fn freeform_with_title_alignment() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: Some("My Title".to_string()),
        content: Some("content here".to_string()),
        lines: None,
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
}

// === table integration ===

#[test]
fn table_empty_headers_error() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec![],
        rows: vec![],
    });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
}

#[test]
fn table_no_rows_alignment() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into()],
        rows: vec![],
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    assert_eq!(output.lines().count(), 2);
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
}

#[test]
fn table_single_column_alignment() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["Name".into()],
        rows: vec![vec!["Alice".into()], vec!["Bob".into()]],
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
}

#[test]
fn table_width_too_narrow_error() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into(), "F".into(), "G".into(), "H".into()],
        rows: vec![],
    });
    let result = render(&d, 10);
    assert!(!result.errors.is_empty());
}

#[test]
fn table_cjk_cells_alignment() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["名前".into(), "状態".into()],
        rows: vec![vec!["田中".into(), "完了".into()]],
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
}

#[test]
fn table_row_fewer_cells_than_headers() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into(), "C".into()],
        rows: vec![vec!["only one".into()]],
    });
    let result = render(&d, 40);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 40, "misaligned: {line:?}");
    }
}

// === unimplemented types return errors ===

#[test]
fn unimplemented_types_return_error() {
    use drawz_core::schema::*;

    let cases: Vec<Diagram> = vec![
        Diagram::Flow(FlowDiagram { title: None, steps: Some(vec![FlowStep::Label("A".into())]), nodes: None, edges: None }),
        Diagram::State(StateDiagram { title: None, states: None, transitions: vec![] }),
        Diagram::Dag(DagDiagram { title: None, nodes: None, edges: vec![] }),
        Diagram::Tree(TreeDiagram { title: None, root: None, indent: Some("a\n  b".into()) }),
        Diagram::Sequence(SequenceDiagram { title: None, actors: vec!["A".into()], messages: vec![] }),
        Diagram::Mermaid(MermaidDiagram { title: None, code: "graph LR; A-->B".into() }),
    ];

    for d in &cases {
        let result = render(d, 80);
        assert!(!result.errors.is_empty(), "expected error for unimplemented type");
        assert!(result.output.is_none());
    }
}

// === freeform uses lines field ===

#[test]
fn freeform_lines_field() {
    let d = Diagram::Freeform(FreeformDiagram {
        title: None,
        content: None,
        lines: Some(vec!["line one".into(), "line two".into()]),
    });
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 30, "misaligned: {line:?}");
    }
}

// === table column shrink path ===

#[test]
fn table_columns_shrink_when_too_wide() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["VeryLongHeader".into(), "AnotherLongOne".into(), "ThirdColumn".into()],
        rows: vec![vec!["data".into(), "more data here".into(), "stuff".into()]],
    });
    // Width 30 forces column shrinking
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    assert!(!result.fit); // should have warnings about truncation
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 30, "misaligned: {line:?}");
    }
}

// === table cell truncation ===

#[test]
fn table_cell_content_truncated() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into()],
        rows: vec![vec!["this is a very long cell value that must be truncated".into(), "short".into()]],
    });
    let result = render(&d, 25);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 25, "misaligned: {line:?}");
    }
}
