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
