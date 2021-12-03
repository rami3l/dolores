use std::fmt::Display;

use anyhow::{bail, Result};

#[allow(clippy::enum_glob_use)]
use crate::{
    lexer::{
        Token,
        TokenType::{self, *},
    },
    run::bail,
};

#[derive(Debug, Clone)]
enum Expr {
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

enum Stmt {
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
        init: Expr,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone)]
enum Lit {
    Bool(bool),
    Number(f64),
    Str(String),
    Nil,
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

struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    fn new(tokens: impl Iterator<Item = Token>) -> Self {
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

    // ** Recursive Descent **

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
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

    fn equality(&mut self) -> Result<Expr> {
        self.recursive_descent_binary(&[BangEqual, EqualEqual], Self::comparison)
    }

    fn comparison(&mut self) -> Result<Expr> {
        self.recursive_descent_binary(&[Greater, GreaterEqual, Less, LessEqual], Self::term)
    }

    fn term(&mut self) -> Result<Expr> {
        self.recursive_descent_binary(&[Plus, Minus], Self::factor)
    }

    fn factor(&mut self) -> Result<Expr> {
        self.recursive_descent_binary(&[Slash, Star], Self::unary)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(op) = self.test(&[Plus, Minus]) {
            let rhs = Box::new(self.unary()?);
            return Ok(Expr::Unary { op, rhs });
        }
        if let Some(op) = self.test(&[
            // TODO: This is false since the precedence is */, +-, <>, ==
            Slash,
            Star,
            Greater,
            GreaterEqual,
            Less,
            LessEqual,
            BangEqual,
            EqualEqual,
        ]) {
            // Consume the ill-formed RHS.
            let _rhs = self.unary()?;
            bail(
                op.pos,
                "while parsing an Unary expression",
                format!("found binary operator `{}`", op.lexeme),
            )?;
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        use Expr::{Grouping, Literal};

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
            s = Str => Literal(Lit::Str(s.lexeme)),
            n = Number => {
                let lexeme = &n.lexeme;
                let val = lexeme.parse();
                if let Err(e) = &val {
                    bail(n.pos, &format!("while parsing Number `{}`", lexeme), e)?;
                }
                Literal(Lit::Number(val.unwrap()))
            },
            lp = LeftParen => {
                let inner = self.expression()?;
                if self.test(&[RightParen]).is_none() {
                    self.sync();
                    bail(lp.pos, "while parsing a parenthesized Group", "`)` expected")?;
                }
                Grouping(Box::new(inner))
            },
        };

        if let Some(t) = self.peek() {
            bail(
                t.pos,
                &format!("while parsing `{}`", &t.lexeme),
                "unexpected token",
            )?;
        }
        bail!("[L??:??] Error while parsing: token index out of range")
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
        let got = Parser::new(tokens).expression().unwrap();
        let expected = "(- (+ 1 (/ 2 3)) (* 4 5))";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    fn parens() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7))").analyze();
        let got = Parser::new(tokens).expression().unwrap();
        let expected = "(- (group (+ (- (+ (- 1) (/ 2 3)) (* 4 5)) (group (/ 6 7)))))";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    #[should_panic(expected = "`)` expected")]
    fn paren_mismatch() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7)").analyze();
        let _got = Parser::new(tokens).expression().unwrap();
    }

    #[test]
    fn paren_mismatch_sync() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7); 8 +9").analyze();
        let mut parser = Parser::new(tokens);
        assert!(dbg!(parser.expression()).is_err());
        let got = parser.expression().unwrap();
        let expected = "(+ 8 9)";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    #[should_panic(expected = "found binary operator `*`")]
    fn mul_used_as_unary() {
        let tokens = Lexer::new("*1").analyze();
        let _got = Parser::new(tokens).expression().unwrap();
    }

    #[test]
    fn mul_used_as_unary_sync() {
        let tokens = Lexer::new("* 1+2 == 3").analyze();
        let mut parser = Parser::new(tokens);
        // >= (1+2)
        assert!(dbg!(parser.expression()).is_err());
        // +2 == 3
        let got = parser.expression().unwrap();
        let expected = "(== (+ 2) 3)";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    fn inequality() {
        let tokens = Lexer::new("-(-1+2) >=3- 4 *5+ (6/ 7)").analyze();
        let got = Parser::new(tokens).expression().unwrap();
        let expected = "(>= (- (group (+ (- 1) 2))) (+ (- 3 (* 4 5)) (group (/ 6 7))))";
        assert_eq!(expected, format!("{}", got));
    }

    #[test]
    #[should_panic(expected = "found binary operator `>=`")]
    fn inequality_used_as_unary() {
        let tokens = Lexer::new("1 + >= 2-3 == 4").analyze();
        let _got = Parser::new(tokens).expression().unwrap();
    }

    #[test]
    fn inequality_used_as_unary_sync() {
        let tokens = Lexer::new(">= 1+2 == 3").analyze();
        let mut parser = Parser::new(tokens);
        // >= (1+2)
        assert!(dbg!(parser.expression()).is_err());
        // == 3
        assert!(dbg!(parser.expression()).is_err());
    }
}
