use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use uuid::Uuid;

use super::{Env, RcCell};
use crate::{lexer::Token, parser::Stmt};

#[derive(Debug, Clone)]
pub struct Closure {
    uid: Uuid,
    pub name: Option<String>,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub env: RcCell<Env>,
}

impl Closure {
    pub fn new<'n>(
        name: impl Into<Option<&'n str>>,
        params: impl IntoIterator<Item = Token>,
        body: impl IntoIterator<Item = Stmt>,
        env: &RcCell<Env>,
    ) -> Self {
        Closure {
            uid: Uuid::new_v4(),
            name: name.into().map(str::to_owned),
            params: params.into_iter().collect(),
            body: body.into_iter().collect(),
            env: Arc::clone(env),
        }
    }
}

impl Hash for Closure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Closure {}
