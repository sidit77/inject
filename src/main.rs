mod process;
mod args;

use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use crate::args::Args;
use crate::process::ModuleIter;


fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .format_target(false)
        .init();

    let pid = args.pid()?;
    log::info!("Listing module of pid {}", pid);

    for module in ModuleIter::new(pid)? {
        log::info!("{}", module.name().display());
    }
    Ok(())
}
