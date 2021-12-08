use anyhow::Result;
use itertools::izip;

use super::{Env, Object, RcCell};
use crate::parser::Stmt;

#[derive(Debug, Clone)]
pub struct Closure {
    pub params: Vec<String>,
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
    pub(crate) fn apply(&self, args: Vec<Object>) -> Result<Object> {
        let env = &Env::from_outer(&self.env).shared();
        if self.params.len() != args.len() {
            todo!()
            // TODO: Should bail here.
        }
        izip!(self.params.iter(), args)
            .for_each(|(ident, defn)| env.borrow_mut().insert_val(ident, defn));
        /*
        match self.body.into_iter().try_for_each(|i| i.eval(env)) {
            Err(e) if e.is::<ReturnMarker>() => todo!("handle return value"),
            _ => todo!(),
        }
        */
        todo!("return a value")
    }
}
