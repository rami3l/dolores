use anyhow::Result;
use clap::Parser;

use crate::run::{run_file, run_prompt};

/// The command line options to be collected.
#[derive(Debug, Parser)]
#[clap(
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    about = clap::crate_description!(),
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Dolores {
    /// Package name or (sometimes) regex.
    #[clap(name = "FILE")]
    pub(crate) file: Option<String>,
}

impl Dolores {
    pub fn launch() -> Result<()> {
        Self::parse().dispatch()
    }

    pub(crate) fn dispatch(self) -> Result<()> {
        let file = self.file;
        file.map_or_else(run_prompt, run_file)
    }
}
