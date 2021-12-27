pub(crate) mod closure;
pub(crate) mod env;
mod expr;
mod jump;
pub(crate) mod object;
mod stmt;
mod tests;

use std::collections::HashMap;

pub use self::{
    closure::Closure,
    env::{Env, RcCell},
    jump::{BreakMarker, ContinueMarker, ReturnMarker},
    object::Object,
};
use crate::lexer::Token;

/// The interpreter, containing the necessary evaluation context for expressions
/// and statements.
#[derive(Debug, Default, Clone)]
pub struct Interpreter {
    pub env: RcCell<Env>,
    pub locals: HashMap<Token, usize>,
}

impl Interpreter {
    pub(crate) fn resolve(&mut self, name: Token, distance: usize) {
        self.locals.insert(name, distance);
    }
}
