use anyhow::Result;
use clap::Parser;
use hxcfe_cli::HxcfeCli;

fn main() -> Result<()> {
    let cli = HxcfeCli::parse();
    hxcfe_cli::run(&cli)
}
