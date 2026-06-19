use crate::measure::pad_right;
use crate::result::RenderContext;
use crate::schema::FreeformDiagram;

/// Unescape common escape sequences that arrive as literals from JSON transport.
fn unescape_ansi(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\x1b", "\x1b")
        .replace("\\u001b", "\x1b")
        .replace("\\033", "\x1b")
        .replace("\\e", "\x1b")
}

/// Render freeform: pad each line to `inner_width`, fixing alignment.
/// Validates box-drawing consistency and warns if hand-drawn boxes are misaligned.
///
/// # Errors
///
/// Returns an error if neither `content` nor `lines` is provided, or if content is empty.
pub(crate) fn render(diagram: &FreeformDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    let raw_lines: Vec<String> = if let Some(content) = &diagram.content {
        let unescaped = unescape_ansi(content);
        unescaped.lines().map(String::from).collect()
    } else if let Some(lines) = &diagram.lines {
        lines.iter().map(|l| unescape_ansi(l)).collect()
    } else {
        return Err("freeform requires 'content' or 'lines' field".to_string());
    };

    if raw_lines.is_empty() {
        return Err("freeform content is empty".to_string());
    }

    // Validate box-drawing alignment
    validate_box_alignment(&raw_lines, ctx);

    // Check for truncation and warn
    let has_truncation = raw_lines.iter().any(|l| crate::measure::display_width(l) > ctx.inner_width);
    if has_truncation {
        ctx.warnings.push("some lines truncated to fit width".to_string());
    }

    let out: Vec<String> = raw_lines
        .iter()
        .map(|line| pad_right(line, ctx.inner_width))
        .collect();

    Ok(out)
}

/// Detect misaligned box-drawing: if lines contain border characters,
/// check that border lines match content line widths.
fn validate_box_alignment(lines: &[String], ctx: &mut RenderContext) {
    use crate::measure::display_width;

    let box_chars = "┌┐└┘┬┴├┤┼─│═║╔╗╚╝╠╣╦╩╬";
    let has_boxes = lines.iter().any(|l| l.chars().any(|c| box_chars.contains(c)));
    if !has_boxes {
        return;
    }

    // Check if all lines with box chars have consistent display widths
    let box_line_widths: Vec<usize> = lines
        .iter()
        .filter(|l| l.chars().any(|c| box_chars.contains(c)))
        .map(|l| display_width(l))
        .collect();

    if box_line_widths.len() >= 2 {
        let first = box_line_widths[0];
        if box_line_widths.iter().any(|&w| w != first) {
            ctx.warnings.push(
                "box-drawing lines have inconsistent widths — diagram may be misaligned".into(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::display_width;
    use crate::result::RenderContext;
    use crate::schema::FreeformDiagram;

    fn ctx(width: usize) -> RenderContext {
        RenderContext { inner_width: width, total_width: u16::try_from(width).unwrap(), warnings: Vec::new() }
    }

    #[test]
    fn should_pad_all_lines_when_content_provided() {
        let d = FreeformDiagram { title: None, content: Some("short\nlonger line".into()), lines: None };
        let lines = render(&d, &mut ctx(20)).unwrap();
        assert_eq!(lines.len(), 2);
        for l in &lines { assert_eq!(display_width(l), 20); }
    }

    #[test]
    fn should_pad_all_lines_when_lines_field_used() {
        let d = FreeformDiagram { title: None, content: None, lines: Some(vec!["a".into(), "bb".into()]) };
        let lines = render(&d, &mut ctx(10)).unwrap();
        assert_eq!(lines.len(), 2);
        for l in &lines { assert_eq!(display_width(l), 10); }
    }

    #[test]
    fn should_return_error_when_no_content_or_lines() {
        let d = FreeformDiagram { title: None, content: None, lines: None };
        assert!(render(&d, &mut ctx(20)).is_err());
    }

    #[test]
    fn should_return_error_when_content_empty() {
        let d = FreeformDiagram { title: None, content: Some(String::new()), lines: None };
        assert!(render(&d, &mut ctx(20)).is_err());
    }

    #[test]
    fn should_truncate_when_content_exceeds_width() {
        let d = FreeformDiagram { title: None, content: Some("x".repeat(50)), lines: None };
        let lines = render(&d, &mut ctx(10)).unwrap();
        for l in &lines { assert_eq!(display_width(l), 10); }
    }
}
