use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use once_cell::unsync::OnceCell;
use widestring::{U16CStr, U16CString, U16String};

use crate::toolhelp::ProcessIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Inject the DLL into the target process
    Inject,
    /// Tries to eject the DLL from the target process
    Eject,
    /// Combination of `Eject` followed by `Inject`
    Reload
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Interpret the process argument as PID
    #[arg(long, short)]
    pub pid: bool,
    /// Create a copy of the DLL before injecting to allow for easier overwriting
    #[arg(long, short)]
    pub copy: bool,
    /// What mode to run the program in
    #[arg(long, short, value_enum, default_value_t = Mode::Inject)]
    pub mode: Mode,
    /// The path of the DLL file
    pub path: PathBuf,
    /// The process name
    pub process: OsString,

    #[arg(skip)]
    copy_path: OnceCell<PathBuf>,
    #[arg(skip)]
    final_path: OnceCell<U16CString>
}

impl Args {
    pub fn pid(&self) -> Result<u32> {
        match self.pid {
            true => {
                log::trace!("Interpreting <PROCESS> as pid");
                let pid = self.process.to_string_lossy().parse()?;
                Ok(pid)
            }
            false => {
                let name = U16String::from_os_str(&self.process);
                log::trace!("Searching for process with name: {}", name.display());
                let pid = ProcessIter::new()?
                    .find(|p| p.name() == name)
                    .context("Can not find specified process")?
                    .pid();
                Ok(pid)
            }
        }
    }

    pub fn copy_path(&self) -> Option<&PathBuf> {
        match self.copy {
            true => Some(
                self.copy_path
                    .get_or_init(|| self.path.with_extension("copy.dll"))
            ),
            false => None
        }
    }

    pub fn copy_dll(&self) -> Result<()> {
        if let Some(copy_path) = self.copy_path() {
            std::fs::copy(&self.path, copy_path)?;
        }
        Ok(())
    }

    pub fn final_path(&self) -> Result<&U16CStr> {
        let buf = self.final_path.get_or_try_init(|| {
            self.copy_path()
                .unwrap_or(&self.path)
                .canonicalize()
                .context("Failed to find DLL file")
                .and_then(|path| U16CString::from_os_str(path.into_os_string()).context("Invalid path"))
                .map(|path| {
                    log::debug!("Resolved DLL path to {}", path.display());
                    path
                })
        })?;
        Ok(buf)
    }
}
