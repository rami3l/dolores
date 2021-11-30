use logos::Logos;

#[repr(u16)]
#[derive(Logos, Debug, PartialEq, Clone)]
enum Token<'a> {
    // Single-character tokens.
    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token("-")]
    Minus,

    #[token("+")]
    Plus,

    #[token(";")]
    Semicolon,

    #[token("/")]
    Slash,

    #[token("*")]
    Star,

    // One or two character tokens.
    #[token("!")]
    Bang,

    #[token("!=")]
    BangEqual,

    #[token("=")]
    Equal,

    #[token("==")]
    EqualEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    // Literals.
    #[regex(r"[a-zA-Z_][0-9a-zA-Z_?!]*")]
    Identifier(&'a str),

    #[regex(r#""(([^\r\n\\"]|\\.)*)"|'(([^\r\n\\']|\\.)*)'"#)]
    String(&'a str),

    // Numerical conversions are painful! It's better to use floats only here...
    #[regex(r"0|0x[0-9a-fA-F_]+|[0-9]+", |lex| lex.slice().parse())]
    #[regex(r"[0-9]*\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse())]
    Number(f64),

    // Keywords.
    #[token("and")]
    And,

    #[token("class")]
    Class,

    #[token("else")]
    Else,

    #[token("false")]
    False,

    #[token("fun")]
    Fun,

    #[token("for")]
    For,

    #[token("if")]
    If,

    #[token("nil")]
    Nil,

    #[token("or")]
    Or,

    #[token("print")]
    Print,

    #[token("return")]
    Return,

    #[token("super")]
    Super,

    #[token("this")]
    This,

    #[token("true")]
    True,

    #[token("var")]
    Var,

    #[token("while")]
    While,

    // Misc.
    #[regex(r"//[^\r\n]*(\r\n|\n)?")]
    // TODO: Add MultiLineComment maybe?
    SingleLineComment(&'a str),

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use std::{fmt, ops::Range};

    use logos::source::Source;
    use pretty_assertions::assert_eq;

    use super::*;

    fn lex_spanned<'a, T, S>(source: &'a T::Source) -> Vec<(T, Range<usize>)>
    where
        T: Logos<'a>,
        T::Extras: Default,
        T::Source: Source<Slice = S>,
        S: 'a + ?Sized,
    {
        T::lexer(source).spanned().collect()
    }

    // Shamelessly copied from [Logos](https://github.com/maciejhirsz/logos/blob/925c49e9bde178700d5c6c1843133017a88bab85/tests/src/lib.rs)
    fn assert_lex<'a, T, S>(source: &'a T::Source, expected_tokens: &[(T, Range<usize>)])
    where
        T: Logos<'a> + PartialEq + fmt::Debug,
        T::Extras: Default,
        T::Source: Source<Slice = S>,
        S: 'a + ?Sized + PartialEq + fmt::Debug,
    {
        assert_eq!(expected_tokens, &lex_spanned(source));
    }

    #[test]
    fn test_basic_lexing() {
        use Token::*;
        assert_lex(
            r#"var language = "lox";"#,
            &[
                (Var, 0..3),
                (Identifier("language"), 4..12),
                (Equal, 13..14),
                (String(r#""lox""#), 15..20),
                (Semicolon, 20..21),
            ],
        );
    }
}
