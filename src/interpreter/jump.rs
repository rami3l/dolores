use derive_more::{Display, Error, From};

use super::Object;

#[derive(Debug, Error, Display, Clone, Copy)]
#[display(fmt = "Internal Error: found Break out of loop context")]
pub struct BreakMarker;

#[derive(Debug, Error, Display, Clone, Copy)]
#[display(fmt = "Internal Error: found Continue out of loop context")]
pub struct ContinueMarker;

#[derive(Debug, Error, Display, From)]
#[display(fmt = "Internal Error: found Return out of function context")]
pub struct ReturnMarker(#[error(not(source))] pub Object);

/* TODO: Add semantic analysis for those jumps in the language
 * so that their being out of context becomes a semantic error.
 */
