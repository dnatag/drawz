pub(super) fn skip_first_line(code: &str) -> &str {
    if let Some(pos) = code.find(['\n', ';']) {
        &code[pos + 1..]
    } else {
        ""
    }
}

pub(super) fn split_statements(body: &str) -> Vec<&str> {
    body.split(['\n', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("%%"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_skip_first_line_when_newline() {
        assert_eq!(skip_first_line("graph LR\nA-->B"), "A-->B");
    }

    #[test]
    fn should_skip_first_line_when_semicolon() {
        assert_eq!(skip_first_line("graph LR;A-->B"), "A-->B");
    }

    #[test]
    fn should_return_empty_when_no_separator() {
        assert_eq!(skip_first_line("graph LR"), "");
    }

    #[test]
    fn should_split_on_newlines_and_semicolons() {
        let stmts = split_statements("A-->B\nC-->D;E-->F");
        assert_eq!(stmts, vec!["A-->B", "C-->D", "E-->F"]);
    }

    #[test]
    fn should_filter_empty_and_comments() {
        let stmts = split_statements("\n%% comment\nA-->B\n\n");
        assert_eq!(stmts, vec!["A-->B"]);
    }
}
