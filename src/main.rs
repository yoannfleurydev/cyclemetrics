use anyhow::Result;
use clap::Parser;
use cyclemetrics::{Args, run_cyclemetrics};

fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    run_cyclemetrics(args)
}
