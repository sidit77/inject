mod args;
mod functions;
mod process;
mod toolhelp;

use anyhow::{ensure, Result};
use clap::Parser;
use log::Level;
use widestring::U16CStr;

use crate::args::{Args, Mode};
use crate::functions::{FREE_LIBRARY, LOAD_LIBRARY_W};
use crate::process::{ProcessHandle, ProcessMemory, ProcessThread};
use crate::toolhelp::ModuleIter;

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.level.into())
        .format_target(false)
        .init();

    let pid = args.pid()?;
    log::debug!("Process id is {}", pid);

    match args.mode {
        Mode::Inject => {
            args.copy_dll()?;
            inject_library(pid, args.final_path()?)?;
        }
        Mode::Eject => {
            eject_library(pid, args.final_path()?)?;
        }
        Mode::Reload => {
            let path = args.final_path()?;
            eject_library(pid, path)?;
            args.copy_dll()?;
            inject_library(pid, path)?;
        }
    }

    if log::log_enabled!(Level::Trace) {
        log::trace!("Final module list:");
        for module in ModuleIter::new(pid)? {
            log::trace!("    {}", module.name().display());
        }
    }

    Ok(())
}

fn inject_library(pid: u32, dll: &U16CStr) -> Result<()> {
    log::trace!("Trying to inject DLL into process");
    let process = ProcessHandle::open(pid)?;
    let memory = ProcessMemory::new(&process, dll.as_slice_with_nul())?;
    let thread = ProcessThread::spawn(&process, *LOAD_LIBRARY_W, Some(memory.to_ptr()))?;
    ensure!(thread.join()? != 0, "Failed to load library");
    log::info!("Successfully inject DLL");
    Ok(())
}

fn eject_library(pid: u32, dll: &U16CStr) -> Result<()> {
    log::trace!("Trying to eject DLL from process");
    let module = ModuleIter::new(pid)?
        .find(|m| m.path() == dll)
        .map(|m| m.handle());
    match module {
        None => log::info!("Can't find DLL in process"),
        Some(module) => {
            let process = ProcessHandle::open(pid)?;
            let thread = ProcessThread::spawn(&process, *FREE_LIBRARY, Some(module.0 as _))?;
            ensure!(thread.join()? != 0, "Failed to free library");
            log::info!("Successfully ejected DLL");
        }
    }
    Ok(())
}
