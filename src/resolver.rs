use std::collections::HashMap;

use itertools::Itertools;

use crate::lexer::Token;

#[derive(Debug, Clone, Copy)]
pub enum ResolutionState {
    Declared,
    Defined,
}

pub type Scope = HashMap<String, ResolutionState>;

#[derive(Debug, Clone, Default)]
pub struct Resolver {
    scopes: Vec<Scope>,
}

impl Resolver {
    #[must_use]
    pub fn new(scopes: impl Iterator<Item = Scope>) -> Self {
        Resolver {
            scopes: scopes.collect(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    /// Sets the resolution state of the given `token` in the currently smallest
    /// scope, returning the last state if exists.
    fn set_state(&mut self, token: &Token, state: ResolutionState) -> Option<ResolutionState> {
        self.scopes
            .last_mut()
            .and_then(|last| last.insert(token.lexeme.clone(), state))
    }

    fn declare(&mut self, token: &Token) -> Option<ResolutionState> {
        self.set_state(token, ResolutionState::Declared)
    }

    fn define(&mut self, token: &Token) -> Option<ResolutionState> {
        self.set_state(token, ResolutionState::Defined)
    }
}

mod expr {
    use anyhow::Result;

    use super::{ResolutionState, Resolver};
    use crate::{parser::Expr, semantic_bail};

    impl Resolver {
        fn check_expr(&mut self, expr: Expr) -> Result<()> {
            match expr {
                Expr::Assign { name, val } => todo!(),
                Expr::Binary { lhs, op, rhs } => todo!(),
                Expr::Call { callee, args, end } => todo!(),
                Expr::Get { obj, name } => todo!(),
                Expr::Grouping(_) => todo!(),
                Expr::Lambda { params, body } => todo!(),
                Expr::Literal(_) => todo!(),
                Expr::Logical { lhs, op, rhs } => todo!(),
                Expr::Set { obj, name, to } => todo!(),
                Expr::Super { kw, method } => todo!(),
                Expr::This(_) => todo!(),
                Expr::Unary { op, rhs } => todo!(),
                Expr::Variable(tk) => {
                    if let Some(ResolutionState::Declared) =
                        self.scopes.last().and_then(|last| last.get(&tk.lexeme))
                    {
                        semantic_bail!(
                            tk.pos,
                            "while resolving a Variable expression",
                            "cannot read local Variable `{}` in its own initializer",
                            &tk.lexeme
                        )
                    }
                    self.resolve_local(Expr::Variable(tk), tk.clone())
                }
            }
        }

        fn resolve_local()
    }
}

mod stmt {}
