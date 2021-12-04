use std::{convert::Infallible, fmt::Display, path::Path};

use anyhow::Result;
use rustyline::{error::ReadlineError, Editor};

use crate::{
    interpreter::Object,
    lexer::Lexer,
    parser::{Parser, Stmt},
};

#[macro_export]
macro_rules! bail {
    ($pos:expr, $ctx:expr, $msg:expr $(,)?) => {
        anyhow::bail!("{}", crate::run::error_report($pos, $ctx, $msg))
    };
    ($pos:expr, $ctx:expr, $msg:expr, $( $arg:expr ),+ $(,)?) => {
        anyhow::bail!("{}", crate::run::error_report(
            $pos,
            $ctx,
            format!($msg, $( $arg ),+),
        ))
    };
}

pub(crate) fn error_report(pos: (usize, usize), ctx: &str, msg: impl Display) -> String {
    format!("[L{}:{}] Error {}: {}", pos.0, pos.1, ctx, msg)
}

pub(crate) fn run_file(path: impl AsRef<Path>) -> Result<()> {
    let contents = std::fs::read_to_string(path)?;
    run(&contents)
}

pub(crate) fn run_prompt() -> Result<()> {
    let mut reader = Editor::<()>::new();
    loop {
        match reader.readline(">>> ") {
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break Ok(()),
            ln => run(&ln?).unwrap_or_else(|e| println!("{}", e)),
        }
    }
}

fn run(src: &str) -> Result<()> {
    let tokens = Lexer::new(src).analyze();
    let mut parser = Parser::new(tokens);
    let res = parser.run()?.into_iter().try_for_each(Stmt::eval)?;
    // println!("==> {}", res);
    Ok(())
}
