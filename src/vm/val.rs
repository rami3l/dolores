use std::ops::Add;

use anyhow::Result;

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
        match (self, other) {
            (Num(x), Num(y)) => Ok(Num(x + y)),
            // TODO: add error handling
            _ => todo!(),
        }
    }
}
