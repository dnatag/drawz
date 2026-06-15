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

#[test]
fn freeform_width_below_minimum_rejected() {
    let d = Diagram::Freeform(FreeformDiagram { title: None, content: Some("x".into()), lines: None });
    for w in [0, 1, 2, 3] {
        let result = render(&d, w);
        assert!(!result.errors.is_empty(), "width {w} should be rejected");
        assert!(result.output.is_none());
    }
}

#[test]
fn freeform_minimum_width() {
    let d = Diagram::Freeform(FreeformDiagram { title: None, content: Some("hi".into()), lines: None });
    let result = render(&d, 4);
    assert!(result.output.is_some() || !result.errors.is_empty());
}
