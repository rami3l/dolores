pub(crate) mod env;
pub(crate) mod object;
mod tests;

use std::fmt::format;

use anyhow::{bail, Context, Result};
use tap::prelude::*;

pub use self::{
    env::{Env, RcCell},
    object::Object,
};
use crate::{
    lexer::TokenType,
    parser::{Expr, Lit, Stmt},
};

impl Expr {
    pub fn eval(self, env: &RcCell<Env>) -> Result<Object> {
        #[allow(clippy::enum_glob_use)]
        use Object::*;
        use TokenType as Tk;

        match self {
            Expr::Assign { name, val } => {
                let name = &name.lexeme;
                Env::lookup(env, name)
                    .with_context(|| format!("Runtime Error: identifier `{}` is undefined", name))
                    .and_then(|_| {
                        let val = val.eval(env)?;
                        Env::set_val(env, name, val.clone());
                        Ok(val)
                    })
            }
            Expr::Binary { lhs, op, rhs } => Ok(match (op.ty, lhs.eval(env)?, rhs.eval(env)?) {
                (Tk::Plus, Str(lhs), Str(rhs)) => Str(lhs + &rhs),
                (Tk::Plus, Str(lhs), rhs) => Str(lhs + &format!("{}", rhs)),
                (Tk::Plus, lhs, Str(rhs)) => Str(format!("{}", lhs) + &rhs),
                (Tk::Plus, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Number(lhs.try_conv::<f64>()? + rhs.try_conv::<f64>()?)
                }
                (Tk::Minus, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Number(lhs.try_conv::<f64>()? - rhs.try_conv::<f64>()?)
                }
                (Tk::Star, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Number(lhs.try_conv::<f64>()? * rhs.try_conv::<f64>()?)
                }
                (Tk::Slash, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Number(lhs.try_conv::<f64>()? / rhs.try_conv::<f64>()?)
                }
                (Tk::Greater, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Bool(lhs.try_conv::<f64>()? > rhs.try_conv::<f64>()?)
                }
                (Tk::GreaterEqual, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Bool(lhs.try_conv::<f64>()? >= rhs.try_conv::<f64>()?)
                }
                (Tk::Less, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Bool(lhs.try_conv::<f64>()? < rhs.try_conv::<f64>()?)
                }
                (Tk::LessEqual, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Bool(lhs.try_conv::<f64>()? <= rhs.try_conv::<f64>()?)
                }
                (Tk::EqualEqual, lhs, rhs) => Bool(lhs == rhs),
                (Tk::BangEqual, lhs, rhs) => Bool(lhs != rhs),

                (op, lhs, rhs) => bail!(
                    "Runtime Error: binary operator `{:?}` undefined for ({:?}, {:?})",
                    op,
                    lhs,
                    rhs,
                ),
            }),
            Expr::Call {
                callee,
                paren,
                args,
            } => todo!(),
            Expr::Get { obj, name } => todo!(),
            Expr::Grouping(expr) => expr.eval(env),
            Expr::Literal(lit) => Ok(lit.into()),
            Expr::Logical { lhs, op, rhs } => match op.ty {
                Tk::And => Ok(Object::Bool(lhs.eval(env)?.into() && rhs.eval(env)?.into())),
                Tk::Or => Ok(Object::Bool(lhs.eval(env)?.into() || rhs.eval(env)?.into())),
                _ => unreachable!(),
            },
            Expr::Set { obj, name, to } => todo!(),
            Expr::Super { kw, method } => todo!(),
            Expr::This(_) => todo!(),
            Expr::Unary { op, rhs } => match op.ty {
                Tk::Bang => Ok(Object::Bool(!rhs.eval(env)?.conv::<bool>())),
                Tk::Minus => Ok(Object::Number(-rhs.eval(env)?.try_into()?)),
                _ => unreachable!(),
            },
            Expr::Variable(name) => {
                let name = &name.lexeme;
                Env::lookup(env, name)
                    .with_context(|| format!("Runtime Error: identifier `{}` is undefined", name))
            }
        }
    }
}

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
            Stmt::Function { name, params, body } => todo!(),
            Stmt::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                if cond.eval(env)?.into() {
                    then_stmt.eval(env)?;
                } else if let Some(else_stmt) = else_stmt {
                    else_stmt.eval(env)?;
                }
            }
            Stmt::Print(expr) => println!("{}", expr.eval(env)?),
            Stmt::Return { kw, val } => todo!(),
            Stmt::Var { name, init } => {
                let init = init
                    .map(|init| init.eval(env))
                    .transpose()?
                    .unwrap_or_default();
                env.borrow_mut().insert_val(&name.lexeme, init);
            }
            Stmt::While { cond, body } => todo!(),
        }
        Ok(())
    }
}
