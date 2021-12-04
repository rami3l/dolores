use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::Object;

type RcCell<T> = Rc<RefCell<T>>;

pub(crate) struct Env {
    dict: HashMap<String, Object>,
    outer: Option<RcCell<Env>>,
}
