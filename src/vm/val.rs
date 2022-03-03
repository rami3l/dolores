use std::ops::Add;

use anyhow::Result;

#[derive(Debug, Clone)]
pub(crate) enum Val {
    Num(f64),
}

impl Add for Val {
    type Output = Result<Self>;

    fn add(self, other: Self) -> Result<Self> {
        use Val::*;
        match (self, other) {
            (Num(x), Num(y)) => Ok(Num(x + y)),
        }
    }
}
