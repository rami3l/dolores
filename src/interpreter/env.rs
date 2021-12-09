use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use super::Object;

pub type RcCell<T> = Arc<Mutex<T>>;

#[derive(Debug, Default)]
pub struct Env {
    pub dict: HashMap<String, Object>,
    pub outer: Option<RcCell<Env>>,
}

impl Env {
    #[must_use]
    pub fn new(dict: HashMap<String, Object>) -> Self {
        Env { dict, outer: None }
    }

    #[must_use]
    pub fn from_outer(outer: &RcCell<Env>) -> Self {
        Env {
            dict: HashMap::new(),
            outer: Some(Arc::clone(outer)),
        }
    }

    #[must_use]
    pub fn shared(self) -> RcCell<Self> {
        Arc::new(Mutex::new(self))
    }

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
    pub fn lookup(this: &RcCell<Env>, ident: &str) -> Option<Object> {
        Self::lookup_with_env(this, ident).map(|(_, obj)| obj)
    }

    pub fn insert_val(&mut self, ident: &str, defn: Object) {
        self.dict.insert(ident.into(), defn);
    }

    pub fn set_val(this: &RcCell<Env>, ident: &str, defn: Object) {
        Self::lookup_with_env(this, ident)
            .map_or_else(|| Arc::clone(this), |(that, _)| that)
            .lock()
            .insert_val(ident, defn);
    }
}
