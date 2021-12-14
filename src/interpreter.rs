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
use crate::parser::Expr;

/// The interpreter, containing the necessary evaluation context for expressions
/// and statements.
#[derive(Debug, Default, Clone)]
pub struct Interpreter {
    pub env: RcCell<Env>,
    pub locals: HashMap<Expr, usize>,
}
