#![cfg(test)]
#![allow(clippy::enum_glob_use)]

use anyhow::Result;
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

#[test]
fn while_stmt() -> Result<()> {
    assert_eval(&[
        ("var i = 1; var product = 1;", ""),
        ("while (i <= 5) { product = product * i; i = i + 1; }", ""),
        ("product", "120"),
    ])
}

#[test]
fn while_stmt_jump() -> Result<()> {
    assert_eval(&[
        ("var i = 1; var product = 1;", ""),
        (
            indoc! {"
                while (true) {
                    if (i == 3 or i == 5) {
                        i = i + 1;
                        continue;
                    }
                    product = product * i;
                    i = i + 1;
                    if (i > 6) {
                        break;
                    }
                }
            "},
            "",
        ),
        ("product", "48"),
    ])
}

#[test]
fn for_stmt() -> Result<()> {
    assert_eval(&[
        ("var product = 1;", ""),
        (
            "for (var i = 1; i <= 5; i = i + 1) { product = product * i; }",
            "",
        ),
        ("product", "120"),
    ])
}

#[test]
fn for_stmt_init_expr() -> Result<()> {
    assert_eval(&[
        ("var i; var product;", ""),
        (
            "for (i = product = 1; i <= 5; i = i + 1) { product = product * i; }",
            "",
        ),
        ("product", "120"),
    ])
}

#[test]
fn for_stmt_jump() -> Result<()> {
    assert_eval(&[
        ("var i = 1; var product = 1;", ""),
        (
            "for (;;) { product = product * i; i = i + 1; if (i > 5) break; }",
            "",
        ),
        ("product", "120"),
    ])
}

#[test]
#[should_panic(expected = "found Break out of loop context")]
fn bare_jump_break() {
    assert_eval(&[("break;", "")]).unwrap();
}

#[test]
#[should_panic(expected = "found Continue out of loop context")]
fn bare_jump_continue() {
    assert_eval(&[("continue;", "")]).unwrap();
}

#[test]
fn fun_closure() -> Result<()> {
    assert_eval(&[
        ("var i = 1; fun f(j, k) { return (i + j) * k; }", ""),
        ("f(2, 3)", "9"),
    ])
}

#[test]
#[should_panic(expected = "unexpected number of parameters (expected 2, got 1)")]
fn fun_arity() {
    assert_eval(&[
        ("var i = 1; fun f(j, k) { return (i + j) * k; }", ""),
        ("f(2)", ""),
    ])
    .unwrap();
}

#[test]
fn fun_currying() -> Result<()> {
    assert_eval(&[
        (
            indoc! {"
                var i = 1;
                fun f(j) { 
                    fun g(k) { return (i + j) * k; }
                    return g;
                }
            "},
            "",
        ),
        ("f(2)(3)", "9"),
    ])
}

#[test]
fn fun_lambda() -> Result<()> {
    assert_eval(&[
        (
            indoc! {"
                var i = 1;
                var f = fun (j) { return fun (k) { return (i + j) * k; }; };
            "},
            "",
        ),
        ("f(2)(3)", "9"),
    ])
}

#[test]
fn fun_lambda_param() -> Result<()> {
    assert_eval(&[
        ("fun thrice(f, x) { return f(x) * 3; }", ""),
        ("thrice(fun (x) { return x + 2; }, 1)", "9"),
    ])
}

#[test]
fn fun_counter() -> Result<()> {
    assert_eval(&[
        (
            indoc! {"
                fun make_counter() {
                    var i = 0;
                    fun count() { i = i + 1; return i; }
                    return count;
                }
                var counter = make_counter();
            "},
            "",
        ),
        ("counter()", "1"),
        ("counter()", "2"),
    ])
}

#[test]
fn fun_var_shadow() -> Result<()> {
    assert_eval(&[
        (
            indoc! {r#"
                var a = "global";
                fun scope(p) {
                    var p = "local";
                    return p;
                }
                var p = scope(a);
            "#},
            "",
        ),
        ("a", r#""global""#),
        ("p", r#""local""#),
    ])
}

#[test]
fn fun_env_trap() -> Result<()> {
    assert_eval(&[
        (
            indoc! {r#"
                var a = "global";
                var a1; var a2;
                {
                    fun get_a() { return a; }
                    a1 = get_a();
                    var a = "block";
                    a2 = get_a();
                }
            "#},
            "",
        ),
        ("a1", r#""global""#),
        ("a2", r#""global""#),
    ])
}

#[test]
fn fun_man_or_boy() -> Result<()> {
    // src: https://rosettacode.org/wiki/Man_or_boy_test#Lox
    fn inner() -> Result<()> {
        assert_eval(&[
            (
                indoc! {r#"
                    fun A(k, xa, xb, xc, xd, xe) {
                        fun B() {
                            k = k - 1;
                            return A(k, B, xa, xb, xc, xd);
                        }
                        if (k <= 0) { return xd() + xe(); }
                        return B();
                    }
                    
                    fun I0()  { return  0; }
                    fun I1()  { return  1; }
                    fun I_1() { return -1; }
                "#},
                "",
            ),
            // ("A(4, I1, I_1, I_1, I1, I0)", "1"),
            ("A(10, I1, I_1, I_1, I1, I0)", "-67"),
        ])
    }
    // HACK: We use a new thread with 32 MiB of stack to avoid stack overflow...
    // src: https://stackoverflow.com/a/44042122
    let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024);
    let handler = builder.spawn(inner).unwrap();
    handler.join().unwrap()?;
    Ok(())
}
