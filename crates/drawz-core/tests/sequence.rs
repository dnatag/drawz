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
