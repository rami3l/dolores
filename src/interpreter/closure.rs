use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use anyhow::Result;
use itertools::izip;
use uuid::Uuid;

use super::{Env, Instance, Interpreter, Object, RcCell, ReturnMarker};
use crate::{lexer::Token, parser::Stmt};

#[derive(Debug, Clone)]
pub struct Closure {
    pub uid: Uuid,
    pub name: Option<String>,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub env: RcCell<Env>,
    is_init: bool,
}

impl Closure {
    pub fn new<'n>(
        name: impl Into<Option<&'n str>>,
        params: impl IntoIterator<Item = Token>,
        body: impl IntoIterator<Item = Stmt>,
        env: &RcCell<Env>,
    ) -> Self {
        Closure {
            uid: Uuid::new_v4(),
            name: name.into().map(str::to_owned),
            params: params.into_iter().collect(),
            body: body.into_iter().collect(),
            env: Arc::clone(env),
            is_init: false,
        }
    }

    pub fn new_init<'n>(
        name: impl Into<Option<&'n str>>,
        params: impl IntoIterator<Item = Token>,
        body: impl IntoIterator<Item = Stmt>,
        env: &RcCell<Env>,
    ) -> Self {
        Closure {
            is_init: true,
            ..Closure::new(name, params, body, env)
        }
    }

    #[must_use]
    pub fn bind(self, instance: Instance) -> Self {
        let mut env = Env::from_outer(&self.env);
        env.insert_val("this", Object::Instance(instance));
        Closure {
            env: env.shared(),
            ..self
        }
    }

    pub fn apply(self, interpreter: &mut Interpreter, args: Vec<Object>) -> Result<Object> {
        // Temporarily switch into the scope environment...
        let old_env = Arc::clone(&interpreter.env);
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
            interpreter.env.lock().insert_val(&ident.lexeme, defn);
        });
        let res = self
            .body
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
        self.uid.hash(state);
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Closure {}
