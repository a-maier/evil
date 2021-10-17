mod app;
mod auto_decompress;
mod config;
mod event;
mod image;
mod import;
mod opt;
mod particle;
mod plotter;

use crate::app::App;
use crate::config::Config;
use crate::import::import;
use crate::opt::Opt;

use anyhow::Result;
use env_logger::Env;
use log::{debug, error};
use structopt::StructOpt;

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let env = Env::default().filter_or("EVIL_LOG", opt.verbosity.as_str());
    env_logger::init_from_env(env);

    let mut events = Vec::new();
    for file in &opt.files {
        debug!("Importing events from {:?}", file);
        import(file.as_ref(), &mut events)?;
    }

    let mut native_options = eframe::NativeOptions::default();
    match confy::load::<Config>("evil") {
        Ok(cfg) => native_options.initial_window_size = cfg.window_size.map(
            |(x, y)| egui::Vec2{x, y}
        ),
        Err(err) => error!("{}", err)
    };

    let app = App::new(events);
    eframe::run_native(Box::new(app), native_options);

    Ok(())
}
