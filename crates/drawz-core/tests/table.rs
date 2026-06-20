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
    assert_eq!(output.lines().count(), 4); // top + header + header_sep + bottom
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
        headers: vec![
            "A".into(),
            "B".into(),
            "C".into(),
            "D".into(),
            "E".into(),
            "F".into(),
            "G".into(),
            "H".into(),
        ],
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

#[test]
fn table_columns_shrink_when_too_wide() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec![
            "VeryLongHeader".into(),
            "AnotherLongOne".into(),
            "ThirdColumn".into(),
        ],
        rows: vec![vec!["data".into(), "more data here".into(), "stuff".into()]],
    });
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    assert!(!result.fit);
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 30, "misaligned: {line:?}");
    }
}

#[test]
fn table_cell_content_truncated() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["A".into(), "B".into()],
        rows: vec![vec![
            "this is a very long cell value that must be truncated".into(),
            "short".into(),
        ]],
    });
    let result = render(&d, 25);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 25, "misaligned: {line:?}");
    }
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
fn complex_table_many_columns() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec![
            "Renderer".into(),
            "Status".into(),
            "Framed".into(),
            "Tests".into(),
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
    assert_eq!(output.lines().count(), 17); // top + header + sep + 7*(row+sep) - 1 sep + bottom
}

#[test]
fn table_unicode_content_alignment() {
    let d = Diagram::Table(TableDiagram {
        title: None,
        headers: vec!["Name".into(), "Status".into()],
        rows: vec![
            vec!["日本語".into(), "✓".into()],
            vec!["café".into(), "⚡".into()],
            vec!["🎉 party".into(), "ok".into()],
        ],
    });
    let result = render(&d, 40);
    assert_aligned(&result, 40);
}
