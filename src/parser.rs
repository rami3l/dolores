pub(crate) mod expr;
pub(crate) mod stmt;

use std::fmt::Display;

use anyhow::{Context, Result};
use itertools::Itertools;

pub(crate) use self::{
    expr::{Expr, Lit},
    stmt::Stmt,
};
#[allow(clippy::enum_glob_use)]
use crate::{
    lexer::{
        Token,
        TokenType::{self, *},
    },
    run::report,
};

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
            .with_context(|| report(self.previous().unwrap().pos, ctx, msg))
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
        self.many0(Self::decl)
    }
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use itertools::izip;
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

    #[test]
    fn print_stmt() {
        let tokens = Lexer::new("print -(-1+2) >=3;").analyze();
        let got = Parser::new(tokens).run().unwrap();
        let expected = ["(print (>= (- (group (+ (- 1) 2))) 3))"];
        izip!(got, expected).for_each(|(got, expected)| assert_eq!(expected, format!("{}", got)));
    }

    #[test]
    fn var_decl() {
        let tokens = Lexer::new("var foo=-(-1+2) >=3;").analyze();
        let got = Parser::new(tokens).run().unwrap();
        let expected = ["(var foo (>= (- (group (+ (- 1) 2))) 3)))"];
        izip!(got, expected).for_each(|(got, expected)| assert_eq!(expected, format!("{}", got)));
    }

    #[test]
    fn var_decl_empty() {
        let tokens = Lexer::new("var foo;").analyze();
        let got = Parser::new(tokens).run().unwrap();
        let expected = ["(var foo)"];
        izip!(got, expected).for_each(|(got, expected)| assert_eq!(expected, format!("{}", got)));
    }
}
