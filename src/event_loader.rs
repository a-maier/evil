use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::event::Event;

use event_file_reader::EventFileReader as Reader;

pub struct EventLoader {
    events: Arc<Mutex<Vec<Event>>>,
    file: PathBuf,
    worker: Option<JoinHandle<()>>,
    worker_is_finished: Arc<AtomicBool>, // TODO: is worker.is_finished() good enough?
}

impl EventLoader {
    pub fn new<P: AsRef<Path>>(
        _cc: &eframe::CreationContext<'_>,
        events: Arc<Mutex<Vec<Event>>>,
        file: P
    ) -> Self {
        Self {
            events,
            file: file.as_ref().to_owned(),
            worker: None,
            worker_is_finished: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl eframe::App for EventLoader {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.worker.is_none() {
            let file = self.file.clone();
            let ctx: egui::Context = ctx.clone();
            let events = self.events.clone();
            let worker_is_finished = self.worker_is_finished.clone();
            self.worker = Some(thread::spawn(move || {
                // TODO: handle errors
                let reader = Reader::new(file.as_path()).unwrap();
                events.lock().unwrap()
                    .extend(reader.map(|e| e.unwrap().into()));
                worker_is_finished.store(true, Ordering::Relaxed);
                ctx.request_repaint();
            }))
        }
        if self.worker_is_finished.load(Ordering::Relaxed) {
            let worker = self.worker.take().unwrap();
            let _ = worker.join();
            frame.close();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
           ui.heading(format!("Loading events from {}", self.file.display()));
       });
    }
}
