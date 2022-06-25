use std::collections::HashMap;

use gc::{Finalize, Gc, Trace};

use super::Object;
use crate::util::{rc_cell_of, MutCell};

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct Env {
    pub(crate) dict: HashMap<String, Object>,
    pub(crate) outer: Option<MutCell<Env>>,
}

impl Env {
    #[must_use]
    pub(crate) fn new(dict: HashMap<String, Object>) -> Self {
        Self { dict, outer: None }
    }

    #[must_use]
    pub(crate) fn from_outer(outer: &MutCell<Env>) -> Self {
        Self {
            dict: HashMap::new(),
            outer: Some(Gc::clone(outer)),
        }
    }

    #[must_use]
    pub(crate) fn outer_nth(this: &MutCell<Env>, n: usize) -> Option<MutCell<Self>> {
        std::iter::successors(Some(Gc::clone(this)), |env| env.borrow().outer.clone()).nth(n)
    }

    #[must_use]
    pub(crate) fn shared(self) -> MutCell<Self> {
        rc_cell_of(self)
    }

    /*
    fn lookup_with_env(this: &RcCell<Env>, ident: &str) -> Option<(RcCell<Env>, Object)> {
        if let Some(obj) = Gc::clone(this).borrow().dict.get(ident) {
            return Some((Gc::clone(this), obj.clone()));
        }
        this.borrow()
            .outer
            .as_ref()
            .and_then(|o| Self::lookup_with_env(o, ident))
    }

    #[must_use]
    pub(crate)fn lookup(this: &RcCell<Env>, ident: &str) -> Option<Object> {
        Self::lookup_with_env(this, ident).map(|(_, obj)| obj)
    }
    */

    #[must_use]
    pub(crate) fn lookup_dict(this: &MutCell<Env>, ident: &str) -> Option<Object> {
        this.borrow().dict.get(ident).cloned()
    }

    pub(crate) fn insert_val(&mut self, ident: &str, val: Object) {
        self.dict.insert(ident.into(), val);
    }

    /*
    pub(crate)fn set_val(this: &RcCell<Env>, ident: &str, val: Object) {
        Self::lookup_with_env(this, ident)
            .map_or_else(|| Gc::clone(this), |(that, _)| that)
            .borrow()
            .insert_val(ident, val);
    }
    */
}

impl Default for Env {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
