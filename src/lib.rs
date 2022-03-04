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
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::enum_glob_use
)]

pub(crate) mod cmd;
pub(crate) mod error;
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod run;
pub(crate) mod util;
pub(crate) mod vm;

pub use crate::cmd::Dolores;
