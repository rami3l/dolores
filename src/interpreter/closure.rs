use std::hash::{Hash, Hasher};

use uuid::Uuid;

use super::{Env, RcCell};
use crate::{lexer::Token, parser::Stmt};

#[derive(Debug, Clone)]
pub struct Closure {
    pub uid: Uuid,
    pub name: Option<String>,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub env: RcCell<Env>,
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
