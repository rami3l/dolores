use std::fmt::Display;

use anyhow::{bail, Result};

use crate::parser::Lit;

#[derive(Debug, Clone)]
pub enum Object {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
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
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::enum_glob_use)]
        use Object::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Nil, _) | (_, Nil) => false,
            (Bool(l0), Bool(r0)) => l0 == r0,
            (Number(l0), Number(r0)) => l0 == r0,
            (Str(l0), Str(r0)) => l0 == r0,
            _ => unreachable!(),
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

impl From<Object> for bool {
    fn from(obj: Object) -> Self {
        !matches!(obj, Object::Nil | Object::Bool(false))
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
