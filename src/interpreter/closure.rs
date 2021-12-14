use std::hash::{Hash, Hasher};

use anyhow::Result;
use itertools::izip;
use uuid::Uuid;

use super::{Env, Object, RcCell, ReturnMarker};
use crate::{lexer::Token, parser::Stmt, runtime_bail};

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
