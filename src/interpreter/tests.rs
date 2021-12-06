#![cfg(test)]
#![allow(clippy::enum_glob_use)]

use indoc::indoc;
use itertools::Itertools;
use pretty_assertions::assert_eq;

use super::*;
use crate::{lexer::Lexer, parser::Parser};

fn run_expr(src: &str, env: &RcCell<Env>) -> Result<String> {
    let tokens = Lexer::new(src).analyze().collect_vec();
    if let Err(e) = Parser::new(tokens.clone())
        .run()
        .and_then(|stmts| stmts.into_iter().try_for_each(|stmt| stmt.eval(env)))
    {
        if let Ok(expr) = Parser::new(tokens).expr() {
            return Ok(format!("{}", expr.eval(env)?));
        }
        return Err(e);
    }
    Ok("".into())
}

fn assert_eval(pairs: &[(&str, &str)]) -> Result<()> {
    let env = &Env::default().shared();
    pairs.iter().try_for_each(|(src, expected)| {
        let got = run_expr(src, env)?;
        assert_eq!(expected, &got);
        Ok(())
    })
}

#[test]
fn calculator() -> Result<()> {
    assert_eval(&[
        ("2 +2", "4"),
        ("11.4 + 5.14 / 19198.10", "11.400267734827926"),
        ("-6 *(-4+ -3) == 6*4 + 2  *((((9))))", "true"),
        (
            indoc! {"
                4/1 - 4/3 + 4/5 - 4/7 + 4/9 - 4/11 
                    + 4/13 - 4/15 + 4/17 - 4/19 + 4/21 - 4/23
            "},
            "3.058402765927333",
        ),
        (
            indoc! {"
                3
                    + 4/(2*3*4)
                    - 4/(4*5*6)
                    + 4/(6*7*8)
                    - 4/(8*9*10)
                    + 4/(10*11*12)
                    - 4/(12*13*14)
            "},
            "3.1408813408813407",
        ),
    ])
}

#[test]
fn vars_and_blocks() -> Result<()> {
    assert_eval(&[
        ("var foo = 2;", ""),
        ("foo", "2"),
        ("foo + 3 == 1 + foo * foo", "true"),
        ("var bar;", ""),
        ("bar", "nil"),
        ("bar = foo = 2;", ""),
        ("foo", "2"),
        ("bar", "2"),
        (
            "{ foo = foo + 1; var bar; var foo = foo; foo = foo + 1; }",
            "",
        ),
        ("foo", "3"),
    ])
}

#[test]
fn if_else() -> Result<()> {
    assert_eval(&[
        ("var foo = 2;", ""),
        ("if (foo == 2) foo = foo + 1; else { foo = 42; }", ""),
        ("foo", "3"),
        ("if (foo == 2) { foo = foo + 1; } else foo = nil;", ""),
        ("foo", "nil"),
        ("if (!foo) foo = 1;", ""),
        ("foo", "1"),
        ("if (foo) foo = 2;", ""),
        ("foo", "2"),
    ])
}

#[test]
fn if_else_and_or() -> Result<()> {
    assert_eval(&[
        ("var foo = 2;", ""),
        (
            "if (foo != 2 and whatever) foo = foo + 42; else { foo = 3; }",
            "",
        ),
        ("foo", "3"),
        (
            "if (0 <= foo and foo <= 3) { foo = foo + 1; } else { foo = nil; }",
            "",
        ),
        ("foo", "4"),
        ("if (!!!(2 + 2 != 5) or !!!!!!!!foo) foo = 1;", ""),
        ("foo", "1"),
        ("if (true or whatever) foo = 2;", ""),
        ("foo", "2"),
    ])
}

#[test]
fn and_or() -> Result<()> {
    assert_eval(&[
        (r#""trick" or __TREAT__"#, r#""trick""#),
        ("996 or 007", "996"),
        (r#"nil or "hi""#, r#""hi""#),
        ("nil and what", "nil"),
        (r#"true and "then_what""#, r#""then_what""#),
        ("var B = 66;", ""),
        ("2*B or !2*B", "132"),
    ])
}
