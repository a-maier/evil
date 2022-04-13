//! An *ev*ent *il*lustrator for Monte Carlo collider events.
//!
//! # How to use
//!
//! You can find precompiled executables on
//! [github](https://github.com/a-maier/evil/releases). Start with
//!
//!     evil EVENTFILE
//!
//! The event file should be in the LHEF or version 2 of the HepMC
//! format and can be compressed (bzip2, gzip, lz4, zstd).
//!
//! If [Rust and Cargo](https://www.rust-lang.org/) are installed on
//! your system, you can of course compile and run directly from the
//! source code:
//!
//!     cargo run --release -- EVENTFILE
//!
mod app;
mod auto_decompress;
mod event;
mod dir_entries;
mod image;
mod import;
mod jets;
mod opt;
mod font;
mod particle;
mod plotter;
mod file_dialog;

use crate::app::App;
use crate::import::import;
use crate::opt::Opt;

use anyhow::Result;
use env_logger::Env;
use log::debug;
use structopt::StructOpt;

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let env = Env::default().filter_or("EVIL_LOG", opt.verbosity.as_str());
    env_logger::init_from_env(env);

    let mut events = Vec::new();
    for file in &opt.files {
        debug!("Importing events from {:?}", file);
        import(file, &mut events)?;
    }

    let app = App::new(events);
    eframe::run_native(Box::new(app), eframe::NativeOptions::default())
}
