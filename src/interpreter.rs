pub(crate) mod closure;
pub(crate) mod env;
mod jump;
pub(crate) mod object;
mod tests;

use std::rc::Rc;

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use tap::prelude::*;

pub use self::{
    closure::Closure,
    env::{Env, RcCell},
    jump::{BreakMarker, ContinueMarker},
    object::Object,
};
use crate::{
    lexer::TokenType as Tk,
    parser::{Expr, Lit, Stmt},
    run::runtime_report,
    runtime_bail,
};

impl Expr {
    #[allow(clippy::too_many_lines)]
    pub fn eval(self, env: &RcCell<Env>) -> Result<Object> {
        #[allow(clippy::enum_glob_use)]
        use Object::*;

        match self {
            Expr::Assign { name, val } => {
                let (pos, name) = (name.pos, &name.lexeme);
                Env::lookup(env, name)
                    .with_context(|| {
                        runtime_report(
                            pos,
                            "while evaluating an Assignment expression",
                            format!("identifier `{}` is undefined", name),
                        )
                    })
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
                (ty, lhs, rhs) => runtime_bail!(
                    op.pos,
                    "while evaluating a Binary expression",
                    "binary operator `{:?}` undefined for ({:?}, {:?})",
                    ty,
                    lhs,
                    rhs,
                ),
            }),
            Expr::Call { callee, args, end } => {
                let callee = callee.eval(env)?;
                let args = args.into_iter().map(|i| i.eval(env)).try_collect()?;
                let res = match callee {
                    Object::NativeFn(closure) => closure.apply(args)?,
                    Object::ForeignFn(f) => f(args)?,
                    obj => runtime_bail!(
                        end.pos,
                        "while evaluating a function Call expression",
                        "the object `{}` is not callable",
                        obj,
                    ),
                };
                Ok(res)
            }
            Expr::Get { obj, name } => todo!(),
            Expr::Grouping(expr) => expr.eval(env),
            Expr::Literal(lit) => Ok(lit.into()),
            Expr::Logical { lhs, op, rhs } => match op.ty {
                Tk::And => {
                    let lhs = lhs.eval(env)?;
                    if lhs.to_bool() {
                        rhs.eval(env)
                    } else {
                        Ok(lhs)
                    }
                }
                Tk::Or => {
                    let lhs = lhs.eval(env)?;
                    if lhs.to_bool() {
                        Ok(lhs)
                    } else {
                        rhs.eval(env)
                    }
                }
                _ => unreachable!(),
            },
            Expr::Set { obj, name, to } => todo!(),
            Expr::Super { kw, method } => todo!(),
            Expr::This(_) => todo!(),
            Expr::Unary { op, rhs } => match op.ty {
                Tk::Bang => Ok(Object::Bool(!rhs.eval(env)?.to_bool())),
                Tk::Minus => {
                    let rhs = rhs.eval(env)?;
                    let rhs = -rhs.try_conv::<f64>().with_context(|| {
                        let err_msg = format!(
                            "unary operator `{:?}` undefined for the given object",
                            op.ty
                        );
                        runtime_report(op.pos, "while evaluating an Unary expression", err_msg)
                    })?;
                    Ok(Object::Number(rhs))
                }
                _ => unreachable!(),
            },
            Expr::Variable(name) => {
                let (pos, name) = (name.pos, &name.lexeme);
                Env::lookup(env, name).with_context(|| {
                    runtime_report(
                        pos,
                        "while evaluating a Variable expression",
                        format!("identifier `{}` is undefined", name),
                    )
                })
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
            Stmt::Fun { name, params, body } => {
                let name = &name.lexeme;
                let closure = Object::NativeFn(Closure {
                    name: name.into(),
                    params,
                    body,
                    env: Rc::clone(env),
                });
                env.borrow_mut().insert_val(name, closure);
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
            Stmt::Return { kw, val } => todo!(),
            Stmt::Var { name, init } => {
                let init = init
                    .map(|init| init.eval(env))
                    .transpose()?
                    .unwrap_or_default();
                env.borrow_mut().insert_val(&name.lexeme, init);
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
