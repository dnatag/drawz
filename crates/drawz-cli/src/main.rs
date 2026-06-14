use std::io::Read;

use drawz_core::schema::DiagramInput;
use drawz_core::render;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli_width: Option<u16> = args.iter()
        .position(|a| a == "--width")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok());

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("failed to read stdin");

    let diagram_input: DiagramInput = serde_json::from_str(&input).expect("invalid diagram JSON");

    // CLI --width overrides JSON width; JSON width overrides default (80)
    let width = cli_width.unwrap_or(diagram_input.width);
    println!("{}", render::render(&diagram_input.diagram, width));
}
