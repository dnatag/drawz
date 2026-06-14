use crate::measure::{display_width, pad_right};

/// Wrap pre-padded content lines in a Unicode box with optional title.
///
/// All input lines must have `display_width` == `inner_width`.
/// Output lines will have `display_width` == `inner_width` + 4.
#[must_use]
pub fn frame_box(lines: &[String], title: Option<&str>, total_width: u16) -> Vec<String> {
    let tw = total_width as usize;
    let inner = tw.saturating_sub(4);
    let mut out = Vec::with_capacity(lines.len() + 4);

    out.push(format!("┌{}┐", "─".repeat(tw.saturating_sub(2))));

    if let Some(t) = title {
        let padded = pad_right(t, inner);
        out.push(format!("│ {padded} │"));
        out.push(format!("│ {} │", pad_right("", inner)));
    }

    for line in lines {
        let w = display_width(line);
        if w == inner {
            out.push(format!("│ {line} │"));
        } else {
            out.push(format!("│ {} │", pad_right(line, inner)));
        }
    }

    out.push(format!("└{}┘", "─".repeat(tw.saturating_sub(2))));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::pad_right;

    #[test]
    fn basic_frame() {
        let lines = vec![pad_right("hello", 10)];
        let framed = frame_box(&lines, None, 14);
        assert_eq!(framed[0], "┌────────────┐");
        assert_eq!(framed[1], "│ hello      │");
        assert_eq!(framed[2], "└────────────┘");
        for line in &framed {
            assert_eq!(display_width(line), 14);
        }
    }

    #[test]
    fn frame_with_title() {
        let lines = vec![pad_right("content", 10)];
        let framed = frame_box(&lines, Some("Title"), 14);
        assert_eq!(framed[1], "│ Title      │");
        assert_eq!(framed[2], "│            │");
        assert_eq!(framed[3], "│ content    │");
    }

    #[test]
    fn frame_empty_lines() {
        let lines: Vec<String> = vec![];
        let framed = frame_box(&lines, None, 20);
        assert_eq!(framed.len(), 2);
        assert_eq!(display_width(&framed[0]), 20);
    }

    #[test]
    fn frame_minimum_width() {
        let lines = vec![String::new()];
        let framed = frame_box(&lines, None, 4);
        assert_eq!(display_width(&framed[0]), 4);
    }

    #[test]
    fn frame_all_lines_same_width() {
        let lines = vec![
            pad_right("short", 20),
            pad_right("a longer line here", 20),
            pad_right("中文内容", 20),
        ];
        let framed = frame_box(&lines, None, 24);
        for line in &framed {
            assert_eq!(display_width(line), 24, "misaligned: {line:?}");
        }
    }

    #[test]
    fn frame_repads_short_lines() {
        // Lines shorter than inner_width get re-padded
        let lines = vec!["short".to_string(), "also short".to_string()];
        let framed = frame_box(&lines, None, 24);
        for line in &framed {
            assert_eq!(display_width(line), 24, "misaligned: {line:?}");
        }
    }
}
