mod auto_decompress;
mod event;
mod import;
mod opt;
mod particle;
mod plot;

use std::fs::File;
use std::io::Write;

use crate::import::import;
use crate::opt::Opt;
use crate::plot::plot;

use anyhow::Result;
use env_logger::Env;
use log::{info, debug};
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

    for (n, event) in events.iter().enumerate() {
        info!("Plotting event number {}", n);
        plot(event, &format!("event_{}.svg", n))?;
    }
    Ok(())
}
