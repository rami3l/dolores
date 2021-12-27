use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use super::{BreakMarker, Closure, ContinueMarker, Env, Interpreter, Object, RcCell, ReturnMarker};
use crate::{lexer::TokenType as Tk, parser::Stmt};

impl Interpreter {
    pub fn exec(&mut self, stmt: Stmt) -> Result<()> {
        let env = &Arc::clone(&self.env);
        match stmt {
            Stmt::Block(stmts) => {
                let old_env = Arc::clone(env);
                // Temporarily switch into the scope environment...
                self.env = Env::from_outer(env).shared();
                stmts.into_iter().try_for_each(|it| self.exec(it))?;
                // Switch back...
                self.env = old_env;
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => todo!(),
            Stmt::Expression(expr) => {
                self.eval(expr)?;
            }
            Stmt::Fun { name, params, body } => {
                let name = &name.lexeme;
                let closure = Object::NativeFn(Closure {
                    uid: Uuid::new_v4(),
                    name: Some(name.into()),
                    params,
                    body,
                    env: Arc::clone(env),
                });
                env.lock().insert_val(name, closure);
            }
            Stmt::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                if self.eval(cond)?.to_bool() {
                    self.exec(*then_stmt)?;
                } else if let Some(else_stmt) = else_stmt {
                    self.exec(*else_stmt)?;
                }
            }

            Stmt::Jump(t) => match t.ty {
                Tk::Break => return Err(anyhow::Error::new(BreakMarker)),
                Tk::Continue => return Err(anyhow::Error::new(ContinueMarker)),
                _ => unreachable!(),
            },

            Stmt::Print(expr) => println!("{}", self.eval(expr)?),
            Stmt::Return { kw: _, val } => {
                let obj = self.eval(val.unwrap_or_default())?;
                return Err(anyhow::Error::new(ReturnMarker(obj)));
            }
            Stmt::Var { name, init } => {
                let init = self.eval(init.unwrap_or_default())?;
                self.env.lock().insert_val(&name.lexeme, init);
            }
            Stmt::While { cond, body } => {
                while self.eval(cond.clone())?.to_bool() {
                    match self.exec(*body.clone()) {
                        Err(e) if e.is::<BreakMarker>() => break,
                        Err(e) if e.is::<ContinueMarker>() => continue,
                        res => res?,
                    }
                }
            }
        }
        Ok(())
    }
}
