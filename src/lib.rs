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

// pub(crate) mod cmd;
pub(crate) mod error;
// pub(crate) mod interpreter;
pub(crate) mod syntax;
// pub(crate) mod resolver;
// pub(crate) mod run;
pub(crate) mod util;

// pub use crate::cmd::Dolores;
pub(crate) use self::syntax::{lexer, parser};
