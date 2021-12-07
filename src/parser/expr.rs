use std::fmt::Display;

use anyhow::Result;

use super::Parser;
#[allow(clippy::enum_glob_use)]
use crate::{
    bail,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        val: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        /// The trailing RightParen of the function call.
        /// Used
        paren: Token,
        args: Vec<Expr>,
    },
    Get {
        obj: Box<Expr>,
        name: Token,
    },
    Grouping(Box<Expr>),
    Literal(Lit),
    Logical {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Set {
        obj: Box<Expr>,
        name: Token,
        to: Box<Expr>,
    },
    Super {
        kw: Token,
        method: Token,
    },
    This(Token),
    Unary {
        op: Token,
        rhs: Box<Expr>,
    },
    Variable(Token),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use Expr::*;

        match self {
            Assign { name, val } => write!(f, "(assign! {} {})", name, val),
            Binary { lhs, op, rhs } | Logical { lhs, op, rhs } => {
                write!(f, "({} {} {})", op, lhs, rhs)
            }
            Call { callee, args, .. } => write!(f, "(call {} {:?})", callee, args),
            Get { obj, name } => write!(f, "(obj-get {} '{})", obj, name),
            Grouping(g) => write!(f, "(group {})", g),
            Literal(lit) => write!(f, "{}", lit),
            Set { obj, name, to } => write!(f, "(obj-set! {} '{} {})", obj, name, to),
            Super { kw, method } => write!(f, "({} '{})", kw, method),
            This(kw) => write!(f, "({})", kw),
            Unary { op, rhs } => write!(f, "({} {})", op, rhs),
            Variable(var) => write!(f, "{}", var),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Lit {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Bool(b) => write!(f, "{}", b),
            Lit::Number(n) => write!(f, "{}", n),
            Lit::Str(s) => write!(f, r#""{}""#, s),
            Lit::Nil => write!(f, "nil"),
        }
    }
}

// ** Recursive Descent for Expr **
impl Parser {
    pub(crate) fn expr(&mut self) -> Result<Expr> {
        self.assignment_expr()
    }

    fn assignment_expr(&mut self) -> Result<Expr> {
        let lhs = self.logic_or_expr()?;
        if self.test(&[Equal]).is_some() {
            // Assignment expression detected.
            if let Expr::Variable(name) = lhs {
                let val = Box::new(self.assignment_expr()?);
                return Ok(Expr::Assign { name, val });
            }
            bail!(
                self.previous().unwrap().pos,
                "while parsing a Assignment expression",
                "can only assign to a variable",
            )
        }
        Ok(lhs)
    }

    #[allow(clippy::similar_names)]
    fn logic_or_expr(&mut self) -> Result<Expr> {
        let mut res = self.logic_and_expr()?;
        while let Some(op) = self.test(&[Or]) {
            let lhs = Box::new(res);
            let rhs = Box::new(self.logic_and_expr()?);
            res = Expr::Logical { lhs, op, rhs }
        }
        Ok(res)
    }

    #[allow(clippy::similar_names)]
    fn logic_and_expr(&mut self) -> Result<Expr> {
        let mut res = self.equality_expr()?;
        while let Some(op) = self.test(&[And]) {
            let lhs = Box::new(res);
            let rhs = Box::new(self.equality_expr()?);
            res = Expr::Logical { lhs, op, rhs }
        }
        Ok(res)
    }

    #[allow(clippy::similar_names)]
    fn recursive_descent_binary<F>(&mut self, tys: &[TokenType], descend_parse: F) -> Result<Expr>
    where
        F: Fn(&mut Self) -> Result<Expr>,
    {
        let mut res = descend_parse(self)?;
        while let Some(op) = self.test(tys) {
            let lhs = Box::new(res);
            let rhs = Box::new(descend_parse(self)?);
            res = Expr::Binary { lhs, op, rhs }
        }
        Ok(res)
    }

    fn equality_expr(&mut self) -> Result<Expr> {
        self.recursive_descent_binary(&[BangEqual, EqualEqual], Self::comparison_expr)
    }

    fn comparison_expr(&mut self) -> Result<Expr> {
        if let Some(op) = self.test(&[BangEqual, EqualEqual]) {
            // Consume the ill-formed RHS.
            let _rhs = self.comparison_expr()?;
            bail!(
                op.pos,
                "while parsing an Comparison expression",
                "found binary operator `{}` with no LHS",
                op.lexeme,
            );
        }
        self.recursive_descent_binary(&[Greater, GreaterEqual, Less, LessEqual], Self::term_expr)
    }

    fn term_expr(&mut self) -> Result<Expr> {
        if let Some(op) = self.test(&[Greater, GreaterEqual, Less, LessEqual]) {
            // Consume the ill-formed RHS.
            let _rhs = self.term_expr()?;
            bail!(
                op.pos,
                "while parsing an Term expression",
                "found binary operator `{}` with no LHS",
                op.lexeme,
            );
        }
        self.recursive_descent_binary(&[Plus, Minus], Self::factor_expr)
    }

    fn factor_expr(&mut self) -> Result<Expr> {
        // `Minus` is special: no LHS is completely fine.
        if let Some(op) = self.test(&[Plus]) {
            // Consume the ill-formed RHS.
            let _rhs = self.factor_expr()?;
            bail!(
                op.pos,
                "while parsing an Factor expression",
                "found binary operator `{}` with no LHS",
                op.lexeme,
            );
        }
        self.recursive_descent_binary(&[Slash, Star], Self::unary_expr)
    }

    fn unary_expr(&mut self) -> Result<Expr> {
        if let Some(op) = self.test(&[Slash, Star]) {
            // Consume the ill-formed RHS.
            let _rhs = self.unary_expr()?;
            bail!(
                op.pos,
                "while parsing an Unary expression",
                "found binary operator `{}` with no LHS",
                op.lexeme,
            );
        }
        if let Some(op) = self.test(&[Bang, Minus]) {
            let rhs = Box::new(self.unary_expr()?);
            return Ok(Expr::Unary { op, rhs });
        }
        self.primary_expr()
    }

    fn primary_expr(&mut self) -> Result<Expr> {
        use Expr::{Grouping, Literal, Variable};

        macro_rules! bail_if_matches {
            ( $( $pat:pat = $ty:expr => $res:expr ),+ $(,)? ) => {{
                $( if let Some($pat) = self.test(&[$ty]) {
                    return Ok($res);
                } )+
            }};
        }

        bail_if_matches! {
            _ = False => Literal(Lit::Bool(false)),
            _ = True => Literal(Lit::Bool(true)),
            _ = Nil => Literal(Lit::Nil),
            s = Str => Literal(Lit::Str({
                s.lexeme
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap()
                    .into()
            })),
            n = Number => {
                let lexeme = &n.lexeme;
                let val = lexeme.parse();
                if let Err(e) = &val {
                    bail!(n.pos, &format!("while parsing Number `{}`", lexeme), e);
                }
                Literal(Lit::Number(val.unwrap()))
            },
            ident = Identifier => Variable(ident),
            lp = LeftParen => {
                let inner = self.expr()?;
                if self.test(&[RightParen]).is_none() {
                    self.sync();
                    bail!(lp.pos, "while parsing a parenthesized Group", "`)` expected");
                }
                Grouping(Box::new(inner))
            },
        };

        if let Some(t) = self.peek() {
            bail!(
                t.pos,
                &format!("while parsing `{}`", &t.lexeme),
                "unexpected token",
            );
        }
        bail!((0, 0), "while parsing", "token index out of range");
    }
}
