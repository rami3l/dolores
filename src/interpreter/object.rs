use std::{fmt::Display, ptr};

use anyhow::{bail, Result};
use gc::{Finalize, Trace};

use super::{Class, Closure, Instance};
use crate::parser::Lit;

#[derive(Debug, Clone, PartialEq, Trace, Finalize)]
pub(crate) enum Object {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
    NativeFn(Closure),
    ForeignFn(fn(Vec<Object>) -> Result<Object>),
    Class(Class),
    Instance(Instance),
}

impl Default for Object {
    fn default() -> Self {
        Self::Nil
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Number(n) => write!(f, "{}", n.to_string().trim_end_matches(".0")),
            Object::Str(s) => write!(f, r#""{}""#, s),
            Object::NativeFn(clos) => {
                if let Some(name) = &clos.name {
                    write!(f, "<fun: {}@native>", name)
                } else {
                    write!(f, "<fun: {:?}@native>", ptr::addr_of!(clos))
                }
            }
            Object::ForeignFn(_) => write!(f, "<fun: _@foreign>"),
            Object::Class(c) => write!(f, "<class: {}>", c.name),
            Object::Instance(i) => write!(f, "<instance: {:?}@{}>", i as *const _, i.class.name),
        }
    }
}

impl From<Lit> for Object {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::Nil => Self::Nil,
            Lit::Bool(b) => Self::Bool(b),
            Lit::Number(n) => Self::Number(n),
            Lit::Str(s) => Self::Str(s),
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
    pub(crate) fn to_bool(&self) -> bool {
        self.into()
    }
}

impl TryFrom<&Object> for f64 {
    type Error = anyhow::Error;

    fn try_from(obj: &Object) -> Result<Self, Self::Error> {
        match obj {
            Object::Number(n) => Ok(*n),
            Object::Bool(b) => Ok(f64::from(u8::from(*b))),
            obj => bail!(
                "Runtime Error: object `{:?}` cannot be converted to Number",
                obj,
            ),
        }
    }
}
