mod mcp;

use std::io::Read;

use clap::{Parser, Subcommand};
use drawz_core::schema::DiagramInput;

/// The rendering guarantee layer for AI agent terminal output.
#[derive(Parser)]
#[command(name = "drawz", version, about)]
struct Cli {
    /// Maximum output width in characters
    #[arg(long, short = 'w')]
    width: Option<u16>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start MCP server (JSON-RPC over stdio)
    Mcp,
}

fn main() {
    let cli = Cli::parse();

    if let Some(Command::Mcp) = cli.command {
        mcp::run();
        return;
    }

    // Default: pipe mode (stdin JSON → stdout diagram)
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

    let width = cli.width.unwrap_or(diagram_input.width);
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
