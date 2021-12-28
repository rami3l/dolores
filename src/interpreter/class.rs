use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use uuid::Uuid;

use super::Object;

#[derive(Debug, Clone)]
pub struct Class {
    pub uid: Uuid,
    pub name: String,
    // pub superclass: Arc<Class>,
    pub methods: HashMap<String, Object>,
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
