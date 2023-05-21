mod args;
mod toolhelp;
mod process;

use anyhow::Result;
use clap::Parser;
use log::LevelFilter;

use crate::args::Args;
use crate::process::{ProcessHandle, ProcessMemory};

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .format_target(false)
        .init();

    args.copy_dll()?;
    log::info!("DLL path: {}", args.final_path()?.display());

    let pid = args.pid()?;
    log::info!("Listing module of pid {}", pid);

    let process = ProcessHandle::open(pid)?;
    let memory = ProcessMemory::new(&process, args.final_path()?.as_slice_with_nul())?;

    //for module in ModuleIter::new(pid)? {
    //    log::info!("{}", module.name().display());
    //}
    Ok(())
}
