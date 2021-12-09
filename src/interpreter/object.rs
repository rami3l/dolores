use std::fmt::Display;

use anyhow::{bail, Result};

use super::{Closure, Env, RcCell};
use crate::parser::Lit;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
    NativeFn(Closure),
    ForeignFn(fn(Vec<Object>) -> Result<Object>),
}

impl Default for Object {
    fn default() -> Self {
        Object::Nil
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Number(n) => write!(f, "{}", n.to_string().trim_end_matches(".0")),
            Object::Str(s) => write!(f, r#""{}""#, s),
            Object::NativeFn(clos) => write!(
                f,
                "<fun: {}@native>",
                clos.name.as_ref().map_or("_", |s| s as _),
            ),
            Object::ForeignFn(_) => write!(f, "<fun: _@foreign>"),
        }
    }
}

impl From<Lit> for Object {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::Nil => Object::Nil,
            Lit::Bool(b) => Object::Bool(b),
            Lit::Number(n) => Object::Number(n),
            Lit::Str(s) => Object::Str(s),
        }
    }
}

impl From<&Object> for bool {
    fn from(obj: &Object) -> Self {
        !matches!(obj, Object::Nil | Object::Bool(false))
    }
}

impl Object {
    #[must_use]
    pub fn to_bool(&self) -> bool {
        self.into()
    }
}

impl TryFrom<Object> for f64 {
    type Error = anyhow::Error;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj {
            Object::Number(n) => Ok(n),
            Object::Bool(b) => Ok(f64::from(b as u8)),
            obj => bail!(
                "Runtime Error: object `{:?}` cannot be converted to Number",
                obj,
            ),
        }
    }
}
