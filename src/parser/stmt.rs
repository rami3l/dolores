use std::fmt::Display;

use anyhow::{Context, Result};
use itertools::Itertools;
use tap::TapFallible;

use super::{Expr, Lit, Parser};
use crate::run::report;
#[allow(clippy::enum_glob_use)]
use crate::{
    bail,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

#[derive(Debug, Clone)]
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
    Fun {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    Jump(Token),
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
            Stmt::Fun { name, params, body } => {
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
            Stmt::Jump(t) => write!(f, "({})", t.lexeme),
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
        match self.test(&[Fun, Var]) {
            Some(t) if t.ty == Fun => self.fun_decl(),
            Some(t) if t.ty == Var => self.var_decl(),
            None => self.stmt(),
            _ => unreachable!(),
        }
        .tap_err(|_| self.sync())
    }

    fn fun_decl(&mut self) -> Result<Stmt> {
        let ctx = "while parsing a Fun declaration";
        let name = self.consume(&[Identifier], ctx, "expected function name")?;
        self.consume(&[LeftParen], ctx, "expected `(` after function name")?;
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
        Ok(Stmt::Fun { name, params, body })
    }

    fn var_decl(&mut self) -> Result<Stmt> {
        let ctx = "while parsing a Var declaration";
        let name = self.consume(&[Identifier], ctx, "expected variable name")?;
        let init = self.test(&[Equal]).map(|_| self.expr()).transpose()?;
        self.consume(&[Semicolon], ctx, "expected `;` after a value")?;
        Ok(Stmt::Var { name, init })
    }

    pub(crate) fn stmt(&mut self) -> Result<Stmt> {
        match self.test(&[Break, Continue, If, While, For, Print, LeftBrace]) {
            Some(t) if [Break, Continue].contains(&t.ty) => self.jump_stmt(),
            Some(t) if t.ty == If => self.if_stmt(),
            Some(t) if t.ty == While => self.while_stmt(),
            Some(t) if t.ty == For => self.for_stmt(),
            Some(t) if t.ty == Print => self.print_stmt(),
            Some(t) if t.ty == LeftBrace => self.block_stmt(),
            None => self.expression_stmt(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn jump_stmt(&mut self) -> Result<Stmt> {
        let kw = self.previous().unwrap();
        self.consume(
            &[Semicolon],
            "while parsing an Jump statement",
            "expected `;` at the end",
        )?;
        Ok(Stmt::Jump(kw))
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        let ctx = "while parsing an If statement";
        let cond = self.parens(Self::expr, "the Predicate")?;
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

    fn while_stmt(&mut self) -> Result<Stmt> {
        let ctx = "while parsing a While statement";
        let cond = self.parens(Self::expr, "the Predicate")?;
        let body = Box::new(self.stmt().with_context(|| {
            report(
                self.previous().unwrap().pos,
                ctx,
                "nothing in the loop body",
            )
        })?);
        Ok(Stmt::While { cond, body })
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        let ctx = "while parsing a For statement";
        let (init, cond, incr) = self.parens(
            |this| {
                let init = match this.test(&[Semicolon, Var]) {
                    Some(t) if t.ty == Semicolon => None,
                    Some(t) if t.ty == Var => Some(this.var_decl()?),
                    None => Some(this.expression_stmt()?),
                    _ => unreachable!(),
                };
                let cond = if this.test(&[Semicolon]).is_none() {
                    let cond = this.expr()?;
                    this.consume(&[Semicolon], ctx, "expected `;` after the Condition Clause")?;
                    Some(cond)
                } else {
                    None
                };
                let incr = if this.peek().map(|t| t.ty) == Some(RightParen) {
                    None
                } else {
                    Some(this.expr()?)
                };
                Ok((init, cond, incr))
            },
            "the Predicate Clauses",
        )?;
        let body = Box::new(self.stmt().with_context(|| {
            report(
                self.previous().unwrap().pos,
                ctx,
                "nothing in the loop body",
            )
        })?);

        // Desugaring begins...
        // for (init; cond; incr) { body; } => { init; while (cond) { body; incr; } }
        let body = Box::new(Stmt::Block(if let Some(incr) = incr {
            vec![*body, Stmt::Expression(incr)]
        } else {
            vec![*body]
        }));

        let cond = cond.unwrap_or(Expr::Literal(Lit::Bool(true)));
        let while_loop = Stmt::While { cond, body };

        Ok(Stmt::Block(if let Some(init) = init {
            vec![init, while_loop]
        } else {
            vec![while_loop]
        }))
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
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

    fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.consume(
            &[Semicolon],
            "while parsing an Expression statement",
            "expected `;` after a value",
        )?;
        Ok(Stmt::Expression(expr))
    }

    fn block_stmt(&mut self) -> Result<Stmt> {
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
