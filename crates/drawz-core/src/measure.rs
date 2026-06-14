use unicode_width::UnicodeWidthChar;

/// Display width of a string, skipping ANSI escape sequences.
/// CJK = 2, normal = 1, combining = 0.
#[must_use]
pub fn display_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if ('\x40'..='\x7e').contains(&next) {
                        break;
                    }
                }
            }
        } else {
            width += c.width().unwrap_or(0);
        }
    }
    width
}

/// Pad string with spaces to reach exact target display width.
/// If already wider, truncates.
#[must_use]
pub fn pad_right(s: &str, target_width: usize) -> String {
    let w = display_width(s);
    if w >= target_width {
        if w > target_width {
            return truncate(s, target_width);
        }
        return s.to_string();
    }
    format!("{}{}", s, " ".repeat(target_width - w))
}

/// Truncate string to fit max display width, appending "…".
#[must_use]
pub fn truncate(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut width = 0;
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let ellipsis_width = 1;

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            result.push(c);
            if chars.peek() == Some(&'[') {
                result.push('[');
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    result.push(next);
                    if ('\x40'..='\x7e').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        let cw = c.width().unwrap_or(0);
        if width + cw > max_width {
            if width + ellipsis_width <= max_width {
                result.push('…');
            }
            return result;
        }
        if width + cw + ellipsis_width > max_width && chars.peek().is_some() {
            // This char fits, but no room for ellipsis after it — and there ARE more chars.
            // Use ellipsis now instead of this char.
            result.push('…');
            return result;
        }
        width += cw;
        result.push(c);
        if width >= max_width {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn empty_string() {
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn cjk_width() {
        assert_eq!(display_width("中文"), 4);
    }

    #[test]
    fn mixed_cjk_ascii() {
        assert_eq!(display_width("ab中cd"), 6);
    }

    #[test]
    fn emoji_width() {
        assert_eq!(display_width("🎉"), 2);
    }

    #[test]
    fn ansi_escape_ignored() {
        assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
    }

    #[test]
    fn nested_ansi() {
        assert_eq!(display_width("\x1b[1m\x1b[31mbold red\x1b[0m"), 8);
    }

    #[test]
    fn incomplete_ansi() {
        assert_eq!(display_width("\x1bhello"), 5);
    }

    #[test]
    fn pad_right_adds_spaces() {
        assert_eq!(pad_right("hi", 5), "hi   ");
        assert_eq!(display_width(&pad_right("hi", 5)), 5);
    }

    #[test]
    fn pad_right_no_change_when_exact() {
        assert_eq!(pad_right("hello", 5), "hello");
    }

    #[test]
    fn pad_right_zero_width() {
        assert_eq!(pad_right("hello", 0), "");
    }

    #[test]
    fn pad_right_wider_than_target() {
        let result = pad_right("hello world", 5);
        assert_eq!(display_width(&result), 5);
        assert!(result.contains('…'));
    }

    #[test]
    fn pad_right_cjk_exact() {
        let result = pad_right("中文", 6);
        assert_eq!(display_width(&result), 6);
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hi", 5), "hi");
    }

    #[test]
    fn truncate_long_string() {
        let t = truncate("hello world", 6);
        assert!(display_width(&t) <= 6);
        assert!(t.contains('…'));
    }

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_zero_width() {
        assert_eq!(truncate("hello", 0), "");
    }

    #[test]
    fn truncate_exact_fit() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_one_over() {
        let result = truncate("hello!", 5);
        assert!(display_width(&result) <= 5);
        assert!(result.contains('…'));
    }

    #[test]
    fn truncate_cjk_boundary() {
        let result = truncate("中文字", 5);
        assert!(display_width(&result) <= 5);
    }

    #[test]
    fn truncate_preserves_ansi() {
        let result = truncate("\x1b[31mhello world\x1b[0m", 6);
        assert!(display_width(&result) <= 6);
        // ANSI codes should be preserved in output
        assert!(result.contains("\x1b[31m"));
    }

    #[test]
    fn truncate_width_1_single_char_fits() {
        // "hi" at width 1: ellipsis signals there's more content
        let result = truncate("hi", 1);
        assert_eq!(display_width(&result), 1);
    }

    #[test]
    fn truncate_single_char_exact() {
        // Single char string that fits exactly — no truncation needed
        let result = truncate("a", 1);
        assert_eq!(result, "a");
    }

    #[test]
    fn pad_right_width_1() {
        let result = pad_right("hello", 1);
        assert_eq!(display_width(&result), 1);
    }
}
