use std::{collections::HashMap, sync::Arc};

use super::Object;
use crate::util::{rc_cell_of, RcCell};

#[derive(Debug, Default, Clone)]
pub(crate) struct Env {
    pub(crate) dict: HashMap<String, Object>,
    pub(crate) outer: Option<RcCell<Env>>,
}

impl Env {
    #[must_use]
    pub(crate) fn new(dict: HashMap<String, Object>) -> Self {
        Self { dict, outer: None }
    }

    #[must_use]
    pub(crate) fn from_outer(outer: &RcCell<Env>) -> Self {
        Self {
            dict: HashMap::new(),
            outer: Some(Arc::clone(outer)),
        }
    }

    #[must_use]
    pub(crate) fn outer_nth(this: &RcCell<Env>, n: usize) -> Option<RcCell<Self>> {
        std::iter::successors(Some(Arc::clone(this)), |env| env.lock().outer.clone()).nth(n)
    }

    #[must_use]
    pub(crate) fn shared(self) -> RcCell<Self> {
        rc_cell_of(self)
    }

    /*
    fn lookup_with_env(this: &RcCell<Env>, ident: &str) -> Option<(RcCell<Env>, Object)> {
        if let Some(obj) = Arc::clone(this).lock().dict.get(ident) {
            return Some((Arc::clone(this), obj.clone()));
        }
        this.lock()
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
    pub(crate) fn lookup_dict(this: &RcCell<Env>, ident: &str) -> Option<Object> {
        this.lock().dict.get(ident).cloned()
    }

    pub(crate) fn insert_val(&mut self, ident: &str, val: Object) {
        self.dict.insert(ident.into(), val);
    }

    /*
    pub(crate)fn set_val(this: &RcCell<Env>, ident: &str, val: Object) {
        Self::lookup_with_env(this, ident)
            .map_or_else(|| Arc::clone(this), |(that, _)| that)
            .lock()
            .insert_val(ident, val);
    }
    */
}
