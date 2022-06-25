use anyhow::Result;
use gc::Gc;

use super::{BreakMarker, Class, Closure, ContinueMarker, Env, Interpreter, Object, ReturnMarker};
use crate::{lexer::TokenType as Tk, parser::Stmt, runtime_bail};

impl Interpreter {
    pub(crate) fn exec(&mut self, stmt: Stmt) -> Result<()> {
        let env = &Gc::clone(&self.env);
        match stmt {
            Stmt::Block(stmts) => {
                let old_env = Gc::clone(env);
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
            } => {
                let (ref env, superclass) = if let Some(it) = superclass {
                    let sup = self.eval(it)?;
                    if let Object::Class(ref sup) = sup {
                        let mut super_env = Env::from_outer(env);
                        super_env.insert_val("super", Object::Class(sup.clone()));
                        (super_env.shared(), Some(sup.clone()))
                    } else {
                        runtime_bail!(
                            name.pos,
                            "while evaluating a Class declaration",
                            "class `{}` cannot inherit from non-class value `{}`",
                            name.lexeme,
                            sup
                        )
                    }
                } else {
                    (Gc::clone(env), None)
                };
                let methods = methods
                    .into_iter()
                    .map(|it| {
                        if let Stmt::Fun { name, params, body } = it {
                            let name: &str = &name.lexeme;
                            let closure = if name == "init" {
                                Closure::new_init(name, params, body, env)
                            } else {
                                Closure::new(name, params, body, env)
                            };
                            (name.to_owned(), Object::NativeFn(closure))
                        } else {
                            unreachable!()
                        }
                    })
                    .collect();
                let class = Object::Class(Class::new(&name.lexeme, superclass, methods));
                self.env.borrow_mut().insert_val(&name.lexeme, class);
            }
            Stmt::Expression(expr) => {
                self.eval(expr)?;
            }
            Stmt::Fun { name, params, body } => {
                let name: &str = &name.lexeme;
                let closure = Object::NativeFn(Closure::new(name, params, body, env));
                env.borrow_mut().insert_val(name, closure);
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
                self.env.borrow_mut().insert_val(&name.lexeme, init);
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

    pub(crate) fn exec_stmts(&mut self, stmts: impl IntoIterator<Item = Stmt>) -> Result<()> {
        stmts.into_iter().try_for_each(|it| self.exec(it))
    }
}
