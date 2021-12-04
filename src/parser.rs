use std::fmt::Display;

use anyhow::{Context, Result};
use itertools::Itertools;
use tap::TapFallible;

use crate::run::error_report;
#[allow(clippy::enum_glob_use)]
use crate::{
    bail,
    lexer::{
        Token,
        TokenType::{self, *},
    },
};

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
            Variable(var) => write!(f, "(var '{})", var),
        }
    }
}

pub(crate) enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Token,
        /// # Note
        /// Tokenhis value **must** be of variant `Expr::Variable`.
        superclass: Expr,
        /// # Note
        /// Tokenhis `Vec` **must** contain instances of
        /// `Stmt::Function`.
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
        else_stmt: Box<Stmt>,
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

pub(crate) struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: impl Iterator<Item = Token>) -> Self {
        Parser {
            tokens: tokens.collect(),
            idx: 0,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.idx).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let res = self.peek()?;
        self.idx += 1;
        Some(res)
    }

    fn previous(&self) -> Option<Token> {
        self.tokens.get(self.idx - 1).cloned()
    }

    fn check(&mut self, ty: TokenType) -> Option<Token> {
        self.peek().filter(|t| t.ty == ty)
    }

    fn test(&mut self, tys: &[TokenType]) -> Option<Token> {
        tys.iter().find_map(|&ty| {
            let curr = self.peek();
            self.check(ty).and_then(|_| {
                self.advance();
                curr
            })
        })
    }

    /// Consumes a specific token or throws an error.
    fn consume(&mut self, tys: &[TokenType], ctx: &str, msg: impl Display) -> Result<Token> {
        self.test(tys)
            .with_context(|| error_report(self.previous().unwrap().pos, ctx, msg))
    }

    fn sync(&mut self) {
        let stmt_begin = [Class, Fun, Var, For, If, While, Print, Return];
        loop {
            self.advance();
            let curr = self.peek();
            let synced = curr.is_none() // Reached the end of the source.
                || self.previous().unwrap().ty == Semicolon // Passed the end of the statement.
                || stmt_begin.contains(&curr.unwrap().ty); // Reached the beginning of another statement.
            if synced {
                break;
            }
        }
    }

    pub(crate) fn many0<T>(
        &mut self,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        std::iter::repeat_with(|| self.peek().map(|_| parser(self)))
            .map_while(|i| i)
            .try_collect()
    }

    pub(crate) fn run(&mut self) -> Result<Vec<Stmt>> {
        self.many0(Self::stmt)
    }
}

// ** Recursive Descent for Expr **
impl Parser {
    pub(crate) fn expr(&mut self) -> Result<Expr> {
        self.equality_expr()
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
            lp = LeftParen => {
                let inner = self.expr()?;
                if self.test(&[RightParen]).is_none() {
                    self.sync();
                    bail!(lp.pos, "while parsing a parenthesized Group", "`)` expected");
                }
                Grouping(Box::new(inner))
            },
            ident = Identifier => Variable(ident)
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
        match self.test(&[Print]) {
            Some(t) if t.ty == Print => self.print_stmt(),
            None => self.expression_stmt(),
            _ => unreachable!(),
        }
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

    pub(crate) fn print_stmt(&mut self) -> Result<Stmt> {
        let rhs = self.expr()?;
        self.test(&[Semicolon]).with_context(|| {
            error_report(
                self.previous().unwrap().pos,
                "while parsing a Print statement",
                "expected `;` after a value",
            )
        })?;
        Ok(Stmt::Print(rhs))
    }
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn basic() {
        let tokens = Lexer::new("1+2 / 3- 4 *5").analyze();
        let got = Parser::new(tokens).expr().unwrap();
        let expected = "(- (+ 1 (/ 2 3)) (* 4 5))";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    fn parens() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7))").analyze();
        let got = Parser::new(tokens).expr().unwrap();
        let expected = "(- (group (+ (- (+ (- 1) (/ 2 3)) (* 4 5)) (group (/ 6 7)))))";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    #[should_panic(expected = "`)` expected")]
    fn paren_mismatch() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7)").analyze();
        let _got = Parser::new(tokens).expr().unwrap();
    }

    #[test]
    fn paren_mismatch_sync() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7); 8 +9").analyze();
        let mut parser = Parser::new(tokens);
        assert!(dbg!(parser.expr()).is_err());
        let got = parser.expr().unwrap();
        let expected = "(+ 8 9)";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    #[should_panic(expected = "found binary operator `*`")]
    fn mul_used_as_unary() {
        let tokens = Lexer::new("*1").analyze();
        let _got = Parser::new(tokens).expr().unwrap();
    }

    #[test]
    fn mul_used_as_unary_sync() {
        let tokens = Lexer::new("* 1-2 == 3").analyze();
        let mut parser = Parser::new(tokens);
        // >= (1+2)
        assert!(dbg!(parser.expr()).is_err());
        // +2 == 3
        let got = parser.expr().unwrap();
        let expected = "(== (- 2) 3)";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    fn inequality() {
        let tokens = Lexer::new("-(-1+2) >=3- 4 *5+ (6/ 7)").analyze();
        let got = Parser::new(tokens).expr().unwrap();
        let expected = "(>= (- (group (+ (- 1) 2))) (+ (- 3 (* 4 5)) (group (/ 6 7))))";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    #[should_panic(expected = "found binary operator `>=`")]
    fn inequality_used_as_unary() {
        let tokens = Lexer::new(">= 1+2 == 3").analyze();
        let _got = Parser::new(tokens).expr().unwrap();
    }

    #[test]
    #[should_panic(expected = "found binary operator `==`")]
    fn inequality_used_as_unary_sync() {
        let tokens = Lexer::new(">= 1+2 == 3").analyze();
        let mut parser = Parser::new(tokens);
        // >= (1+2)
        assert!(dbg!(parser.expr()).is_err());
        // == 3
        dbg!(parser.expr()).unwrap();
    }
}
