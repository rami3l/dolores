pub(crate) mod expr;
pub(crate) mod stmt;

use std::{fmt::Display, iter::Peekable};

use anyhow::{Context, Result};
use itertools::Itertools;

pub(crate) use self::{
    expr::{Expr, Lit},
    stmt::Stmt,
};
use crate::lexer::{
    Token,
    TokenType::{self, *},
};
use crate::{error::Error, lexer::Lexer};

pub(crate) struct Parser<'s> {
    tokens: Peekable<Lexer<'s>>,
    prev: Option<Token>,
}

impl<'s> Parser<'s> {
    pub(crate) fn new(tokens: Lexer<'s>) -> Self {
        Self {
            tokens: tokens.peekable(),
            prev: None,
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn advance(&mut self) -> Option<&Token> {
        let new_prev = self.peek()?.clone();
        self.prev.replace(new_prev);
        self.tokens.next();
        self.prev.as_ref()
    }

    fn previous(&self) -> Option<&Token> {
        self.prev.as_ref()
    }

    fn check(&mut self, ty: TokenType) -> Option<&Token> {
        self.peek().filter(|&t| t.ty == ty)
    }

    fn test(&mut self, tys: &[TokenType]) -> Option<&Token> {
        for &ty in tys {
            if self.check(ty).is_some() {
                self.advance();
                return self.previous();
            }
        }
        None
    }

    /// Consumes a specific token or throws an error.
    fn consume(&mut self, tys: &[TokenType], msg: impl Display) -> Result<Token> {
        self.test(tys).cloned().with_context(|| Error::ParseError {
            pos: self.previous().unwrap().pos,
            msg: format!("{msg}"),
        })
    }

    fn sync(&mut self) {
        let stmt_begin = [Class, Fun, Var, For, If, While, Print, Return];
        loop {
            self.advance();
            let curr_ty = self.peek().map(|it| it.ty);
            let synced = curr_ty.is_none() // Reached the end of the source.
                || self.previous().unwrap().ty == Semicolon // Passed the end of the statement.
                || stmt_begin.contains(&curr_ty.unwrap()); // Reached the beginning of another statement.
            if synced {
                break;
            }
        }
    }

    pub(crate) fn many<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        std::iter::from_fn(|| self.peek().is_some().then(|| parser(self))).try_collect()
    }

    pub(crate) fn many_till<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
        till: TokenType,
    ) -> Result<Vec<T>> {
        std::iter::from_fn(|| {
            self.peek()
                .filter(|t| t.ty != till)
                .is_some()
                .then(|| parser(self))
        })
        .try_collect()
    }

    pub(crate) fn parens<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
        ctx: &str,
    ) -> Result<T> {
        self.consume(&[LeftParen], format!("expected `(` before {ctx}"))?;
        let res = parser(self)?;
        self.consume(&[RightParen], format!("expected `)` after {ctx}"))?;
        Ok(res)
    }

    pub(crate) fn parse(&mut self) -> Result<Vec<Stmt>> {
        self.many(Self::decl)
    }
}
