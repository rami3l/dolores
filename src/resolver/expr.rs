use anyhow::Result;

use super::{FunctionContextType, JumpContext, ResolutionState, Resolver};
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
            Expr::Get { obj, .. } => self.resolve_expr(*obj)?,
            Expr::Grouping(inner) => self.resolve_expr(*inner)?,
            Expr::Lambda { params, body } => {
                let ctx = JumpContext {
                    fun_ty: Some(FunctionContextType::Function),
                    in_loop: false,
                };
                self.resolve_lambda(ctx, &params, body)?;
            }
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
}
