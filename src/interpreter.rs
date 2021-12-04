use anyhow::{bail, Result};
use tap::prelude::*;

use crate::{
    lexer::TokenType,
    parser::{Expr, Lit},
};

#[derive(Debug, Clone)]
pub(crate) enum Object {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::enum_glob_use)]
        use Object::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Nil, _) => false,
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
                obj
            ),
        }
    }
}

impl Expr {
    pub fn eval(self) -> Result<Object> {
        #[allow(clippy::enum_glob_use)]
        use Object::*;
        use TokenType as Tk;

        match self {
            Expr::Assign { name, val } => todo!(),
            Expr::Binary { lhs, op, rhs } => {
                let (lhs, rhs) = (lhs.eval()?, rhs.eval()?);
                Ok(match (op.ty, lhs, rhs) {
                    (Tk::Plus, Str(lhs), Str(rhs)) => Str(lhs + &rhs),
                    (Tk::Plus, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Number(lhs.try_conv::<f64>()? + rhs.try_conv::<f64>()?)
                    }
                    (Tk::Minus, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Number(lhs.try_conv::<f64>()? - rhs.try_conv::<f64>()?)
                    }
                    (Tk::Star, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Number(lhs.try_conv::<f64>()? * rhs.try_conv::<f64>()?)
                    }
                    (Tk::Slash, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Number(lhs.try_conv::<f64>()? / rhs.try_conv::<f64>()?)
                    }
                    (Tk::Greater, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Bool(lhs.try_conv::<f64>()? > rhs.try_conv::<f64>()?)
                    }
                    (
                        Tk::GreaterEqual,
                        lhs @ (Bool(_) | Number(_)),
                        rhs @ (Bool(_) | Number(_)),
                    ) => Bool(lhs.try_conv::<f64>()? >= rhs.try_conv::<f64>()?),
                    (Tk::Less, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Bool(lhs.try_conv::<f64>()? < rhs.try_conv::<f64>()?)
                    }
                    (Tk::LessEqual, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                        Bool(lhs.try_conv::<f64>()? <= rhs.try_conv::<f64>()?)
                    }
                    (Tk::EqualEqual, lhs, rhs) => Bool(lhs == rhs),
                    (Tk::BangEqual, lhs, rhs) => Bool(lhs != rhs),

                    (op, lhs, rhs) => bail!(
                        "Runtime Error: binary operator `{:?}` undefined for ({:?}, {:?})",
                        op,
                        lhs,
                        rhs
                    ),
                })
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => todo!(),
            Expr::Get { obj, name } => todo!(),
            Expr::Grouping(expr) => expr.eval(),
            Expr::Literal(lit) => Ok(lit.into()),
            Expr::Logical { lhs, op, rhs } => todo!(),
            Expr::Set { obj, name, to } => todo!(),
            Expr::Super { kw, method } => todo!(),
            Expr::This(_) => todo!(),
            Expr::Unary { op, rhs } => match op.ty {
                Tk::Bang => Ok(Object::Bool(!rhs.eval()?.try_into()?)),
                Tk::Minus => Ok(Object::Number(-rhs.eval()?.try_into()?)),
                _ => unreachable!(),
            },
            Expr::Variable(_) => todo!(),
        }
    }
}
