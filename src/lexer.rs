use itertools::Itertools;

pub(crate) fn tokenize(src: &str) -> Vec<&str> {
    src.split_whitespace().collect_vec()
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_basic_lexing() {
        let src = r#"var language = "lox";"#;
        assert_equal(&["var", "language", "=", r#""lox""#, ";"], &tokenize(src));
    }
}
