use anyhow::Result;
use clap::Parser;
use dolores::cmd::Dolores;

fn main() -> Result<()> {
    Dolores::parse().dispatch()
}
