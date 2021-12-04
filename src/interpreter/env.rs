use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::Object;

pub type RcCell<T> = Rc<RefCell<T>>;

pub fn rc_cell_new<T>(inner: T) -> RcCell<T> {
    Rc::new(RefCell::new(inner))
}

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
            outer: Some(Rc::clone(outer)),
        }
    }

    fn lookup_with_env(this: &RcCell<Env>, ident: &str) -> Option<(RcCell<Env>, Object)> {
        if let Some(obj) = Rc::clone(this).borrow().dict.get(ident) {
            return Some((Rc::clone(this), obj.clone()));
        }
        this.borrow()
            .outer
            .as_ref()
            .and_then(|o| Self::lookup_with_env(o, ident))
    }

    pub fn lookup(this: &RcCell<Env>, ident: &str) -> Option<Object> {
        Self::lookup_with_env(this, ident).map(|(_, obj)| obj)
    }

    pub fn insert_val(&mut self, ident: &str, defn: Object) {
        self.dict.insert(ident.into(), defn);
    }

    pub fn set_val(this: &RcCell<Env>, ident: &str, defn: Object) {
        Self::lookup_with_env(this, ident)
            .map_or_else(|| Rc::clone(this), |(that, _)| that)
            .borrow_mut()
            .insert_val(ident, defn);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn basic() {
        let outer = rc_cell_new(Env::default());
        let inner = rc_cell_new(Env::from_outer(&outer));
        outer.borrow_mut().insert_val("a", Object::Number(42.));
        Env::set_val(&inner, "a", Object::Bool(false));
        Env::set_val(&inner, "foo", Object::Number(114.));
        dbg!(&inner);
        dbg!(Env::lookup_with_env(&inner, "a"));
        dbg!(Env::lookup_with_env(&inner, "foo"));
        todo!()
    }
}
