use anyhow::{anyhow, Context, Result};
use gc::Gc;
use itertools::Itertools;
use tap::prelude::*;

use super::{Closure, Env, Instance, Interpreter, Object};
use crate::{
    error::runtime_report,
    lexer::{Token, TokenType as Tk},
    parser::Expr,
    runtime_bail,
};

impl Interpreter {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn eval(&mut self, expr: Expr) -> Result<Object> {
        let env = &Gc::clone(&self.env);
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
                            self.globals.borrow_mut().insert_val(ident, val.clone());
                        }
                        Ok(val)
                    })
            }
            Expr::Binary { lhs, op, rhs } => {
                #[allow(clippy::enum_glob_use)]
                use Object::*;
                Ok(match (op.ty, &self.eval(*lhs)?, &self.eval(*rhs)?) {
                    (Tk::Plus, Str(lhs), Str(rhs)) => Str(format!("{lhs}{rhs}")),
                    (Tk::Plus, Str(lhs), rhs) => Str(format!("{lhs}{rhs}")),
                    (Tk::Plus, lhs, Str(rhs)) => Str(format!("{lhs}{rhs}")),
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
                let res = match &callee {
                    Object::NativeFn(clos) => {
                        clos.clone().apply(self, args).with_context(|| {
                            runtime_report(
                                end.pos,
                                "while evaluating a function Call expression",
                                "",
                            )
                        })?
                    }
                    Object::ForeignFn(f) => f(args)?,
                    Object::Class(c) => {
                        let instance = Instance::from(c.clone());
                        if let Some(it) = instance.class.method("init") {
                            if let Object::NativeFn(clos) = &it {
                                clos.clone().bind(instance.clone()).apply(self, args)?;
                            } else {
                                unreachable!();
                            }
                        } else if !args.is_empty() {
                            runtime_bail!(
                                end.pos,
                                "while evaluating a new Class expression",
                                "unexpected number of parameters (expected 0, got {})",
                                args.len(),
                            );
                        }
                        Object::Instance(instance)
                    }
                    obj => runtime_bail!(
                        end.pos,
                        "while evaluating a function Call expression",
                        "the object `{}` is not callable",
                        obj,
                    ),
                };
                Ok(res)
            }
            Expr::Get { obj, name } => {
                let ctx = "while evaluating a Get expression";
                let obj = self.eval(*obj)?;
                if let Object::Instance(ref i) = obj {
                    let lexeme = &name.lexeme;
                    i.get(lexeme).with_context(|| {
                        let err_msg =
                            format!("property `{}` undefined for the given object", lexeme);
                        runtime_report(name.pos, ctx, err_msg)
                    })
                } else {
                    runtime_bail!(name.pos, ctx, "the object `{}` cannot have properties", obj);
                }
            }
            Expr::Grouping(expr) => self.eval(*expr),
            Expr::Lambda { params, body } => {
                Ok(Object::NativeFn(Closure::new(None, params, body, env)))
            }
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
            Expr::Set { obj, name, to } => {
                let ctx = "while evaluating a Set expression";
                let obj = self.eval(*obj)?;
                if let Object::Instance(ref i) = obj {
                    let lexeme = &name.lexeme;
                    let to = self.eval(*to)?;
                    i.set(lexeme, to.clone());
                    Ok(to)
                } else {
                    runtime_bail!(name.pos, ctx, "the object `{}` cannot have properties", obj)
                }
            }
            Expr::Super { kw, method } => {
                let ctx = "while evaluating a superclass method";
                let distance = *self.locals.get(&kw).with_context(|| {
                    runtime_report(kw.pos, ctx, "identifier `super` is undefined")
                })?;
                let outer_err = |dist| {
                    anyhow!(
                        "Internal Error while looking up `super`: distance ({}) out of range",
                        dist,
                    )
                };
                // When evaluating a superclass method, `this` is always bound to the class
                // where `super` appears, and `super` to its direct superclass.
                let this_env =
                    &Env::outer_nth(env, distance - 1).ok_or_else(|| outer_err(distance - 1))?;
                let this = Env::lookup_dict(this_env, "this").with_context(|| {
                    runtime_report(kw.pos, ctx, "identifier `this` is undefined")
                })?;
                let sup_env = &Env::outer_nth(this_env, 1).ok_or_else(|| outer_err(distance))?;
                let sup = Env::lookup_dict(sup_env, "super").with_context(|| {
                    runtime_report(kw.pos, ctx, "identifier `super` is undefined")
                })?;
                match (&this, &sup) {
                    (Object::Instance(this), Object::Class(sup)) => {
                        let lexeme = &method.lexeme;
                        let method = sup.method(lexeme).with_context(|| {
                            let err_msg =
                                format!("property `{}` undefined for the given object", lexeme);
                            runtime_report(method.pos, ctx, err_msg)
                        })?;
                        if let Object::NativeFn(clos) = &method {
                            Ok(Object::NativeFn(clos.clone().bind(this.clone())))
                        } else {
                            unreachable!()
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::This(kw) => self.lookup(&kw).with_context(|| {
                runtime_report(
                    kw.pos,
                    "while evaluating a This expression",
                    "identifier `this` is undefined",
                )
            }),
            Expr::Unary { op, rhs } => match op.ty {
                Tk::Bang => Ok(Object::Bool(!self.eval(*rhs)?.to_bool())),
                Tk::Minus => {
                    let rhs = &self.eval(*rhs)?;
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
        target.borrow_mut().insert_val(ident, val);
        Ok(())
    }
}
