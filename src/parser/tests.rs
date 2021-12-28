#![cfg(test)]
#![allow(clippy::enum_glob_use)]

use indoc::indoc;
use pretty_assertions::assert_eq;

use super::*;
use crate::lexer::Lexer;

// ** Expressions **

fn assert_expr(src: &str, expected: &str) {
    let tokens = Lexer::new(src).analyze();
    let got = Parser::new(tokens)
        .expr()
        .map(|i| format!("{}", i))
        .unwrap();
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
        "(- (+ (- (+ (- 1) (/ 2 3)) (* 4 5)) (/ 6 7)))",
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
        "(>= (- (+ (- 1) 2)) (+ (- 3 (* 4 5)) (/ 6 7)))",
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
    assert_expr("a = b = c = 3", "(assign! a (assign! b (assign! c 3)))");
}

#[test]
fn bool_logic() {
    assert_expr(
        "foo == nil or !!bar and a != (b = c = 3)",
        "(or (== foo nil) (and (! (! bar)) (!= a (assign! b (assign! c 3)))))",
    );
}

#[test]
fn lambda() {
    assert_expr("fun () { }", "(lambda () '())");
    assert_expr("fun () { } ()", "((lambda () '()) )");
    assert_expr(
        "fun (a, b, c, d) { print a * b - c / d; }",
        "(lambda (a b c d) (print (- (* a b) (/ c d))))",
    );
}

#[test]
fn get() {
    assert_expr(
        "egg.scramble(3).with(cheddar)",
        "((. ((. egg scramble) 3) with) cheddar)",
    );
}

// ** Statements & Declarations **

fn assert_stmts(src: &str, expected: &[&str]) {
    let tokens = Lexer::new(src).analyze();
    let got = Parser::new(tokens)
        .parse()
        .unwrap()
        .iter()
        .map(|i| format!("{}", i))
        .collect_vec();
    assert_eq!(expected, got);
}

#[test]
fn print_stmt() {
    assert_stmts("print -(-1+2) >=3;", &["(print (>= (- (+ (- 1) 2)) 3))"]);
}

#[test]
fn if_stmt() {
    assert_stmts(
        "var year; if (2 + 2 == 5) year = 1984; else year = 2021;",
        &[
            "(var year)",
            "(if (== (+ 2 2) 5) (assign! year 1984) (assign! year 2021))",
        ],
    );
}

#[test]
#[should_panic(expected = "nothing in the Then branch")]
fn if_stmt_no_then() {
    assert_stmts("var year; if (2 + 2 == 5)", &[]);
}

#[test]
fn if_stmt_no_else() {
    assert_stmts(
        "var year; if (2 + 2 == 5) { year = 1984; }",
        &[
            "(var year)",
            "(if (== (+ 2 2) 5) (begin (assign! year 1984)))",
        ],
    );
}

#[test]
fn if_stmt_why_not_kr_style() {
    assert_stmts(
        indoc! {"
                if (first)
                    if (second) whenTrue;
                    else whenFalse;
            "},
        &["(if first (if second whenTrue whenFalse))"],
    );
}

#[test]
fn while_stmt() {
    assert_stmts(
        "while (i <= 5) { product = product * i; i = i + 1; }",
        &["(while (<= i 5) (begin (assign! product (* product i)) (assign! i (+ i 1))))"],
    );
}

#[test]
fn for_stmt() {
    assert_stmts(
        "for (i = product = 1; i <= 5; i = i + 1) { product = product * i; }",
        &["(begin (assign! i (assign! product 1)) (while (<= i 5) (begin (begin (assign! product (* product i))) (assign! i (+ i 1)))))"],
    );
    assert_stmts(
        "for (;;) { product = product * i; }",
        &["(begin (while true (begin (begin (assign! product (* product i))))))"],
    );
}

#[test]
fn jump_stmt() {
    assert_stmts(
        indoc! {"
            while (true) {
                if (i == 3 or i == 5) {
                    i = i + 1;
                    continue;
                }
                product = product * i;
                i = i + 1;
                if (i > 6) break;
            }
        "},
        &["(while true (begin (if (or (== i 3) (== i 5)) (begin (assign! i (+ i 1)) (continue))) (assign! product (* product i)) (assign! i (+ i 1)) (if (> i 6) (break))))"],
    );
}

#[test]
#[should_panic(expected = "expected `;` after the Condition Clause")]
fn for_stmt_typo() {
    assert_stmts(
        "for (i = product = 1; i <= 5, i = i + 1) { product = product * i; }",
        &[],
    );
}

#[test]
fn foo() {
    assert_stmts("foo;", &["foo"]);
}

#[test]
#[should_panic(expected = "expected `;` after a value")]
fn foo_no_semicolon() {
    assert_stmts("foo", &[""]);
}

#[test]
fn print_stmt_var() {
    assert_stmts("print foo;", &["(print foo)"]);
}

#[test]
fn var_decl() {
    assert_stmts(
        "var foo=-(-1+2) >=3;",
        &["(var foo (>= (- (+ (- 1) 2)) 3))"],
    );
}

#[test]
fn var_decl_empty() {
    assert_stmts("var foo;", &["(var foo)"]);
}

#[test]
fn block_stmt() {
    assert_stmts(
        "var foo; { var bar = 1; print bar; } var baz;",
        &["(var foo)", "(begin (var bar 1) (print bar))", "(var baz)"],
    );
}

#[test]
#[should_panic(expected = "expected `}` to finish the block")]
fn block_stmt_no_rightbrace() {
    assert_stmts("var foo; { var bar = 1; print bar; var baz;", &[]);
}

#[test]
fn fun_call() {
    assert_stmts(
        "func (c) (u, r) (r(y), i) (n) (g) ();",
        &["((((((func c) u r) (r y) i) n) g) )"],
    );
}

#[test]
#[should_panic(expected = "expected `)` to end the parameter list")]
fn fun_call_typo() {
    assert_stmts("func (c) (u, r (r(y), i) (n) (g) ();", &[]);
}

#[test]
fn fun_decl() {
    assert_stmts("fun foo() { }", &["(fun foo () '())"]);
    assert_stmts(
        "fun foo_bar(a, b, c, d) { print a * b - c / d; }",
        &["(fun foo_bar (a b c d) (print (- (* a b) (/ c d))))"],
    );
    assert_stmts(
        "fun foo_bar(a, b, c, d) { return a * b - c / d; }",
        &["(fun foo_bar (a b c d) (return (- (* a b) (/ c d))))"],
    );
}

#[test]
fn lambda_expr_stmt() {
    assert_stmts("(fun () {});", &["(lambda () '())"]);
    assert_stmts("g(fun () {});", &["(g (lambda () '()))"]);
}

#[test]
fn class_decl() {
    assert_stmts(
        indoc! {r#"
            class Foo {
                bar(baz, boo) {
                    return "Boom";
                }
            }
        "#},
        &[r#"(class Foo ((fun bar (baz boo) (return "Boom"))))"#],
    );
}
