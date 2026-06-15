use crate::measure::{display_width, pad_right, truncate};
use crate::result::RenderContext;
use crate::schema::TableDiagram;

/// Render table with auto-sized columns, borders, and truncation.
///
/// # Errors
///
/// Returns an error if headers are empty or the table cannot fit at the given width.
pub(crate) fn render(diagram: &TableDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String> {
    if diagram.headers.is_empty() {
        return Err("table requires at least one header".to_string());
    }

    let num_cols = diagram.headers.len();
    let separator_width = if num_cols > 1 { (num_cols - 1) * 3 } else { 0 };
    let available = ctx.inner_width.saturating_sub(separator_width);

    if available < num_cols {
        return Err(format!(
            "cannot render {num_cols}-column table at width {} (minimum needed: {})",
            ctx.total_width,
            separator_width + num_cols * 6
        ));
    }

    // Compute natural column widths from content
    let mut col_widths: Vec<usize> = diagram.headers.iter().map(|h| display_width(h)).collect();
    for row in &diagram.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(display_width(cell));
            }
        }
    }

    // Shrink columns to fit available width
    let total_natural: usize = col_widths.iter().sum();
    if total_natural > available {
        let mut truncated = false;
        for w in &mut col_widths {
            let new_w = (*w * available) / total_natural;
            let new_w = new_w.max(3);
            if new_w < *w {
                truncated = true;
            }
            *w = new_w;
        }
        // Fix rounding: shrink or grow to hit exactly `available`
        let sum: usize = col_widths.iter().sum();
        if sum < available {
            col_widths[0] += available - sum;
        } else if sum > available {
            // Over-allocated due to .max(3) — shrink last columns to fit
            let mut excess = sum - available;
            for w in col_widths.iter_mut().rev() {
                if excess == 0 { break; }
                let reduce = excess.min(w.saturating_sub(3));
                *w -= reduce;
                excess -= reduce;
            }
        }
        let final_sum: usize = col_widths.iter().sum();
        if truncated || final_sum > available {
            ctx.warnings.push("suggestion: reduce columns or set wider width".to_string());
        }
    }

    let mut lines = Vec::new();

    // Header line
    let header: String = diagram.headers.iter().enumerate()
        .map(|(i, h)| fit_cell(h, col_widths[i]))
        .collect::<Vec<_>>()
        .join(" │ ");
    lines.push(pad_right(&header, ctx.inner_width));

    // Separator line
    let sep: String = col_widths.iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("─┼─");
    lines.push(pad_right(&sep, ctx.inner_width));

    // Data rows
    for row in &diagram.rows {
        let cells: String = (0..num_cols)
            .map(|i| {
                let cell = row.get(i).map_or("", String::as_str);
                fit_cell(cell, col_widths[i])
            })
            .collect::<Vec<_>>()
            .join(" │ ");
        lines.push(pad_right(&cells, ctx.inner_width));
    }

    Ok(lines)
}

fn unescape_cell(s: &str) -> String {
    s.replace("\\n", " ").replace("\\t", " ")
}

fn fit_cell(content: &str, width: usize) -> String {
    let content = unescape_cell(content);
    let w = display_width(&content);
    if w <= width {
        pad_right(&content, width)
    } else {
        truncate(&content, width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::display_width;
    use crate::result::RenderContext;
    use crate::schema::TableDiagram;

    fn ctx(width: usize) -> RenderContext {
        RenderContext { inner_width: width, total_width: u16::try_from(width).unwrap(), warnings: Vec::new() }
    }

    #[test]
    fn should_render_headers_separator_rows_when_valid_input() {
        let d = TableDiagram {
            title: None,
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()]],
        };
        let lines = render(&d, &mut ctx(20)).unwrap();
        assert_eq!(lines.len(), 3); // header + separator + 1 row
        for l in &lines { assert_eq!(display_width(l), 20); }
        assert!(lines[1].contains('┼'));
    }

    #[test]
    fn should_return_error_when_headers_empty() {
        let d = TableDiagram { title: None, headers: vec![], rows: vec![] };
        assert!(render(&d, &mut ctx(20)).is_err());
    }

    #[test]
    fn should_shrink_and_warn_when_columns_exceed_width() {
        let d = TableDiagram {
            title: None,
            headers: vec!["LongHeader".into(), "Another".into()],
            rows: vec![vec!["data".into(), "more".into()]],
        };
        let mut c = ctx(15);
        let lines = render(&d, &mut c).unwrap();
        for l in &lines { assert_eq!(display_width(l), 15); }
        assert!(!c.warnings.is_empty());
    }

    #[test]
    fn should_pad_missing_cells_when_row_shorter_than_headers() {
        let d = TableDiagram {
            title: None,
            headers: vec!["X".into(), "Y".into(), "Z".into()],
            rows: vec![vec!["only one".into()]],
        };
        let lines = render(&d, &mut ctx(30)).unwrap();
        assert_eq!(lines.len(), 3);
        for l in &lines { assert_eq!(display_width(l), 30); }
    }

    #[test]
    fn should_return_error_when_width_too_narrow() {
        let d = TableDiagram {
            title: None,
            headers: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            rows: vec![],
        };
        assert!(render(&d, &mut ctx(5)).is_err());
    }
}
