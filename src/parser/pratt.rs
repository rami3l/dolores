use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

use anyhow::{anyhow, Context, Result};
use num_enum::{FromPrimitive, IntoPrimitive};
use once_cell::sync::Lazy;
use tap::Conv;

use super::{Expr, Parser};
use crate::{
    bail,
    lexer::{Token, TokenType},
};

/// Expression precedence in the Lox language.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub(crate) enum LoxPrec {
    #[default]
    None = 0,
    Assignment = 10, // =
    Or = 20,         // or
    And = 30,        // and
    Equality = 40,   // == !=
    Comparison = 50, // < > <= >=
    Term = 60,       // + -
    Factor = 70,     // * /
    Unary = 80,      // ! -
    Call = 90,       // . ()
    Primary = 100,
}

impl<T: Into<u8>> Add<T> for LoxPrec {
    type Output = u8;

    fn add(self, rhs: T) -> Self::Output {
        self.conv::<u8>().saturating_add(rhs.into())
    }
}

impl<T: Into<u8>> Sub<T> for LoxPrec {
    type Output = u8;

    fn sub(self, rhs: T) -> Self::Output {
        self.conv::<u8>().saturating_sub(rhs.into())
    }
}

/// Description of parsing rules of a [`TokenType`].
pub(crate) struct Parselet {
    /// The "Null Denotation" handler used to handle a prefix/nilfix expression.
    prefix: Option<ParseletRule>,
    /// The "Left Denotation" handler used to handle an infix/postfix
    /// expression.
    infix: Option<ParseletRule>,
    /// The precedence of the [`TokenType`].
    prec: u8,
}

impl Parselet {
    fn new(
        prefix: impl Into<Option<ParseletRule>>,
        infix: impl Into<Option<ParseletRule>>,
        prec: impl Into<u8>,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            infix: infix.into(),
            prec: prec.into(),
        }
    }
}

// Using a shorthand. The actual type signature is:
// `for<'r, 's> fn(&'r mut Parser<'s>) -> Result<Expr>`.
pub(crate) type ParseletRule = fn(&mut Parser) -> Result<Expr>;

static PARSELETS: Lazy<HashMap<TokenType, Parselet>> = Lazy::new(|| {
    macro_rules! rules {
            ( $( ($ty:expr, $prefix:expr, $infix:expr, $prec:expr) ),+ $(,)? ) => {{
                let mut rules = HashMap::new();
                $( rules.insert($ty, Parselet::new($prefix, $infix, $prec)); )+
                rules
            }};
        }
    use LoxPrec as P;
    use TokenType::*;
    rules! {
        (RightParen, None, None, P::None),
        (LeftBrace, None, None, P::None),
        (RightBrace, None, None, P::None),
        (Comma, None, None, P::None),
        (Dot, None, Parser::dot, P::Call),
        (Dot, None, None, P::Call),
        (Minus, Parser::unary, Parser::binary, P::Term),
        (Plus, None, Parser::binary, P::Term),
        (Semicolon, None, None, P::None),
        (Slash, None, Parser::binary, P::Factor),
        (Star, None, Parser::binary, P::Factor),
        (Bang, Parser::unary, None, P::None),
        (BangEqual, None, Parser::binary, P::Equality),
        (Equal, None, None, P::None),
        (EqualEqual, None, Parser::binary, P::Equality),
        (Greater, None, Parser::binary, P::Comparison),
        (GreaterEqual, None, Parser::binary, P::Comparison),
        (Less, None, Parser::binary, P::Comparison),
        (LessEqual, None, Parser::binary, P::Comparison),
        (Identifier, Parser::variable, None, P::None),
        (Str, Parser::string, None, P::None),
        (Number, Parser::number, None, P::None),
        (And, None, Parser::and_op, P::And),
        (Class, None, None, P::None),
        (Else, None, None, P::None),
        (False, Parser::literal, None, P::None),
        (For, None, None, P::None),
        (Fun, None, None, P::None),
        (If, None, None, P::None),
        (Nil, Parser::literal, None, P::None),
        (Or, None, Some(Parser::or_op), P::Or),
        (Print, None, None, P::None),
        (Return, None, None, P::None),
        (Super, Parser::super_, None, P::None),
        (This, Parser::this, None, P::None),
        (True, Parser::literal, None, P::None),
        (Var, None, None, P::None),
        (While, None, None, P::None),
        (Error, None, None, P::None),
    }
});

impl<'s> Parser<'s> {
    fn pratt(&'s mut self, min_prec: u8) -> Result<Expr> {
        let ctx = "while executing the Pratt parser";
        let fst = self.advance().cloned().ok_or_else(|| {
            anyhow!(
                "Internal Error {} with `min_prec={}`: unexpected EoF",
                ctx,
                min_prec
            )
        })?;
        let prefix_rule = if let Some(parselet) = PARSELETS.get(&fst.ty) {
            parselet.prefix
        } else {
            None
        };
        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self)?;
        } else {
            bail!(
                fst.pos,
                &format!("while parsing `{}`", &fst.lexeme),
                "unexpected token",
            );
        }
        todo!()
    }
}
