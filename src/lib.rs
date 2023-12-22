#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod clustering;
mod event;
mod export;
mod font;
mod particle;
mod plotter;
mod windows;

pub use app::TemplateApp;
pub use event::Event;
