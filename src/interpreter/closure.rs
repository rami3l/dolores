use anyhow::Result;
use itertools::izip;

use super::{jump::ReturnMarker, Env, Object, RcCell};
use crate::{lexer::Token, parser::Stmt, runtime_bail};

#[derive(Debug, Clone)]
pub struct Closure {
    pub name: Option<String>,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub env: RcCell<Env>,
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Provide UID for true PartialEq
        false
    }
}

impl Closure {
    pub(crate) fn apply(self, args: Vec<Object>) -> Result<Object> {
        let env = &Env::from_outer(&self.env).shared();
        let (expected_len, got_len) = (self.params.len(), args.len());
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
        izip!(self.params.iter(), args)
            .for_each(|(ident, defn)| env.lock().insert_val(&ident.lexeme, defn));
        match self.body.into_iter().try_for_each(|i| i.eval(env)) {
            Err(e) if e.is::<ReturnMarker>() => return Ok(e.downcast::<ReturnMarker>().unwrap().0),
            e => e?,
        }
        Ok(Object::Nil)
    }
}
