use std::ffi::OsString;
use std::path::PathBuf;
use clap::{Parser, ValueEnum};

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
}