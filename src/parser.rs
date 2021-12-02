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
        let stmt_beginning = [Class, Fun, Var, For, If, While, Print, Return];
        loop {
            self.advance();
            let curr = self.peek();
            let synced = curr.is_none() // Reached the end of the source.
                || self.previous().unwrap().ty == Semicolon // Passed the end of the statement.
                || stmt_beginning.contains(&curr.unwrap().ty); // Reached the beginning of another statement.
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
        while self.test(tys).is_some() {
            let lhs = Box::new(res);
            let op = self.previous().unwrap();
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
        if self.test(&[Plus, Minus]).is_some() {
            let op = self.previous().unwrap();
            let rhs = Box::new(self.unary()?);
            return Ok(Expr::Unary { op, rhs });
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
                    bail(lp.pos, "while parsing parenthesized Group", "`)` expected")?;
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
    use anyhow::Result;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_basic_parsing() -> Result<()> {
        let tokens = Lexer::new("1+2 / 3- 4 *5").analyze();
        let got = Parser::new(tokens).expression()?;
        let expected = indoc!(
            r#"Binary { lhs: Binary { lhs: Literal(Number(1.0)), op: Token { ty: Plus, lexeme: "+", pos: (1, 2) }, rhs: Binary { lhs: Literal(Number(2.0)), op: Token { ty: Slash, lexeme: "/", pos: (1, 5) }, rhs: Literal(Number(3.0)) } }, op: Token { ty: Minus, lexeme: "-", pos: (1, 8) }, rhs: Binary { lhs: Literal(Number(4.0)), op: Token { ty: Star, lexeme: "*", pos: (1, 12) }, rhs: Literal(Number(5.0)) } }"#
        );
        assert_eq!(expected, format!("{:?}", got));
        Ok(())
    }

    #[test]
    fn test_basic_parsing_with_parens() -> Result<()> {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7))").analyze();
        let got = Parser::new(tokens).expression()?;
        let expected = indoc!(
            r#"Unary { op: Token { ty: Minus, lexeme: "-", pos: (1, 1) }, rhs: Grouping(Binary { lhs: Binary { lhs: Binary { lhs: Unary { op: Token { ty: Minus, lexeme: "-", pos: (1, 3) }, rhs: Literal(Number(1.0)) }, op: Token { ty: Plus, lexeme: "+", pos: (1, 5) }, rhs: Binary { lhs: Literal(Number(2.0)), op: Token { ty: Slash, lexeme: "/", pos: (1, 8) }, rhs: Literal(Number(3.0)) } }, op: Token { ty: Minus, lexeme: "-", pos: (1, 11) }, rhs: Binary { lhs: Literal(Number(4.0)), op: Token { ty: Star, lexeme: "*", pos: (1, 15) }, rhs: Literal(Number(5.0)) } }, op: Token { ty: Plus, lexeme: "+", pos: (1, 17) }, rhs: Grouping(Binary { lhs: Literal(Number(6.0)), op: Token { ty: Slash, lexeme: "/", pos: (1, 21) }, rhs: Literal(Number(7.0)) }) }) }"#
        );
        assert_eq!(expected, format!("{:?}", got));
        Ok(())
    }

    #[test]
    #[should_panic(expected = "`)` expected")]
    fn test_basic_parsing_with_mismatch() {
        let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7)").analyze();
        let _got = Parser::new(tokens).expression().unwrap();
    }
}
