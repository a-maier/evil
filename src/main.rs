#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::{Arc, Mutex};

use log::info;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();

    let event_files = std::env::args().skip(1);
    let events = Arc::new(Mutex::new(Vec::new()));
    for event_file in event_files {
        info!("Loading events from {event_file}");
        let events = events.clone();
        eframe::run_native(
            "load events",
            native_options.clone(),
            Box::new(move |cc| Box::new(evil::EventLoader::new(cc, events, event_file))),
        )?;
    }
    let events: Mutex<_> = Arc::into_inner(events).unwrap();
    let events = events.into_inner().unwrap();
    info!("Loaded {} events", events.len());

    eframe::run_native(
        "evil",
        native_options,
        Box::new(|cc| Box::new(evil::TemplateApp::new(cc, events))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(evil::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
