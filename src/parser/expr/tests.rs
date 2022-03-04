#![cfg(test)]

use pretty_assertions::assert_eq;

use super::*;
use crate::lexer::Lexer;

fn assert_expr(src: &str, expected: &str) {
    let tokens = Lexer::new(src);
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
    let tokens = Lexer::new("-(-1+2 / 3- 4 *5+ (6/ 7); 8 +9");
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
    let tokens = Lexer::new("* 1-2 == 3");
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
    let tokens = Lexer::new(">= 1+2 == 3");
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
fn fun_call() {
    assert_expr(
        "func (c) (u, r) (r(y), i) (n) (g) ()",
        "((((((func c) u r) (r y) i) n) g))",
    );
}

#[test]
#[should_panic(expected = "expected `)` to end the parameter list")]
fn fun_call_typo() {
    assert_expr("func (c) (u, r (r(y), i) (n) (g) ()", "");
}

#[test]
fn lambda() {
    assert_expr("fun () { }", "(lambda () '())");
    assert_expr("fun () { } ()", "((lambda () '()))");
    assert_expr(
        "fun (a, b, c, d) { print a * b - c / d; }",
        "(lambda (a b c d) (print (- (* a b) (/ c d))))",
    );
}

#[test]
fn class_instance_get() {
    assert_expr(
        "egg.scramble(3).with(cheddar)",
        "((. ((. egg scramble) 3) with) cheddar)",
    );
}

#[test]
fn class_instance_set() {
    assert_expr(
        "breakfast.omelette.filling.meat = ham",
        "(.set! (. (. breakfast omelette) filling) meat ham)",
    );
}

#[test]
fn class_method() {
    assert_expr(
        "he.breakfast(omelette.filledWith(cheese), sausage)",
        "((. he breakfast) ((. omelette filledWith) cheese) sausage)",
    );
}

#[test]
fn class_super() {
    assert_expr("super.method()", "((. (super) method))");
}
