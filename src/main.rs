mod args;
mod toolhelp;
mod process;
mod functions;

use anyhow::{ensure, Result};
use clap::Parser;
use log::LevelFilter;

use crate::args::Args;
use crate::functions::LOAD_LIBRARY_W;
use crate::process::{ProcessHandle, ProcessMemory, ProcessThread};

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
    let thread = ProcessThread::spawn(&process, *LOAD_LIBRARY_W, Some(memory.to_ptr()))?;
    ensure!(thread.join()? != 0, "Failed to load library");

    //for module in ModuleIter::new(pid)? {
    //    log::info!("{}", module.name().display());
    //}
    Ok(())
}
