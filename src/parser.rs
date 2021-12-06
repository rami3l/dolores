pub(crate) mod expr;
pub(crate) mod stmt;
mod tests;

use std::fmt::Display;

use anyhow::{Context, Result};
use itertools::Itertools;

pub(crate) use self::{
    expr::{Expr, Lit},
    stmt::Stmt,
};
#[allow(clippy::enum_glob_use)]
use crate::{
    lexer::{
        Token,
        TokenType::{self, *},
    },
    run::report,
};

pub(crate) struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: impl IntoIterator<Item = Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().collect(),
            idx: 0,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.idx).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let res = self.peek()?;
        self.idx += 1;
        Some(res)
    }

    fn previous(&self) -> Option<Token> {
        self.tokens.get(self.idx - 1).cloned()
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

    pub(crate) fn many0<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        std::iter::from_fn(|| self.peek().map(|_| parser(self))).try_collect()
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

    pub(crate) fn run(&mut self) -> Result<Vec<Stmt>> {
        self.many0(Self::decl)
    }
}
