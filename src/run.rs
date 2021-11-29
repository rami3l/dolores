use std::{convert::Infallible, path::Path};

use anyhow::{bail, Result};
use rustyline::{error::ReadlineError, Editor};

use crate::lexer::tokenize;

pub(crate) fn bail(line_num: u64, place: &str, message: &str) -> Result<Infallible> {
    bail!("[L{}] Error {}: {}", line_num, place, message)
}

pub(crate) fn run_file(path: impl AsRef<Path>) -> Result<()> {
    let contents = std::fs::read_to_string(path)?;
    run(&contents)
}

pub(crate) fn run_prompt() -> Result<()> {
    let mut reader = Editor::<()>::new();
    loop {
        let ln = reader.readline("> ");
        match ln {
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break Ok(()),
            ln => run(&ln?).unwrap_or_else(|e| println!("{}", e)),
        }
    }
}

fn run(src: &str) -> Result<()> {
    println!("{:?}", tokenize(src));
    Ok(())
    // todo!("should read source file: {}", src);
}
