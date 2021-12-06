use std::fmt::Display;

use anyhow::{Context, Result};
use itertools::Itertools;
use tap::TapFallible;

use super::{Expr, Parser};
use crate::run::report;
#[allow(clippy::enum_glob_use)]
use crate::{
    bail,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Token,
        /// # Note
        /// This `Option` **must** contain an instance of `Expr::Variable`.
        superclass: Option<Expr>,
        /// # Note
        /// This `Vec` **must** contain instances of `Stmt::Function`.
        methods: Vec<Stmt>,
    },
    Expression(Expr),
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        kw: Token,
        val: Expr,
    },
    Var {
        name: Token,
        init: Option<Expr>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn disp_slice(xs: &[impl Display]) -> String {
            xs.iter().map(|x| format!("{}", x)).join(" ")
        }

        match self {
            Stmt::Block(stmts) => write!(f, "(begin {})", disp_slice(stmts)),
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let superclass = superclass
                    .as_ref()
                    .map_or_else(String::new, |sup| format!(" (extends {})", sup));
                let methods = disp_slice(methods);
                write!(f, "(class {}{} ({}))", name, superclass, methods)
            }
            Stmt::Expression(expr) => write!(f, "{}", expr),
            Stmt::Function { name, params, body } => {
                let (params, body) = (disp_slice(params), disp_slice(body));
                write!(f, "(fun {} ({}) {}", name, params, body)
            }
            Stmt::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                let else_stmt = else_stmt
                    .as_ref()
                    .map_or_else(String::new, |i| format!(" {}", i));
                write!(f, "(if {} {}{})", cond, then_stmt, else_stmt)
            }
            Stmt::Print(expr) => write!(f, "(print {})", expr),
            Stmt::Return { kw, val } => write!(f, "({} {})", kw, val),
            Stmt::Var { name, init } => {
                let init = init
                    .as_ref()
                    .map_or_else(String::new, |i| format!(" {}", i));
                write!(f, "(var {}{})", name, init)
            }
            Stmt::While { cond, body } => write!(f, "(while {} {})", cond, body),
        }
    }
}

// ** Recursive Descent for Stmt and Decl **
impl Parser {
    pub(crate) fn decl(&mut self) -> Result<Stmt> {
        match self.test(&[Var]) {
            Some(t) if t.ty == Var => self.var_decl(),
            None => self.stmt(),
            _ => unreachable!(),
        }
        .tap_err(|_| self.sync())
    }

    pub(crate) fn var_decl(&mut self) -> Result<Stmt> {
        let ctx = "while parsing a Var declaration";
        let name = self.consume(&[Identifier], ctx, "expected variable name")?;
        let init = self.test(&[Equal]).map(|_| self.expr()).transpose()?;
        self.consume(&[Semicolon], ctx, "expected `;` after a value")?;
        Ok(Stmt::Var { name, init })
    }

    pub(crate) fn stmt(&mut self) -> Result<Stmt> {
        match self.test(&[Print, LeftBrace, If]) {
            Some(t) if t.ty == If => self.if_stmt(),
            Some(t) if t.ty == Print => self.print_stmt(),
            Some(t) if t.ty == LeftBrace => self.block_stmt(),
            None => self.expression_stmt(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn if_stmt(&mut self) -> Result<Stmt> {
        let ctx = "while parsing an If statement";

        self.consume(&[LeftParen], ctx, "expected `(` before the Predicate")?;
        let cond = self.expr()?;
        self.consume(&[RightParen], ctx, "expected `)` after the Predicate")?;

        let then_stmt = Box::new(self.stmt().with_context(|| {
            report(
                self.previous().unwrap().pos,
                ctx,
                "nothing in the Then branch",
            )
        })?);

        let else_stmt = self
            .test(&[Else])
            .map(|_| anyhow::Ok(Box::new(self.stmt()?)))
            .transpose()?;

        Ok(Stmt::If {
            cond,
            then_stmt,
            else_stmt,
        })
    }

    pub(crate) fn print_stmt(&mut self) -> Result<Stmt> {
        let rhs = self.expr().with_context(|| {
            report(
                self.previous().unwrap().pos,
                "while parsing a Print statement",
                "nothing to print",
            )
        })?;
        self.consume(
            &[Semicolon],
            "while parsing an Print statement",
            "expected `;` after a value",
        )?;
        Ok(Stmt::Print(rhs))
    }

    pub(crate) fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.consume(
            &[Semicolon],
            "while parsing an Expression statement",
            "expected `;` after a value",
        )?;
        Ok(Stmt::Expression(expr))
    }

    pub(crate) fn block_stmt(&mut self) -> Result<Stmt> {
        // When parsing statements here, we need an 1-token lookahead.
        let stmts = std::iter::from_fn(|| {
            self.peek()
                .filter(|t| t.ty != RightBrace)
                .map(|_| self.decl())
        })
        .try_collect()?;
        self.consume(
            &[RightBrace],
            "while parsing a Block statement",
            "expected `}` to finish the block",
        )?;
        Ok(Stmt::Block(stmts))
    }
}
