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
