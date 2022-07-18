use std::{
    hash::{Hash, Hasher},
    mem, ptr,
};

use anyhow::Result;
use gc::{Finalize, Gc, Trace};
use itertools::izip;
use tap::prelude::*;

use super::{Env, Instance, Interpreter, MutCell, Object, ReturnMarker};
use crate::{lexer::Token, parser::Stmt};

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct Closure {
    pub(crate) name: Option<String>,
    #[unsafe_ignore_trace]
    pub(crate) params: Vec<Token>,
    #[unsafe_ignore_trace]
    pub(crate) body: Vec<Stmt>,
    pub(crate) env: MutCell<Env>,
    is_init: bool,
}

impl Closure {
    pub(crate) fn new<'n>(
        name: impl Into<Option<&'n str>>,
        params: impl IntoIterator<Item = Token>,
        body: impl IntoIterator<Item = Stmt>,
        env: &MutCell<Env>,
    ) -> Self {
        Self {
            name: name.into().map(str::to_owned),
            params: params.into_iter().collect(),
            body: body.into_iter().collect(),
            env: Gc::clone(env),
            is_init: false,
        }
    }

    pub(crate) fn new_init<'n>(
        name: impl Into<Option<&'n str>>,
        params: impl IntoIterator<Item = Token>,
        body: impl IntoIterator<Item = Stmt>,
        env: &MutCell<Env>,
    ) -> Self {
        Self::new(name, params, body, env).tap_mut(|it| it.is_init = true)
    }

    #[must_use]
    pub(crate) fn bind(self, instance: Instance) -> Self {
        let mut env = Env::from_outer(&self.env);
        env.insert_val("this", Object::Instance(instance));
        self.tap_mut(|it| it.env = env.shared())
    }

    pub(crate) fn apply(
        mut self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object> {
        // Temporarily switch into the scope environment...
        let old_env = Gc::clone(&interpreter.env);
        interpreter.env = Env::from_outer(&self.env).shared();
        let (expected_len, got_len) = (self.params.len(), args.len());
        if expected_len != got_len {
            anyhow::bail!(
                "[..] unexpected number of parameters (expected {}, got {})",
                expected_len,
                got_len
            )
        }
        izip!(self.params.iter(), args).for_each(|(ident, defn)| {
            interpreter.env.borrow_mut().insert_val(&ident.lexeme, defn);
        });
        let res = self
            .body
            .pipe_ref_mut(mem::take)
            .into_iter()
            .try_for_each(|it| interpreter.exec(it));
        // Switch back...
        interpreter.env = old_env;
        match res {
            Err(e) if e.is::<ReturnMarker>() => Ok(e.downcast::<ReturnMarker>().unwrap().0),
            e => {
                e?;
                if self.is_init {
                    // Special case: for initializers, we implicitly return `this`.
                    // This is actually not quite elegant as it adds a branch to all closure
                    // applications, penalizing the overall performance.
                    // See: <https://www.craftinginterpreters.com/classes.html#invoking-init-directly>
                    Env::lookup_dict(&self.env, "this").ok_or_else(|| anyhow::anyhow!(
                        "Internal Error while applying an initializer Closure: `this` not found in closure environment",
                    ))
                } else {
                    Ok(Object::Nil)
                }
            }
        }
    }
}

impl Hash for Closure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.params.hash(state);
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for Closure {}
