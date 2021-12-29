#![cfg(test)]
#![allow(clippy::enum_glob_use)]

use anyhow::Result;
use indoc::indoc;
use pretty_assertions::assert_eq;

use super::*;
use crate::run::run_str;

fn assert_eval(pairs: &[(&str, &str)]) -> Result<()> {
    let interpreter = &mut Interpreter::default();
    pairs.iter().try_for_each(|(src, expected)| {
        let got = run_str(src, interpreter, true)?;
        assert_eq!(expected, &got, "unexpected output for `{}`", src);
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
fn var_and_block() -> Result<()> {
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
            "{ foo = foo + 1; var bar; var foo1 = foo; foo1 = foo1 + 1; }",
            "",
        ),
        ("foo", "3"),
    ])
}

#[test]
#[should_panic(expected = "cannot read local Variable `foo` in its own initializer")]
fn var_own_init() {
    assert_eval(&[("var foo = 2;", ""), ("{ var foo = foo; }", "")]).unwrap();
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
fn for_stmt_return_in_dup_fun() -> Result<()> {
    assert_eval(&[(
        "for(var i = 0; i < 10; i = i + 1) { fun g() { return; } }",
        "",
    )])
}

#[test]
#[should_panic(expected = "found `break` out of loop context")]
fn bare_jump_break() {
    assert_eval(&[("break;", "")]).unwrap();
}

#[test]
#[should_panic(expected = "found `break` out of loop context")]
fn bare_jump_break_in_fun() {
    assert_eval(&[(
        "for(var i = 0; i < 10; i = i + 1) { fun g() { break; } }",
        "",
    )])
    .unwrap();
}

#[test]
#[should_panic(expected = "found `continue` out of loop context")]
fn bare_jump_continue() {
    assert_eval(&[("continue;", "")]).unwrap();
}

#[test]
#[should_panic(expected = "found `continue` out of loop context")]
fn bare_jump_continue_in_fun() {
    assert_eval(&[(
        "for(var i = 0; i < 10; i = i + 1) { fun g() { continue; } }",
        "",
    )])
    .unwrap();
}

#[test]
#[should_panic(expected = "found `return` out of function context")]
fn bare_return() {
    assert_eval(&[("return true;", "")]).unwrap();
}

#[test]
fn bare_return_in_fun_loop() -> Result<()> {
    assert_eval(&[
        (
            indoc! {"
                fun fact(n) {
                    var i; var product;
                    for (i = product = 1;; i = i + 1) {
                        product = product * i;
                        if (i >= n) { return product; }
                    }
                }
            "},
            "",
        ),
        ("fact(5)", "120"),
    ])
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
fn fun_rec() -> Result<()> {
    assert_eval(&[
        (
            "fun fact(i) { if (i <= 0) { return 1; } return i * fact(i - 1); }",
            "",
        ),
        ("fact(5)", "120"),
    ])
}

#[test]
fn fun_late_init() -> Result<()> {
    assert_eval(&[
        ("var a = 3;", ""),
        ("fun f() { return a; }", ""),
        ("a = 4;", ""),
        ("f()", "4"),
    ])
}

#[test]
fn fun_decl_order() -> Result<()> {
    assert_eval(&[
        (
            "fun fact(i) { if (i <= 0) { return one(); } return i * fact(i - 1); }",
            "",
        ),
        ("fun one() { return 1; }", ""),
        ("fact(5)", "120"),
    ])
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
fn fun_lambda_inline() -> Result<()> {
    assert_eval(&[("(fun (x) { return x + 2; })(3)", "5")])
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

const MAN_OR_BOY: &str = indoc! {r#"
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
"#};

#[test]
fn fun_man_or_boy_4() -> Result<()> {
    assert_eval(&[(MAN_OR_BOY, ""), ("A(4, I1, I_1, I_1, I1, I0)", "1")])
}

#[test]
#[ignore = "stack consuming"]
fn fun_man_or_boy_10() -> Result<()> {
    // src: https://rosettacode.org/wiki/Man_or_boy_test#Lox
    fn inner() -> Result<()> {
        assert_eval(&[(MAN_OR_BOY, ""), ("A(10, I1, I_1, I_1, I1, I0)", "-67")])
    }
    // HACK: We use a new thread with 32 MiB of stack to avoid stack overflow...
    // src: https://stackoverflow.com/a/44042122
    let builder = std::thread::Builder::new().stack_size(64 * 1024 * 1024);
    let handler = builder.spawn(inner).unwrap();
    handler.join().unwrap()?;
    Ok(())
}

#[test]
fn class_empty() -> Result<()> {
    assert_eval(&[
        ("class Foo {}", ""),
        (
            indoc! {r#"
                class DevonshireCream {
                    serveOn() {
                        return "Scones";
                    }
                }
            "#},
            "",
        ),
        ("Foo", "<class: Foo>"),
        ("DevonshireCream", "<class: DevonshireCream>"),
    ])
}

#[test]
fn class_get_set() -> Result<()> {
    assert_eval(&[
        ("class Foo {}", ""),
        ("var f = Foo();", ""),
        ("f.bar = 10086;", ""),
        ("f.bar", "10086"),
        (r#"f.bar = "foobar""#, r#""foobar""#),
        ("f.bar", r#""foobar""#),
    ])
}

#[test]
#[should_panic(expected = "property `bar` undefined for the given object")]
fn class_get_undefined() {
    assert_eval(&[
        ("class Foo {}", ""),
        ("var f = Foo();", ""),
        (r#"f.bar""#, ""),
    ])
    .unwrap();
}

#[test]
#[should_panic(expected = "the object `true` cannot have properties")]
fn class_get_invalid() {
    assert_eval(&[("true.story", "")]).unwrap();
}

#[test]
#[should_panic(expected = "the object `true` cannot have properties")]
fn class_set_invalid() {
    assert_eval(&[("true.story = 42", "")]).unwrap();
}

#[test]
fn class_method_simple() -> Result<()> {
    assert_eval(&[
        (r#"class Bacon { eat() { return "crunch"; } }"#, ""),
        ("Bacon().eat()", r#""crunch""#),
    ])
}

#[test]
fn class_method_this_simple() -> Result<()> {
    assert_eval(&[
        (
            r#"class Egotist { speak() { return "Just " + this.name; } }"#,
            "",
        ),
        (r#"var jimmy = Egotist(); jimmy.name = "Jimmy";"#, ""),
        ("jimmy.speak()", r#""Just Jimmy""#),
    ])
}

#[test]
fn class_method_this_save() -> Result<()> {
    assert_eval(&[
        (
            r#"class Egotist { speak() { return "Just " + this.name; } }"#,
            "",
        ),
        (r#"var jimmy = Egotist(); jimmy.name = "Jimmy";"#, ""),
        ("var f = jimmy.speak;", ""),
        ("f()", r#""Just Jimmy""#),
    ])
}

#[test]
#[should_panic(expected = "found `this` out of class context")]
fn bare_this() {
    assert_eval(&[("this;", "")]).unwrap();
}
