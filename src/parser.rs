pub(crate) mod expr;
pub(crate) mod stmt;

use std::{fmt::Display, iter::Peekable};

use anyhow::{Context, Result};
use itertools::Itertools;

pub(crate) use self::{
    expr::{Expr, Lit},
    stmt::Stmt,
};
use crate::lexer::Lexer;
#[allow(clippy::enum_glob_use)]
use crate::{
    error::report,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

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

    fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let new_prev = self.peek()?;
        self.prev.replace(new_prev);
        self.tokens.next();
        self.prev.clone()
    }

    fn previous(&self) -> Option<Token> {
        self.prev.clone()
    }

    fn check(&mut self, ty: TokenType) -> Option<Token> {
        self.peek().filter(|t| t.ty == ty)
    }

    fn test(&mut self, tys: &[TokenType]) -> Option<Token> {
        tys.iter().find_map(|&ty| {
            let curr = self.peek();
            self.check(ty).and_then(|_| {
                self.advance();
                curr
            })
        })
    }

    /// Consumes a specific token or throws an error.
    fn consume(&mut self, tys: &[TokenType], ctx: &str, msg: impl Display) -> Result<Token> {
        self.test(tys)
            .with_context(|| report(self.previous().unwrap().pos, ctx, msg))
    }

    fn sync(&mut self) {
        let stmt_begin = [Class, Fun, Var, For, If, While, Print, Return];
        loop {
            self.advance();
            let curr = self.peek();
            let synced = curr.is_none() // Reached the end of the source.
                || self.previous().unwrap().ty == Semicolon // Passed the end of the statement.
                || stmt_begin.contains(&curr.unwrap().ty); // Reached the beginning of another statement.
            if synced {
                break;
            }
        }
    }

    pub(crate) fn many<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        std::iter::from_fn(|| self.peek().map(|_| parser(self))).try_collect()
    }

    pub(crate) fn many_till<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
        till: TokenType,
    ) -> Result<Vec<T>> {
        std::iter::from_fn(|| self.peek().filter(|t| t.ty != till).map(|_| parser(self)))
            .try_collect()
    }

    pub(crate) fn parens<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
        ctx: &str,
    ) -> Result<T> {
        self.consume(&[LeftParen], ctx, format!("expected `(` before {}", ctx))?;
        let res = parser(self)?;
        self.consume(&[RightParen], ctx, format!("expected `)` after {}", ctx))?;
        Ok(res)
    }

    pub(crate) fn parse(&mut self) -> Result<Vec<Stmt>> {
        self.many(Self::decl)
    }
}
