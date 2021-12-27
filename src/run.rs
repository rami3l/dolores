use std::path::Path;

use anyhow::Result;
use itertools::Itertools;
use rustyline::{error::ReadlineError, Editor};

use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

pub(crate) fn run_file(path: impl AsRef<Path>) -> Result<()> {
    let interpreter = &mut Interpreter::default();
    let contents = std::fs::read_to_string(path)?;
    run(&contents, interpreter, false);
    Ok(())
}

pub(crate) fn run_prompt() -> Result<()> {
    let interpreter = &mut Interpreter::default();
    let mut reader = Editor::<()>::new();
    loop {
        match reader.readline(">>> ") {
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break Ok(()),
            ln => run(&ln?, interpreter, true),
        }
    }
}

pub(crate) fn run_str(src: &str, interpreter: &mut Interpreter, repl_mode: bool) -> Result<String> {
    let tokens = Lexer::new(src).analyze().collect_vec();
    let res = Parser::new(tokens.clone()).run().and_then(|stmts| {
        interpreter.resolve_stmts(stmts.clone())?;
        interpreter.exec_stmts(stmts)
    });
    match res {
        Ok(_) => Ok("".into()),
        // In REPL mode, if the user types an expression instead of a statement, the
        // value of that expression is automatically printed out.
        Err(_e) if repl_mode => Parser::new(tokens)
            .expr()
            .and_then(|expr| interpreter.eval(expr))
            .map(|obj| format!("{}", obj)),
        Err(e) => Err(e),
    }
}

fn run(src: &str, interpreter: &mut Interpreter, repl_mode: bool) {
    run_str(src, interpreter, repl_mode).map_or_else(
        |e| println!("{:?}", e),
        |expr| {
            if !expr.is_empty() {
                println!("<<< {}", expr);
            }
        },
    );
}
