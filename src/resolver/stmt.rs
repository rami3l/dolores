use anyhow::Result;

use super::{FunctionContextType, JumpContext, Resolver};
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
                // We define a function's name eagerly to enable hoisting, which is ideal for
                // usages like recursion. We don't like JavaScript, so we don't
                // hoist variables.
                self.define(&name);
                let ctx = JumpContext {
                    fun_ty: Some(FunctionContextType::Function),
                    in_loop: false,
                };
                self.resolve_lambda(ctx, &params, body)?;
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
                if !self.jump_ctx.in_loop {
                    semantic_bail!(
                        kw.pos,
                        "while resolving a Jump statement",
                        "found `{}` out of loop context",
                        kw.lexeme,
                    )
                }
            }
            Stmt::Print(val) => self.resolve_expr(val)?,
            Stmt::Return { val, kw } => {
                if self.jump_ctx.fun_ty.is_none() {
                    semantic_bail!(
                        kw.pos,
                        "while resolving a Return statement",
                        "found `return` out of function context",
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
                let old_in_loop = std::mem::replace(&mut self.jump_ctx.in_loop, true);
                self.resolve_expr(cond)?;
                self.resolve_stmt(*body)?;
                self.jump_ctx.in_loop = old_in_loop;
            }
        }
        Ok(())
    }
}
