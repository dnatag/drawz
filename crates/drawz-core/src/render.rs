use crate::frame;
use crate::mermaid;
use crate::renderers;
use crate::result::{RenderContext, RenderResult};
use crate::schema::Diagram;

/// Render a diagram within the given width.
#[must_use]
pub fn render(diagram: &Diagram, width: u16) -> RenderResult {
    let framed = matches!(
        diagram,
        Diagram::Freeform(_) | Diagram::Flow(_) | Diagram::State(_)
            | Diagram::Sequence(_) | Diagram::Dag(_) | Diagram::Mermaid(_)
    );

    let inner_width = if framed {
        (width as usize).saturating_sub(4)
    } else {
        width as usize
    };

    let mut ctx = RenderContext {
        inner_width,
        total_width: width,
        warnings: Vec::new(),
    };

    let title = extract_title(diagram);

    let lines = match diagram {
        Diagram::Freeform(d) => renderers::freeform::render(d, &mut ctx),
        Diagram::Table(d) => renderers::table::render(d, &mut ctx),
        Diagram::Flow(d) => renderers::flow::render(d, &mut ctx),
        Diagram::Tree(d) => renderers::tree::render(d, &mut ctx),
        Diagram::State(d) => renderers::state::render(d, &mut ctx),
        Diagram::Sequence(d) => renderers::sequence::render(d, &mut ctx),
        Diagram::Dag(d) => renderers::dag::render(d, &mut ctx),
        Diagram::Mermaid(d) => {
            match mermaid::parse(&d.code) {
                Ok(parsed) => return render(&parsed, width),
                Err(e) => Err(e),
            }
        }
    };

    match lines {
        Err(e) => RenderResult {
            output: None,
            fit: false,
            errors: vec![e],
            warnings: ctx.warnings,
        },
        Ok(lines) => {
            let output_lines = if framed {
                frame::frame_box(&lines, title, width)
            } else {
                lines
            };
            let fit = ctx.warnings.is_empty();
            RenderResult {
                output: Some(output_lines.join("\n")),
                fit,
                errors: Vec::new(),
                warnings: ctx.warnings,
            }
        }
    }
}

fn extract_title(diagram: &Diagram) -> Option<&str> {
    match diagram {
        Diagram::Freeform(d) => d.title.as_deref(),
        Diagram::Table(d) => d.title.as_deref(),
        Diagram::Flow(d) => d.title.as_deref(),
        Diagram::Tree(d) => d.title.as_deref(),
        Diagram::State(d) => d.title.as_deref(),
        Diagram::Sequence(d) => d.title.as_deref(),
        Diagram::Dag(d) => d.title.as_deref(),
        Diagram::Mermaid(d) => d.title.as_deref(),
    }
}
