// pub(crate) mod expr;
pub(crate) mod pratt;
// pub(crate) mod stmt;

use std::{fmt::Display, iter::Peekable};

use anyhow::{Context, Result};
use itertools::Itertools;
use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, Language};

/* pub(crate) use self::{
    expr::{Expr, Lit},
    stmt::Stmt,
}; */
use super::{lexer::Lexer, LoxLanguage};
#[allow(clippy::enum_glob_use)]
use crate::{
    error::report,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

pub(crate) struct Parser<'s> {
    lexer: Peekable<Lexer<'s>>,
    builder: GreenNodeBuilder<'static>,
    // Ideally we should store a list of errors here, but I'll keep just the first one for
    // simplicity.
    // errors: Vec<String>,
}

// Pseudo-inheritance...
impl<'s> Parser<'s> {
    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: TokenType) {
        self.builder
            .start_node_at(checkpoint, LoxLanguage::kind_to_raw(kind));
    }

    fn token(&mut self, kind: TokenType, text: &str) {
        self.builder.token(LoxLanguage::kind_to_raw(kind), text);
    }

    fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }
}

// Util methods...
impl<'s> Parser<'s> {
    fn peek(&mut self) -> Option<TokenType> {
        self.lexer.peek().map(|(kind, _)| *kind)
    }
}

pub(crate) struct Parse {
    green_node: GreenNode,
    error: Option<anyhow::Error>,
}
