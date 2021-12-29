use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use uuid::Uuid;

use super::{Object, RcCell};
use crate::util::rc_cell_of;

#[derive(Debug, Clone)]
pub struct Class {
    pub uid: Uuid,
    pub name: String,
    // pub superclass: Arc<Class>,
    pub methods: RcCell<HashMap<String, Object>>,
}

impl Class {
    #[must_use]
    pub fn new(name: &str, methods: HashMap<String, Object>) -> Self {
        Class {
            uid: Uuid::new_v4(),
            name: name.into(),
            methods: rc_cell_of(methods),
        }
    }
}

impl Hash for Class {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Class {}

#[derive(Debug, Clone)]
pub struct Instance {
    pub uid: Uuid,
    pub class: Class,
    pub fields: RcCell<HashMap<String, Object>>,
}

impl From<Class> for Instance {
    fn from(class: Class) -> Self {
        Instance {
            uid: Uuid::new_v4(),
            class,
            fields: rc_cell_of(HashMap::default()),
        }
    }
}

impl Instance {
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Object> {
        self.fields
            .lock()
            .get(name)
            .cloned()
            .or_else(|| self.method(name))
    }

    #[must_use]
    pub fn method(&self, name: &str) -> Option<Object> {
        self.class.methods.lock().get(name).cloned().map(|it| {
            if let Object::NativeFn(clos) = it {
                Object::NativeFn(clos.bind(self.clone()))
            } else {
                unreachable!()
            }
        })
    }

    #[allow(clippy::must_use_candidate)]
    pub fn set(&self, name: &str, to: Object) -> Option<Object> {
        self.fields.lock().insert(name.into(), to)
    }
}

impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Instance {}
