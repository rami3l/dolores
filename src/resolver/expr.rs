use anyhow::Result;

use super::{ResolutionState, Resolver};
use crate::{
    lexer::Token,
    parser::{Expr, Stmt},
    semantic_bail,
};

impl Resolver {
    pub(crate) fn resolve_expr(&mut self, expr: Expr) -> Result<()> {
        match expr {
            Expr::Assign { name, val } => {
                self.resolve_expr(*val)?;
                self.resolve_local(&name);
            }
            Expr::Binary { lhs, rhs, .. } | Expr::Logical { lhs, rhs, .. } => {
                self.resolve_expr(*lhs)?;
                self.resolve_expr(*rhs)?;
            }
            Expr::Call { callee, args, .. } => {
                self.resolve_expr(*callee)?;
                args.into_iter().try_for_each(|it| self.resolve_expr(it))?;
            }
            Expr::Get { obj, name } => todo!(),
            Expr::Grouping(inner) => self.resolve_expr(*inner)?,
            Expr::Lambda { params, body } => self.resolve_lambda(&params, body)?,
            Expr::Literal(_) => (),
            Expr::Set { obj, name, to } => todo!(),
            Expr::Super { kw, method } => todo!(),
            Expr::This(_) => todo!(),
            Expr::Unary { rhs, .. } => self.resolve_expr(*rhs)?,
            Expr::Variable(tk) => {
                if let Some(ResolutionState::Declared) =
                    self.scopes.last().and_then(|last| last.get(&tk.lexeme))
                {
                    semantic_bail!(
                        tk.pos,
                        "while resolving a Variable expression",
                        "cannot read local Variable `{}` in its own initializer",
                        tk.lexeme
                    )
                }
                self.resolve_local(&tk);
            }
        }
        Ok(())
    }

    fn resolve_local(&mut self, name: &Token) {
        self.scopes
            .iter()
            .rev()
            .enumerate()
            .find_map(|(distance, scope)| {
                scope
                    .contains_key(&name.lexeme)
                    .then(|| self.interpreter.locals.insert(name.clone(), distance))
            });
    }

    pub(crate) fn resolve_lambda(&mut self, params: &[Token], body: Vec<Stmt>) -> Result<()> {
        self.begin_scope();
        for it in params {
            self.declare(it);
            self.define(it);
        }
        body.into_iter().try_for_each(|it| self.resolve_stmt(it))?;
        self.end_scope();
        Ok(())
    }
}
