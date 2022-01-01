use num_enum::{FromPrimitive, IntoPrimitive};

use self::lexer::SyntaxKind;

pub(crate) mod lexer;
pub(crate) mod parser;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LoxLanguage {}

impl rowan::Language for LoxLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        raw.0.into()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into())
    }
}

/// Expression precedence in the Lox language.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub(crate) enum LoxPrec {
    #[default]
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl LoxPrec {
    /// Get the next (higher) precedence in the list.
    pub(crate) fn next(self) -> Self {
        Self::from(1 + self.into())
    }
}
