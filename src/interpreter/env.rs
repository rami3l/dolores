use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::Object;

type RcCell<T> = Rc<RefCell<T>>;

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

    fn lookup_with_env(this: RcCell<Env>, sym: &str) -> Option<(RcCell<Env>, Object)> {
        if let Some(obj) = Rc::clone(&this).borrow().dict.get(sym) {
            return Some((this, obj.clone()));
        }
        this.borrow()
            .outer
            .as_ref()
            .and_then(|o| Self::lookup_with_env(Rc::clone(o), sym))
    }

    pub fn lookup(this: RcCell<Env>, sym: &str) -> Option<Object> {
        Self::lookup_with_env(this, sym).map(|(_, obj)| obj)
    }

    pub fn insert_val(&mut self, sym: &str, defn: Object) -> Option<Object> {
        self.dict.insert(sym.into(), defn)
    }

    pub fn set_val(this: RcCell<Env>, sym: &str, defn: Object) {
        Self::lookup_with_env(Rc::clone(&this), sym)
            .map_or(this, |(that, _)| that)
            .borrow_mut()
            .insert_val(sym, defn);
    }
}
