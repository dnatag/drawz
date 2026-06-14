use crate::measure::pad_right;
use crate::result::RenderContext;
use crate::schema::FreeformDiagram;

/// Render freeform: pad each line to `inner_width`, fixing alignment.
///
/// # Errors
///
/// Returns an error if neither `content` nor `lines` is provided, or if content is empty.
pub fn render(diagram: &FreeformDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    let raw_lines: Vec<&str> = if let Some(content) = &diagram.content {
        content.lines().collect()
    } else if let Some(lines) = &diagram.lines {
        lines.iter().map(String::as_str).collect()
    } else {
        return Err("freeform requires 'content' or 'lines' field".to_string());
    };

    if raw_lines.is_empty() {
        return Err("freeform content is empty".to_string());
    }

    let out: Vec<String> = raw_lines
        .iter()
        .map(|line| pad_right(line, ctx.inner_width))
        .collect();

    Ok(out)
}
