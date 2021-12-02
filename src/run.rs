use std::{convert::Infallible, fmt::Display, path::Path};

use anyhow::{bail, Result};
use rustyline::{error::ReadlineError, Editor};

pub(crate) fn bail(pos: (usize, usize), ctx: &str, message: impl Display) -> Result<Infallible> {
    bail!("[L{}:{}] Error {}: {}", pos.0, pos.1, ctx, message)
}

pub(crate) fn run_file(path: impl AsRef<Path>) -> Result<()> {
    let contents = std::fs::read_to_string(path)?;
    run(&contents)
}

pub(crate) fn run_prompt() -> Result<()> {
    let mut reader = Editor::<()>::new();
    loop {
        match reader.readline("> ") {
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break Ok(()),
            ln => run(&ln?).unwrap_or_else(|e| println!("{}", e)),
        }
    }
}

fn run(src: &str) -> Result<()> {
    todo!("should read source file: {}", src);
}
