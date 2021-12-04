use std::fmt::Display;

use logos::Logos;

use crate::util::index_to_pos;

pub(crate) struct Lexer<'s> {
    inner: logos::Lexer<'s, TokenType>,
}

impl<'s> Lexer<'s> {
    pub fn new(src: &'s str) -> Self {
        Lexer {
            inner: TokenType::lexer(src),
        }
    }

    pub fn analyze(self) -> impl Iterator<Item = Token> + 's {
        let src = self.inner.source();
        self.inner.spanned().map(|(ty, span)| Token {
            ty,
            pos: index_to_pos(src, span.start),
            lexeme: src[span].into(),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    /// The `(line_num, column_num)` pair of the starting position of this
    /// token, in the text editor standard (index starting from 1).
    pub pos: (usize, usize),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum TokenType {
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
    SingleLineComment,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn lex(src: &str) -> Vec<(TokenType, String)> {
        Lexer::new(src)
            .analyze()
            .map(|t| (t.ty, t.lexeme))
            .collect()
    }

    macro_rules! assert_lex {
        ($expected:expr, $got:expr $(,)?) => {
            assert_eq!($expected, format!("{:?}", &lex($got)))
        };
    }

    #[test]
    fn test_basic_lexing() {
        assert_lex!(
            r#"[(Var, "var"), (Identifier, "language"), (Equal, "="), (Str, "\"lox\""), (Semicolon, ";")]"#,
            r#"var language = "lox";"#
        );
    }

    #[test]
    fn test_lexing_calculator() {
        assert_lex!(
            r#"[(Number, "1"), (Plus, "+"), (Number, "2"), (Slash, "/"), (Number, "3"), (Minus, "-"), (Number, "4"), (Star, "*"), (Number, "5"), (Plus, "+"), (LeftParen, "("), (Number, "6"), (Slash, "/"), (Number, "7"), (RightParen, ")")]"#,
            "1+ 2 / 3 -4 * 5 + (6/7)",
        );
    }
}
