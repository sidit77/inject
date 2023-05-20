mod process;
mod args;

use clap::Parser;
use log::LevelFilter;
use widestring::U16String;
use crate::args::Args;
use crate::process::{ModuleIter, ProcessIter};


fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .format_target(false)
        .init();

    log::info!("Test");
    let process = U16String::from_os_str(&args.process);
    ProcessIter::new()
        .unwrap()
        .filter(|p| p.name() == process)
        .take(1)
        .flat_map(|p| ModuleIter::new(p.pid()).unwrap())
        .for_each(|p| log::info!("{}", p.name().display()));
}
