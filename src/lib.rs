#![forbid(unsafe_code)]
#![warn(
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    // missing_docs,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts,
    unused_allocation
)]
// TODO: Remove the whitelist below.
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod cmd;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod run;
pub mod util;
