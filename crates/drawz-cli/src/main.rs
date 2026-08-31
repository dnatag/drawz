mod mcp;
mod render;

use std::io::Read;

use clap::{Parser, Subcommand};
use drawz_core::schema::DiagramInput;

use render::RenderType;

/// The rendering guarantee layer for AI agent terminal output.
#[derive(Parser)]
#[command(name = "drawz", version, about)]
struct Cli {
    /// Maximum output width in characters
    #[arg(long, short = 'w', global = true)]
    width: Option<u16>,

    /// Diagram JSON (reads from stdin if omitted)
    #[arg(value_name = "JSON")]
    json: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start MCP server (JSON-RPC over stdio)
    Mcp,

    /// Render a diagram from type-specific flags or JSON
    #[command(subcommand)]
    Render(RenderInput),
}

/// Input source for `drawz render`: either a type subcommand or raw JSON.
#[derive(Subcommand)]
pub enum RenderInput {
    /// Render from raw JSON (positional or stdin)
    #[command(name = "json")]
    Json {
        /// Diagram JSON string (reads from stdin if omitted)
        input: Option<String>,
    },

    #[command(flatten)]
    Typed(RenderType),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Mcp) => {
            if let Err(e) = mcp::run().await {
                eprintln!("error: MCP server failed: {e}");
                std::process::exit(1);
            }
        }
        Some(Command::Render(input)) => {
            let diagram = match input {
                RenderInput::Typed(render_type) => render::build_diagram(render_type),
                RenderInput::Json { input } => parse_json_input(input),
            };
            match diagram {
                Ok(d) => render_and_output(d, cli.width),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            let diagram = parse_json_input(cli.json);
            match diagram {
                Ok(d) => render_and_output(d, cli.width),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn parse_json_input(input: Option<String>) -> Result<drawz_core::schema::Diagram, String> {
    let raw = match input {
        Some(s) => s,
        None => read_stdin()?,
    };
    let diagram_input: DiagramInput =
        serde_json::from_str(&raw).map_err(|e| format!("invalid diagram JSON: {e}"))?;
    Ok(diagram_input.diagram)
}

fn read_stdin() -> Result<String, String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    Ok(buf)
}

fn detect_width() -> u16 {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0)
        .unwrap_or(120)
}

fn render_and_output(diagram: drawz_core::schema::Diagram, width_override: Option<u16>) {
    let width = width_override.unwrap_or_else(detect_width);
    let result = drawz_core::render(&diagram, width);
    output_result(result);
}

fn output_result(result: drawz_core::RenderResult) {
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
