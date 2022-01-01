mod tests;

use std::fmt::Display;

use anyhow::Result;
use itertools::Itertools;

use super::{Parser, Stmt};
use crate::util::disp_slice;
#[allow(clippy::enum_glob_use)]
use crate::{
    bail,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

const MAX_FUN_ARG_COUNT: usize = 255;

#[derive(Debug, Clone)]
pub(crate) enum Expr {
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
        args: Vec<Expr>,
        /// The trailing RightParen of the function call.
        /// Its position is memorized for error reports.
        end: Token,
    },
    Get {
        obj: Box<Expr>,
        name: Token,
    },
    Grouping(Box<Expr>),
    Lambda {
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
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
            Call { callee, args, .. } => {
                if args.is_empty() {
                    write!(f, "({})", callee)
                } else {
                    write!(f, "({} {})", callee, args.iter().join(" "))
                }
            }
            Get { obj, name } => write!(f, "(. {} {})", obj, name),
            Grouping(expr) => write!(f, "{}", expr),
            Lambda { params, body } => {
                let (params, body) = (disp_slice(params, false), disp_slice(body, true));
                write!(f, "(lambda ({}) {})", params, body)
            }
            Literal(lit) => write!(f, "{}", lit),
            Set { obj, name, to } => write!(f, "(.set! {} {} {})", obj, name, to),
            Super { method, .. } => write!(f, "(. (super) {})", method),
            This(_) => write!(f, "(this)"),
            Unary { op, rhs } => write!(f, "({} {})", op, rhs),
            Variable(var) => write!(f, "{}", var),
        }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Self::Literal(Lit::Nil)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Lit {
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
            let mut rhs = || self.assignment_expr();
            match lhs {
                Expr::Variable(name) => {
                    let val = Box::new(rhs()?);
                    return Ok(Expr::Assign { name, val });
                }
                Expr::Get { obj, name } => {
                    let to = Box::new(rhs()?);
                    return Ok(Expr::Set { obj, name, to });
                }
                _ => bail!(
                    self.previous().unwrap().pos,
                    "while parsing a Assignment expression",
                    "can only assign to a variable",
                ),
            }
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
        self.call_expr()
    }

    fn call_expr(&mut self) -> Result<Expr> {
        let mut res = self.primary_expr()?;
        loop {
            if self.test(&[LeftParen]).is_some() {
                let args = self.call_params(Self::expr)?;
                res = Expr::Call {
                    callee: Box::new(res),
                    args,
                    end: self.previous().unwrap(),
                };
            } else if self.test(&[Dot]).is_some() {
                let name = self.consume(
                    &[Identifier],
                    "while parsing a Get expression",
                    "expect property name after `.`",
                )?;
                res = Expr::Get {
                    obj: Box::new(res),
                    name,
                }
            } else {
                break;
            }
        }
        Ok(res)
    }

    pub(crate) fn call_params<F, O>(&mut self, arg_parser: F) -> Result<Vec<O>>
    where
        F: Fn(&mut Self) -> Result<O>,
    {
        let ctx = "while parsing function parameter list";
        let mut args = vec![];
        if self.peek().filter(|t| t.ty == RightParen).is_none() {
            loop {
                args.push(arg_parser(self)?);
                if self.test(&[Comma]).is_none() {
                    break;
                }
            }
        }
        if self.test(&[RightParen]).is_none() {
            self.sync();
            bail!(
                self.previous().unwrap().pos,
                ctx,
                "expected `)` to end the parameter list"
            );
        }
        if args.len() > MAX_FUN_ARG_COUNT {
            bail!(
                self.previous().unwrap().pos,
                ctx,
                format!("cannot have more than {} parameters", MAX_FUN_ARG_COUNT)
            )
        }
        Ok(args)
    }

    fn primary_expr(&mut self) -> Result<Expr> {
        macro_rules! bail_if_matches {
            ( $( $pat:pat = $ty:expr => $res:expr ),+ $(,)? ) => {{
                $( if let Some($pat) = self.test(&[$ty]) {
                    return Ok($res);
                } )+
            }};
        }

        bail_if_matches! {
            _ = False => Expr::Literal(Lit::Bool(false)),
            _ = True => Expr::Literal(Lit::Bool(true)),
            _ = Nil => Expr::Literal(Lit::Nil),
            s = Str => Expr::Literal(Lit::Str({
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
                Expr::Literal(Lit::Number(val.unwrap()))
            },
            t = This => Expr::This(t),
            i = Identifier => Expr::Variable(i),
            _ = Fun => {
                let ctx = "while parsing a Lambda expression";
                self.consume(&[LeftParen], ctx, "expected `(` to begin the parameter list")?;
                let params =
                    self.call_params(|this| this.consume(&[Identifier], ctx, "expected parameter name"))?;
                self.consume(
                    &[LeftBrace],
                    ctx,
                    "expected `{` after function parameter list",
                )?;
                let body = if let Stmt::Block(stmts) = self.block_stmt()? {
                    stmts
                } else {
                    unreachable!()
                };
                Expr::Lambda { params, body }
            },
            lp = LeftParen => {
                let inner = self.expr()?;
                if self.test(&[RightParen]).is_none() {
                    self.sync();
                    bail!(lp.pos, "while parsing a parenthesized Group", "`)` expected");
                }
                Expr::Grouping(Box::new(inner))
            },
            kw = Super => {
                let ctx = "while parsing a superclass method";
                self.consume(&[Dot], ctx, "expected `.` after `super`")?;
                let method = self.consume(&[Identifier], ctx, "expected superclass method name after `.`")?;
                Expr::Super { kw, method }
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
