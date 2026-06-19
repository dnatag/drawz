use crate::frame;
use crate::mermaid;
use crate::renderers;
use crate::result::{RenderContext, RenderResult};
use crate::schema::Diagram;

/// Render a diagram within the given width.
#[must_use]
pub fn render(diagram: &Diagram, width: u16) -> RenderResult {
    if width < 4 {
        return RenderResult {
            output: None,
            fit: false,
            errors: vec![format!("width {width} too small (minimum: 4)")],
            warnings: Vec::new(),
        };
    }

    let framed = matches!(
        diagram,
        Diagram::Freeform(_) | Diagram::Flow(_) | Diagram::State(_)
            | Diagram::Sequence(_) | Diagram::Dag(_)
    );

    // Use requested width as inner_width for layout hints, but renderers
    // that can render at natural size (horizontal flow, DAG) will overflow.
    // The post-processing step trims and re-aligns all lines.
    let inner_width = if framed { (width as usize).saturating_sub(4) } else { width as usize };
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
                Ok(mut parsed) => {
                    // Pass through the mermaid diagram's title
                    if d.title.is_some() {
                        set_title(&mut parsed, d.title.as_deref());
                    }
                    return render(&parsed, width);
                }
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
            // Check if any line exceeds the requested inner width
            let max_line_w = lines.iter()
                .map(|l| crate::measure::display_width(l.trim_end()))
                .max()
                .unwrap_or(0);

            let (output_lines, content_width) = if max_line_w > inner_width {
                // Content overflows — render at natural size (no truncation)
                let trimmed: Vec<String> = lines.iter()
                    .map(|l| l.trim_end().to_string())
                    .collect();
                let natural_width = trimmed.iter()
                    .map(|l| crate::measure::display_width(l))
                    .max()
                    .unwrap_or(0);
                let title_width = title.map(|t| crate::measure::display_width(t) + 4).unwrap_or(0);
                let final_width = natural_width.max(title_width);
                let aligned: Vec<String> = trimmed.iter()
                    .map(|l| crate::measure::pad_right(l, final_width))
                    .collect();
                if framed {
                    let fw = (final_width + 4).min(65535) as u16;
                    (frame::frame_box(&aligned, title, fw), final_width + 4)
                } else {
                    (aligned, final_width)
                }
            } else {
                // Content fits — use requested width (original behavior)
                if framed {
                    (frame::frame_box(&lines, title, width), width as usize)
                } else {
                    (lines, inner_width)
                }
            };

            let fit = content_width <= width as usize
                && ctx.warnings.iter().all(|w| w.starts_with("info:"));

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

fn set_title(diagram: &mut Diagram, title: Option<&str>) {
    let t = title.map(String::from);
    match diagram {
        Diagram::Freeform(d) => d.title = t,
        Diagram::Table(d) => d.title = t,
        Diagram::Flow(d) => d.title = t,
        Diagram::Tree(d) => d.title = t,
        Diagram::State(d) => d.title = t,
        Diagram::Sequence(d) => d.title = t,
        Diagram::Dag(d) => d.title = t,
        Diagram::Mermaid(d) => d.title = t,
    }
}
