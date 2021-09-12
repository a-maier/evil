use std::ffi::OsString;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "evil",
    about = "Visualise collider events"
)]
pub struct Opt {
    /// Verbosity level: 'off', 'error', 'warn', 'info', 'debug', 'trace'
    #[structopt(short, long, default_value = "info")]
    pub verbosity: String,

    /// Event files to import
    pub files: Vec<OsString>,
}
