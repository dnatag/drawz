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
fn state_transitions_alignment() {
    let d = Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions: vec![
            Edge { from: "Idle".into(), to: "Running".into(), label: Some("start".into()) },
            Edge { from: "Running".into(), to: "Done".into(), label: None },
        ],
    });
    let result = render(&d, 30);
    assert!(result.errors.is_empty());
    let output = result.output.unwrap();
    for line in output.lines() {
        assert_eq!(display_width(line), 30, "misaligned: {line:?}");
    }
    assert!(output.contains("╭") && output.contains("╰"));
}

#[test]
fn state_empty_transitions_error() {
    let d = Diagram::State(StateDiagram {
        title: None,
        states: None,
        transitions: vec![],
    });
    let result = render(&d, 80);
    assert!(!result.errors.is_empty());
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
