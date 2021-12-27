use std::{fmt::Display, path::Path};

use anyhow::Result;
use itertools::Itertools;
use rustyline::{error::ReadlineError, Editor};

use crate::{
    interpreter::{self, Env, Interpreter, RcCell},
    lexer::Lexer,
    parser::Parser,
};

pub(crate) fn run_file(path: impl AsRef<Path>) -> Result<()> {
    let interpreter = &mut Interpreter::default();
    let contents = std::fs::read_to_string(path)?;
    run(&contents, interpreter, false)
}

pub(crate) fn run_prompt() -> Result<()> {
    let interpreter = &mut Interpreter::default();
    let mut reader = Editor::<()>::new();
    loop {
        match reader.readline(">>> ") {
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break Ok(()),
            ln => run(&ln?, interpreter, true).unwrap_or_else(|e| println!("{}", e)),
        }
    }
}

fn run(src: &str, interpreter: &mut Interpreter, repl_mode: bool) -> Result<()> {
    let tokens = Lexer::new(src).analyze().collect_vec();
    if let Err(e) = Parser::new(tokens.clone())
        .run()
        .and_then(|stmts| stmts.into_iter().try_for_each(|it| interpreter.exec(it)))
    {
        // In REPL mode, if the user types an expression instead of a statement, the
        // value of that expression is automatically printed out.
        if repl_mode {
            if let Ok(expr) = Parser::new(tokens).expr() {
                interpreter
                    .eval(expr)
                    .map_or_else(|e| println!("<<< {:?}", e), |expr| println!("<<< {}", expr));
                return Ok(());
            }
        }
        println!("{:?}", e);
    }
    Ok(())
}
