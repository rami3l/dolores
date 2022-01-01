mod class;
pub(crate) mod closure;
pub(crate) mod env;
mod expr;
mod jump;
pub(crate) mod object;
mod stmt;
mod tests;

use std::{collections::HashMap, mem, sync::Arc};

use anyhow::Result;

pub(crate) use self::{
    class::{Class, Instance},
    closure::Closure,
    env::Env,
    jump::{BreakMarker, ContinueMarker, ReturnMarker},
    object::Object,
};
use crate::{
    lexer::Token,
    parser::{Expr, Stmt},
    resolver::Resolver,
    util::RcCell,
};

/// The interpreter, containing the necessary evaluation context for expressions
/// and statements.
#[derive(Debug, Clone)]
pub(crate) struct Interpreter {
    env: RcCell<Env>,
    pub(crate) globals: RcCell<Env>,
    pub(crate) locals: HashMap<Token, usize>,
}

impl Interpreter {
    #[must_use]
    pub(crate) fn new(env: &RcCell<Env>) -> Self {
        Self {
            env: Arc::clone(env),
            globals: Arc::clone(env),
            locals: HashMap::new(),
        }
    }

    pub(crate) fn resolve_expr(&mut self, expr: Expr) -> Result<()> {
        let mut resolver = Resolver::new(mem::take(self));
        resolver.resolve_expr(expr)?;
        *self = resolver.interpreter;
        Ok(())
    }

    pub(crate) fn resolve_stmts(&mut self, stmts: impl IntoIterator<Item = Stmt>) -> Result<()> {
        *self = Resolver::new(mem::take(self)).resolve(stmts)?;
        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new(&Env::default().shared())
    }
}
