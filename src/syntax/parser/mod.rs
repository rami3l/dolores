// pub(crate) mod expr;
pub(crate) mod pratt;
// pub(crate) mod stmt;

use std::{fmt::Display, iter::Peekable};

use anyhow::{Context, Result};
use itertools::Itertools;
use rowan::{Checkpoint, GreenNodeBuilder, Language};

/* pub(crate) use self::{
    expr::{Expr, Lit},
    stmt::Stmt,
}; */
use super::{lexer::Lexer, LoxLanguage};
#[allow(clippy::enum_glob_use)]
use crate::{
    error::report,
    lexer::{
        SyntaxKind::{self, *},
        Token,
    },
};

pub(crate) struct Parser<'s> {
    lexer: Peekable<Lexer<'s>>,
    builder: GreenNodeBuilder<'static>,
}

// Pseudo-inheritance...
impl<'s> Parser<'s> {
    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder
            .start_node_at(checkpoint, LoxLanguage::kind_to_raw(kind));
    }

    fn token(&mut self, kind: SyntaxKind, text: &str) {
        self.builder.token(LoxLanguage::kind_to_raw(kind), text);
    }

    fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }
}

impl<'s> Parser<'s> {
    fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|(kind, _)| *kind)
    }
}
