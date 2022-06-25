use derive_more::{Display, Error, From};

use super::Object;

#[derive(Debug, Error, Display, Clone, Copy)]
#[display(fmt = "Internal Error: found `break` out of loop context")]
pub(crate) struct BreakMarker;

#[derive(Debug, Error, Display, Clone, Copy)]
#[display(fmt = "Internal Error: found `continue` out of loop context")]
pub(crate) struct ContinueMarker;

#[derive(Debug, Error, Display, From)]
#[display(fmt = "Internal Error: found `return` out of function context")]
pub(crate) struct ReturnMarker(#[error(not(source))] pub(crate) Object);

// ! This is a HIGHLY DANGEROUS hack and non-idiomatic Rust.
// SAFETY: Those `unsafe impl`s are required because `ReturnMarker` abuses
// `anyhow::Error` to return a value in our interpreter, however `anyhow::Error`
// requires `Send + Sync` to work.
// It's safe to do so here because:
// - Those types are `pub(crate)` only;
// - Our interpreter won't be sending `ReturnMarker`s across thread boundaries.
// See: <https://github.com/dtolnay/anyhow/issues/81>
unsafe impl Send for ReturnMarker {}
unsafe impl Sync for ReturnMarker {}
