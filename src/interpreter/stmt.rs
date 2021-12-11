use std::sync::Arc;

use anyhow::Result;

use super::{BreakMarker, Closure, ContinueMarker, Env, Object, RcCell, ReturnMarker};
use crate::{lexer::TokenType as Tk, parser::Stmt};

impl Stmt {
    pub fn eval(self, env: &RcCell<Env>) -> Result<()> {
        match self {
            Stmt::Block(stmts) => {
                let inner = Env::from_outer(env).shared();
                stmts.into_iter().try_for_each(|stmt| stmt.eval(&inner))?;
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => todo!(),
            Stmt::Expression(expr) => {
                expr.eval(env)?;
            }
            Stmt::Fun { name, params, body } => {
                let name = &name.lexeme;
                let closure = Object::NativeFn(Closure {
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
                if cond.eval(env)?.to_bool() {
                    then_stmt.eval(env)?;
                } else if let Some(else_stmt) = else_stmt {
                    else_stmt.eval(env)?;
                }
            }

            Stmt::Jump(t) => match t.ty {
                Tk::Break => return Err(anyhow::Error::new(BreakMarker)),
                Tk::Continue => return Err(anyhow::Error::new(ContinueMarker)),
                _ => unreachable!(),
            },

            Stmt::Print(expr) => println!("{}", expr.eval(env)?),
            Stmt::Return { kw: _, val } => {
                let obj = val.unwrap_or_default().eval(env)?;
                return Err(anyhow::Error::new(ReturnMarker(obj)));
            }
            Stmt::Var { name, init } => {
                let init = init
                    .map(|init| init.eval(env))
                    .transpose()?
                    .unwrap_or_default();
                env.lock().insert_val(&name.lexeme, init);
            }
            Stmt::While { cond, body } => {
                while cond.clone().eval(env)?.to_bool() {
                    match body.clone().eval(env) {
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
