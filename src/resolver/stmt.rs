use anyhow::Result;

use super::{FunctionContext, Resolver};
use crate::{parser::Stmt, semantic_bail};

impl Resolver {
    pub(crate) fn resolve_stmt(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                stmts.into_iter().try_for_each(|it| self.resolve_stmt(it))?;
                self.end_scope();
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => todo!(),
            Stmt::Expression(expr) => self.resolve_expr(expr)?,
            Stmt::Fun { name, params, body } => {
                self.declare(&name);
                self.define(&name);
                self.resolve_lambda(FunctionContext::Function, &params, body)?;
            }
            Stmt::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                self.resolve_expr(cond)?;
                self.resolve_stmt(*then_stmt)?;
                if let Some(else_stmt) = else_stmt {
                    self.resolve_stmt(*else_stmt)?;
                }
            }
            Stmt::Jump(kw) => {
                // TODO: Add out-of-context jump detection here.
            }
            Stmt::Print(val) => self.resolve_expr(val)?,
            Stmt::Return { val, kw } => {
                if self.function_ctx.is_none() {
                    semantic_bail!(
                        kw.pos,
                        "while resolving a Return statement",
                        "found Return out of function context",
                    )
                }
                if let Some(val) = val {
                    self.resolve_expr(val)?;
                }
            }
            Stmt::Var { name, init } => {
                self.declare(&name);
                if let Some(init) = init {
                    self.resolve_expr(init)?;
                }
                self.define(&name);
            }
            Stmt::While { cond, body } => {
                self.resolve_expr(cond)?;
                self.resolve_stmt(*body)?;
            }
        }
        Ok(())
    }
}
