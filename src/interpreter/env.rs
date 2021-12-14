use std::{iter, sync::Arc};

use im::HashMap;
use parking_lot::Mutex;

use super::Object;

pub type RcCell<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, Default)]
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
            dict: HashMap::default(),
            outer: Some(Arc::clone(outer)),
        }
    }

    #[must_use]
    pub fn outer_nth(&self, n: usize) -> Option<RcCell<Self>> {
        iter::successors(self.outer.clone(), |outer| outer.lock().outer.clone()).nth(n)
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

    pub fn insert_val(this: &RcCell<Env>, ident: &str, defn: Object) {
        this.lock().dict.insert(ident.into(), defn);
    }

    #[must_use]
    pub fn frozen(this: &RcCell<Env>) -> RcCell<Env> {
        this.lock().clone().shared()
    }

    pub fn set_val(this: &RcCell<Env>, ident: &str, defn: Object) {
        let target =
            Self::lookup_with_env(this, ident).map_or_else(|| Arc::clone(this), |(that, _)| that);
        Env::insert_val(&target, ident, defn);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn insert() {
        let ins = |env, ident, s: &str| Env::insert_val(env, ident, Object::Str(s.into()));
        let assert_env_s = |env, ident, expected: &str| {
            assert_eq!(Some(Object::Str(expected.into())), Env::lookup(env, ident));
        };

        let outer = &Env::new(HashMap::default()).shared();
        ins(outer, "a", "global");
        let inner = &Env::from_outer(outer).shared();
        ins(inner, "b", "bee");
        let frozen = &Env::frozen(inner);

        ins(inner, "a", "local");
        assert_env_s(outer, "a", "global");
        assert_env_s(inner, "a", "local");
        assert_env_s(frozen, "a", "global");

        ins(outer, "a", "global-ng");
        assert_env_s(outer, "a", "global-ng");
        assert_env_s(inner, "a", "local");
        assert_env_s(frozen, "a", "global-ng");

        assert_env_s(inner, "b", "bee");
        assert_env_s(frozen, "b", "bee");
        ins(inner, "b", "bee-ng");
        assert_env_s(inner, "b", "bee-ng");
        assert_env_s(frozen, "b", "bee-ng");
    }
}
