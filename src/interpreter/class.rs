use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ptr,
};

use gc::{Finalize, Trace};

use super::{MutCell, Object};
use crate::util::rc_cell_of;

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct Class {
    pub(crate) name: String,
    pub(crate) superclass: Option<Box<Class>>,
    pub(crate) methods: MutCell<HashMap<String, Object>>,
}

impl Class {
    #[must_use]
    pub(crate) fn new(
        name: &str,
        superclass: impl Into<Option<Class>>,
        methods: HashMap<String, Object>,
    ) -> Self {
        Self {
            name: name.into(),
            superclass: superclass.into().map(Box::new),
            methods: rc_cell_of(methods),
        }
    }

    #[must_use]
    pub(crate) fn method(&self, name: &str) -> Option<Object> {
        self.methods
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.superclass.as_ref().and_then(|sup| sup.method(name)))
    }
}

impl Hash for Class {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for Class {}

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct Instance {
    #[unsafe_ignore_trace]
    pub(crate) class: Class,
    pub(crate) fields: MutCell<HashMap<String, Object>>,
}

impl From<Class> for Instance {
    fn from(class: Class) -> Self {
        Self {
            class,
            fields: rc_cell_of(HashMap::default()),
        }
    }
}

impl Instance {
    #[must_use]
    pub(crate) fn get(&self, name: &str) -> Option<Object> {
        self.fields.borrow().get(name).cloned().or_else(|| {
            self.class.method(name).map(|it| {
                if let Object::NativeFn(clos) = &it {
                    Object::NativeFn(clos.clone().bind(self.clone()))
                } else {
                    unreachable!()
                }
            })
        })
    }

    #[allow(clippy::must_use_candidate)]
    pub(crate) fn set(&self, name: &str, to: Object) -> Option<Object> {
        self.fields.borrow_mut().insert(name.into(), to)
    }
}

impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::addr_of!(self).hash(state);
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for Instance {}
