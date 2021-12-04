pub mod env;

use std::fmt::Display;

use anyhow::{bail, Result};
use tap::prelude::*;

use crate::{
    lexer::TokenType,
    parser::{Expr, Lit, Stmt},
};

#[derive(Debug, Clone)]
pub enum Object {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
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
            Expr::Binary { lhs, op, rhs } => Ok(match (op.ty, lhs.eval()?, rhs.eval()?) {
                (Tk::Plus, Str(lhs), Str(rhs)) => Str(lhs + &rhs),
                (Tk::Plus, Str(lhs), rhs) => Str(lhs + &format!("{}", rhs)),
                (Tk::Plus, lhs, Str(rhs)) => Str(format!("{}", lhs) + &rhs),
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
                (Tk::GreaterEqual, lhs @ (Bool(_) | Number(_)), rhs @ (Bool(_) | Number(_))) => {
                    Bool(lhs.try_conv::<f64>()? >= rhs.try_conv::<f64>()?)
                }
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
            }),
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

impl Stmt {
    pub fn eval(self) -> Result<()> {
        match self {
            Stmt::Block(_) => todo!(),
            Stmt::Class {
                name,
                superclass,
                methods,
            } => todo!(),
            Stmt::Expression(expr) => {
                expr.eval()?;
            }
            Stmt::Function { name, params, body } => todo!(),
            Stmt::If {
                cond,
                then_stmt,
                else_stmt,
            } => todo!(),
            Stmt::Print(expr) => println!("{}", expr.eval()?),
            Stmt::Return { kw, val } => todo!(),
            Stmt::Var { name, init } => todo!(),
            Stmt::While { cond, body } => todo!(),
        }
        Ok(())
    }
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{lexer::Lexer, parser::Parser};

    fn assert_expr_eval(src: &str, expected: &str) {
        let tokens = Lexer::new(src).analyze();
        let expr = Parser::new(tokens).expr().unwrap().eval().unwrap();
        let got = format!("{}", expr);
        assert_eq!(got, expected);
    }

    #[test]
    fn basic() {
        assert_expr_eval("2 +2", "4");
        assert_expr_eval("-6 *(-4+ -3) == 6*4 + 2  *((((9))))", "true");
        assert_expr_eval(
            "4/1 - 4/3 + 4/5 - 4/7 + 4/9 - 4/11 + 4/13 - 4/15 + 4/17 - 4/19 + 4/21 - 4/23",
            "3.058402765927333",
        );
        assert_expr_eval(
            "3 + 4/(2*3*4) - 4/(4*5*6) + 4/(6*7*8) - 4/(8*9*10) + 4/(10*11*12) - 4/(12*13*14)",
            "3.1408813408813407",
        );
    }
}
