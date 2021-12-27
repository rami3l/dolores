use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use itertools::{izip, Itertools};
use tap::prelude::*;
use uuid::Uuid;

use super::{Env, Interpreter, Object};
use crate::{
    error::runtime_report,
    interpreter::{Closure, ReturnMarker},
    lexer::{Token, TokenType as Tk},
    parser::Expr,
    runtime_bail,
};

impl Interpreter {
    #[allow(clippy::too_many_lines)]
    pub fn eval(&mut self, expr: Expr) -> Result<Object> {
        #[allow(clippy::enum_glob_use)]
        use Object::*;

        let env = &Arc::clone(&self.env);
        match expr {
            Expr::Assign { name, val } => {
                let dist = self.locals.get(&name).copied();
                let ident = &name.lexeme;
                self.lookup(&name)
                    .with_context(|| {
                        runtime_report(
                            name.pos,
                            "while evaluating an Assignment expression",
                            format!("identifier `{}` is undefined", name),
                        )
                    })
                    .and_then(|_| {
                        let val = self.eval(*val)?;
                        if let Some(dist) = dist {
                            self.assign_at(dist, ident, val.clone())?;
                        } else {
                            self.globals.lock().insert_val(ident, val.clone());
                        }
                        Ok(val)
                    })
            }
            Expr::Binary { lhs, op, rhs } => {
                Ok(match (op.ty, self.eval(*lhs)?, self.eval(*rhs)?) {
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
                    (
                        Tk::GreaterEqual,
                        lhs @ (Bool(_) | Number(_)),
                        rhs @ (Bool(_) | Number(_)),
                    ) => Bool(lhs.try_conv::<f64>()? >= rhs.try_conv::<f64>()?),
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
                })
            }
            Expr::Call { callee, args, end } => {
                let callee = self.eval(*callee)?;
                let args: Vec<Object> = args.into_iter().map(|i| self.eval(i)).try_collect()?;
                let res = match callee {
                    Object::NativeFn(clos) => {
                        // Temporarily switch into the scope environment...
                        let old_env = Arc::clone(&self.env);
                        self.env = Env::from_outer(&clos.env).shared();
                        let (expected_len, got_len) = (clos.params.len(), args.len());
                        if expected_len != got_len {
                            runtime_bail!(
                                // TODO: Fix position maybe?
                                (0, 0),
                                "while evaluating a function Call expression",
                                "unexpected number of parameters (expected {}, got {})",
                                expected_len,
                                got_len
                            )
                        }
                        izip!(clos.params.iter(), args).for_each(|(ident, defn)| {
                            self.env.lock().insert_val(&ident.lexeme, defn);
                        });
                        let res = clos.body.into_iter().try_for_each(|it| self.exec(it));
                        // Switch back...
                        self.env = old_env;
                        match res {
                            Err(e) if e.is::<ReturnMarker>() => {
                                e.downcast::<ReturnMarker>().unwrap().0
                            }
                            e => {
                                e?;
                                Object::Nil
                            }
                        }
                    }
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
            Expr::Grouping(expr) => self.eval(*expr),
            Expr::Lambda { params, body } => Ok(Object::NativeFn(Closure {
                uid: Uuid::new_v4(),
                name: None,
                params,
                body,
                env: Arc::clone(env),
            })),
            Expr::Literal(lit) => Ok(lit.into()),
            Expr::Logical { lhs, op, rhs } => match op.ty {
                Tk::And => {
                    let lhs = self.eval(*lhs)?;
                    if lhs.to_bool() {
                        self.eval(*rhs)
                    } else {
                        Ok(lhs)
                    }
                }
                Tk::Or => {
                    let lhs = self.eval(*lhs)?;
                    if lhs.to_bool() {
                        Ok(lhs)
                    } else {
                        self.eval(*rhs)
                    }
                }
                _ => unreachable!(),
            },
            Expr::Set { obj, name, to } => todo!(),
            Expr::Super { kw, method } => todo!(),
            Expr::This(_) => todo!(),
            Expr::Unary { op, rhs } => match op.ty {
                Tk::Bang => Ok(Object::Bool(!self.eval(*rhs)?.to_bool())),
                Tk::Minus => {
                    let rhs = self.eval(*rhs)?;
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
            Expr::Variable(name) => self.lookup(&name).with_context(|| {
                runtime_report(
                    name.pos,
                    "while evaluating a Variable expression",
                    format!("identifier `{}` is undefined", name),
                )
            }),
        }
    }

    /// Look up a variable definition in the current evaluation context.
    fn lookup(&self, name: &Token) -> Option<Object> {
        let ident = &name.lexeme;
        self.locals.get(name).map_or_else(
            || Env::lookup_dict(&self.globals, ident),
            |&dist| Env::outer_nth(&self.env, dist).and_then(|it| Env::lookup_dict(&it, ident)),
        )
    }

    fn assign_at(&self, dist: usize, ident: &str, val: Object) -> Result<()> {
        let target = Env::outer_nth(&self.env, dist).ok_or_else(|| {
            anyhow!(
                "Internal Error while assigning to Variable `{}`: distance ({}) out of range",
                ident,
                dist,
            )
        })?;
        target.lock().insert_val(ident, val);
        Ok(())
    }
}
