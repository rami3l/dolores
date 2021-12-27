pub(crate) mod closure;
pub(crate) mod env;
mod expr;
mod jump;
pub(crate) mod object;
mod stmt;
mod tests;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

pub use self::{
    closure::Closure,
    env::{Env, RcCell},
    jump::{BreakMarker, ContinueMarker, ReturnMarker},
    object::Object,
};
use crate::{
    lexer::Token,
    parser::{Expr, Stmt},
    resolver::Resolver,
};

/// The interpreter, containing the necessary evaluation context for expressions
/// and statements.
#[derive(Debug, Clone)]
pub struct Interpreter {
    env: RcCell<Env>,
    pub globals: RcCell<Env>,
    pub locals: HashMap<Token, usize>,
}

impl Interpreter {
    #[must_use]
    pub fn new(env: &RcCell<Env>) -> Self {
        Interpreter {
            env: Arc::clone(env),
            globals: Arc::clone(env),
            locals: HashMap::new(),
        }
    }

    pub fn resolve_expr(&mut self, expr: Expr) -> Result<()> {
        let mut resolver = Resolver::new(std::mem::take(self));
        resolver.resolve_expr(expr)?;
        *self = resolver.interpreter;
        Ok(())
    }

    pub fn resolve_stmts(&mut self, stmts: impl IntoIterator<Item = Stmt>) -> Result<()> {
        *self = Resolver::new(std::mem::take(self)).resolve(stmts)?;
        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new(&Env::default().shared())
    }
}
