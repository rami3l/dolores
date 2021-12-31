use std::fmt::Display;

use logos::Logos;

use crate::util::index_to_pos;

pub(crate) struct Lexer<'s> {
    inner: logos::Lexer<'s, TokenType>,
}

impl<'s> Lexer<'s> {
    pub(crate) fn new(src: &'s str) -> Self {
        Self {
            inner: TokenType::lexer(src),
        }
    }

    pub(crate) fn analyze(self) -> impl Iterator<Item = Token> + 's {
        let src = self.inner.source();
        self.inner.spanned().map(|(ty, span)| Token {
            ty,
            pos: index_to_pos(src, span.start),
            lexeme: src[span].into(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct Token {
    pub(crate) ty: TokenType,
    pub(crate) lexeme: String,
    /// The `(line_num, column_num)` pair of the starting position of this
    /// token, in the text editor standard (index starting from 1).
    pub(crate) pos: (usize, usize),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u16)]
pub(crate) enum TokenType {
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
    Identifier,

    #[regex(r#""(([^\r\n\\"]|\\.)*)"|'(([^\r\n\\']|\\.)*)'"#)]
    Str,

    // Numerical conversions are painful! It's better to use floats only here...
    #[regex(r"0|0x[0-9a-fA-F_]+|[0-9]+")]
    #[regex(r"[0-9]*\.[0-9]+([eE][+-]?[0-9]+)?")]
    Number,

    // Keywords.
    #[token("and")]
    And,

    #[token("break")]
    Break,

    #[token("class")]
    Class,

    #[token("continue")]
    Continue,

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
    SingleLineComment,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    fn lex(src: &str) -> Vec<(TokenType, String)> {
        Lexer::new(src)
            .analyze()
            .map(|t| (t.ty, t.lexeme))
            .collect()
    }

    macro_rules! assert_lex {
        ($got:expr, $expected:expr $(,)?) => {
            assert_eq!($expected, format!("{:?}", &lex($got)))
        };
    }

    #[test]
    fn basic() {
        assert_lex!(
            r#"var language = "lox";"#,
            r#"[(Var, "var"), (Identifier, "language"), (Equal, "="), (Str, "\"lox\""), (Semicolon, ";")]"#,
        );
        assert_lex!(
            r#"print language;"#,
            r#"[(Print, "print"), (Identifier, "language"), (Semicolon, ";")]"#,
        );
    }

    #[test]
    fn calculator() {
        assert_lex!(
            "1+ 2 / 3 -4 * 5 + (6/7)",
            r#"[(Number, "1"), (Plus, "+"), (Number, "2"), (Slash, "/"), (Number, "3"), (Minus, "-"), (Number, "4"), (Star, "*"), (Number, "5"), (Plus, "+"), (LeftParen, "("), (Number, "6"), (Slash, "/"), (Number, "7"), (RightParen, ")")]"#,
        );
    }

    #[test]
    fn comments() {
        assert_lex!(
            indoc! {r"
                var a = 1; // Wow
                // This is a comment
                a = false;
            "},
            r#"[(Var, "var"), (Identifier, "a"), (Equal, "="), (Number, "1"), (Semicolon, ";"), (SingleLineComment, "// Wow\n"), (SingleLineComment, "// This is a comment\n"), (Identifier, "a"), (Equal, "="), (False, "false"), (Semicolon, ";")]"#,
        );
    }
}
