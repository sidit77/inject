//mod process;
mod args;

use clap::Parser;
use log::LevelFilter;
use crate::args::Args;


fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .format_target(false)
        .init();

    log::info!("Test");

    println!("{:?}", args);
}
