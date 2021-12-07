use anyhow::Result;
use itertools::izip;

use super::{Env, Object, RcCell};
use crate::parser::Stmt;

pub struct Closure {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub env: RcCell<Env>,
}

impl Closure {
    fn apply(&self, args: Vec<Object>) -> Result<Object> {
        let env = &Env::from_outer(&self.env).shared();
        if self.params.len() != args.len() {
            todo!()
            // TODO: Should bail here.
        }
        izip!(self.params, args)
            .for_each(|(ident, defn)| env.borrow_mut().insert_val(&ident, defn));
        /*
        match self.body.into_iter().try_for_each(|i| i.eval(env)) {
            Err(e) if e.is::<ReturnMarker>() => todo!("handle return value"),
            _ => todo!(),
        }
        */
        todo!("return a value")
    }
}
