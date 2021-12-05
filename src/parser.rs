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
        std::iter::from_fn(|| self.peek().map(|_| parser(self))).try_collect()
    }

    pub(crate) fn run(&mut self) -> Result<Vec<Stmt>> {
        self.many0(Self::decl)
    }
}

#[allow(clippy::enum_glob_use)]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::lexer::Lexer;

    fn assert_expr(src: &str, expected: &str) {
        let tokens = Lexer::new(src).analyze();
        let got = Parser::new(tokens)
            .expr()
            .map(|i| format!("{}", i))
            .unwrap();
        assert_eq!(expected, got);
    }

    fn assert_run(src: &str, expected: &[&str]) {
        let tokens = Lexer::new(src).analyze();
        let got = Parser::new(tokens)
            .run()
            .unwrap()
            .iter()
            .map(|i| format!("{}", i))
            .collect_vec();
        assert_eq!(expected, got);
    }

    #[test]
    fn basic() {
        assert_expr("1+2 / 3- 4 *5", "(- (+ 1 (/ 2 3)) (* 4 5))");
    }

    #[test]
    fn parens() {
        assert_expr(
            "-(-1+2 / 3- 4 *5+ (6/ 7))",
            "(- (group (+ (- (+ (- 1) (/ 2 3)) (* 4 5)) (group (/ 6 7)))))",
        );
    }

    #[test]
    #[should_panic(expected = "`)` expected")]
    fn paren_mismatch() {
        assert_expr("-(-1+2 / 3- 4 *5+ (6/ 7)", "");
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
        assert_expr("*1", "");
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
        assert_expr(
            "-(-1+2) >=3- 4 *5+ (6/ 7)",
            "(>= (- (group (+ (- 1) 2))) (+ (- 3 (* 4 5)) (group (/ 6 7))))",
        );
    }

    #[test]
    #[should_panic(expected = "found binary operator `>=`")]
    fn inequality_used_as_unary() {
        assert_expr(">= 1+2 == 3", "");
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
    fn assign() {
        assert_expr("a = b = c = 3;", "(assign! a (assign! b (assign! c 3)))");
    }

    #[test]
    fn print_stmt() {
        assert_run(
            "print -(-1+2) >=3;",
            &["(print (>= (- (group (+ (- 1) 2))) 3))"],
        );
    }

    #[test]
    fn var() {
        assert_run("foo;", &["foo"]);
    }

    #[test]
    fn print_stmt_var() {
        assert_run("print foo;", &["(print foo)"]);
    }

    #[test]
    fn var_decl() {
        assert_run(
            "var foo=-(-1+2) >=3;",
            &["(var foo (>= (- (group (+ (- 1) 2))) 3))"],
        );
    }

    #[test]
    fn var_decl_empty() {
        assert_run("var foo;", &["(var foo)"]);
    }

    #[test]
    fn block_stmt() {
        assert_run(
            "var foo; { var bar = 1; print bar; } var baz;",
            &["(var foo)", "(begin (var bar 1) (print bar))", "(var baz)"],
        );
    }

    #[test]
    #[should_panic(expected = "expected `}` to finish the block")]
    fn block_stmt_no_rightbrace() {
        assert_run("var foo; { var bar = 1; print bar; var baz;", &[]);
    }
}
