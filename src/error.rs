use std::fmt::Display;

use thiserror::Error;

use crate::util::SrcPos;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Parsing Error at {pos:?}: {msg}")]
    ParseError { pos: SrcPos, msg: String },

    #[error("Runtime Error: Operation `{op}` is unimplemented for `{args}`")]
    #[allow(missing_docs)]
    OperationUnimplementedError { op: String, args: String },

    // #[error(transparent)]
    // IoError(#[from] io::Error),
    #[error("{0}")]
    OtherError(String),
}
