use std::io::Read;

use drawz_core::schema::DiagramInput;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli_width: Option<u16> = args.iter()
        .position(|a| a == "--width")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok());

    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("error: failed to read stdin: {e}");
        std::process::exit(1);
    }

    let diagram_input: DiagramInput = match serde_json::from_str(&input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: invalid diagram JSON: {e}");
            std::process::exit(1);
        }
    };

    let width = cli_width.unwrap_or(diagram_input.width);
    let result = drawz_core::render(&diagram_input.diagram, width);

    if let Some(output) = &result.output {
        println!("{output}");
    }
    for err in &result.errors {
        eprintln!("error: {err}");
    }
    for warn in &result.warnings {
        eprintln!("warning: {warn}");
    }

    std::process::exit(i32::from(!result.errors.is_empty()));
}
