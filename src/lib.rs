#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod clustering;
mod event;
mod event_loader;
mod font;
mod particle;
mod plotter;
mod windows;

pub use app::TemplateApp;
pub use event::Event;
pub use event_loader::EventLoader;
