use std::ops::Add;

use anyhow::{bail, Result};

use crate::error::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Val {
    Bool(bool),
    // BoundMethod(GcRef<BoundMethod>),
    // Class(GcRef<Class>),
    // Closure(GcRef<Closure>),
    // Function(GcRef<Function>),
    // Instance(GcRef<Instance>),
    // NativeFunction(NativeFunction),
    Nil,
    Num(f64),
    // Str(GcRef<String>),
}

impl Default for Val {
    fn default() -> Self {
        Self::Nil
    }
}

impl Add for Val {
    type Output = Result<Self>;

    fn add(self, other: Self) -> Result<Self> {
        use Val::*;
        Ok(match (self, other) {
            (Num(x), Num(y)) => Num(x + y),
            // TODO: add String concat
            _ => bail!(Error::OperationUnimplementedError {
                op: "+".into(),
                args: format!("({self:?}, {other:?})"),
            }),
        })
    }
}
