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
