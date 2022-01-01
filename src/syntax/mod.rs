use num_enum::{FromPrimitive, IntoPrimitive};
use tap::Conv;

pub(crate) mod lexer;
pub(crate) mod parser;

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
        Self::from(1_u8 + self.conv::<u8>())
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
