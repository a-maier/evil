use std::default::Default;
use std::ops::Range;

use crate::image::Image;
use crate::event::Event;
use crate::config::Config;
use crate::plotter::Plotter;

use eframe::{egui, epi};
use log::{error, debug, trace};
use serde::{Deserialize, Serialize};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    plotter: Plotter,

    #[serde(skip)]
    events: Vec<Event>,

    #[serde(skip)]
    cur_ev_idx: usize,

    #[serde(skip)]
    y_phi: Image,

    #[serde(skip)]
    y_phi_id: egui::TextureId,

    #[serde(skip)]
    y_logpt: Image,

    #[serde(skip)]
    y_logpt_id: egui::TextureId,

    #[serde(skip)]
    first_draw: bool,

    #[serde(skip)]
    ev_idx_str: String,

    #[serde(skip)]
    ev_idx_str_col: Option<egui::Color32>,

    #[serde(skip)]
    lpt_range: Range<f64>,

    window_size: (f32, f32),
}

impl App {

    pub fn new(events: Vec<Event>) -> Self {

        let plotter = Plotter::new();

        // TODO: itertools::minmax is more efficient
        let mut min = f64::MAX;
        let mut max = f64::MIN_POSITIVE;
        for ev in &events {
            for p in &ev.out {
                if p.pt < min {
                    min = p.pt
                }
                if p.pt > max {
                    max = p.pt
                }
            }
        }
        let lpt_range = min.log10() .. max.log10();
        debug!("logpt range: {}..{}", lpt_range.start, lpt_range.end);

        let y_phi;
        let y_logpt;
        if let Some(event) = events.first() {
            y_phi = plotter.plot_y_phi(event);
            y_logpt = plotter.plot_y_logpt(event, lpt_range.clone());
        } else {
            let ev = Event::default();
            y_phi = plotter.plot_y_phi(&ev);
            y_logpt = plotter.plot_y_logpt(&ev, lpt_range.clone());
        };
        App {
            plotter,
            events,
            cur_ev_idx: 0,
            y_phi: Image::new(y_phi.unwrap(), (640, 480)),
            y_phi_id: Default::default(),
            y_logpt: Image::new(y_logpt.unwrap(), (640, 480)),
            y_logpt_id: Default::default(),
            first_draw: true,
            ev_idx_str: "1".to_string(),
            ev_idx_str_col: None,
            lpt_range,
            window_size: Default::default()
        }
    }

    fn update_ev(&mut self, ev_idx: usize, allocator: &mut dyn epi::TextureAllocator) {
        self.cur_ev_idx = ev_idx;
        self.ev_idx_str = format!("{}", ev_idx + 1);
        self.update_img(allocator);
    }

    fn update_img(&mut self, allocator: &mut dyn epi::TextureAllocator) {
        debug_assert!(self.cur_ev_idx < self.events.len());
        debug!("Update image to event {}/{}", self.cur_ev_idx, self.events.len());
        let svg = self.plotter.plot_y_phi(&self.events[self.cur_ev_idx]).unwrap();
        self.y_phi = Image::new(svg, (1280, 960));
        allocator.free(self.y_phi_id);
        self.y_phi_id = allocator
            .alloc_srgba_premultiplied(self.y_phi.size(), &self.y_phi.pixels());

        let svg = self.plotter.plot_y_logpt(
            &self.events[self.cur_ev_idx],
            self.lpt_range.clone()
        ).unwrap();
        self.y_logpt = Image::new(svg, (1280, 960));
        allocator.free(self.y_logpt_id);
        self.y_logpt_id = allocator
            .alloc_srgba_premultiplied(self.y_logpt.size(), &self.y_logpt.pixels());
    }

    fn prev_img(&mut self, frame: &mut epi::Frame<'_>) {
        if !self.events.is_empty() {
            let new_idx = if self.cur_ev_idx == 0 {
                self.events.len() - 1
            } else {
                self.cur_ev_idx - 1
            };
            self.update_ev(new_idx, frame.tex_allocator())
        }
    }

    fn next_img(&mut self, frame: &mut epi::Frame<'_>) {
        if !self.events.is_empty() {
            self.update_ev(
                self.cur_ev_idx.wrapping_add(1) % self.events.len(),
                frame.tex_allocator()
            )
        }
    }

    fn handle_keys(&mut self, input: &egui::InputState, frame: &mut epi::Frame<'_>) {
        use egui::Key;
        use egui::Event;

        let key_events = input.events.iter().filter_map(
            |ev| if let Event::Key{ key, pressed, modifiers } = ev {
                Some((key, pressed, modifiers))
            } else {
                None
            }
        );

        for (key, pressed, modifiers) in key_events {
            if modifiers.ctrl && key == &Key::Q {
                frame.quit();
            } else if *pressed {
                match key {
                    Key::ArrowRight => self.next_img(frame),
                    Key::ArrowLeft => self.prev_img(frame),
                    _ => {}
                }
            }
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "evil"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.handle_keys(ctx.input(), frame);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit\t(Ctrl+q)").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        if !self.events.is_empty() {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                egui::Grid::new("bottom_panel_grid").show(ui, |ui| {
                    if ui.add(egui::Button::new("<-")).clicked() {
                        self.prev_img(frame)
                    }
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.ev_idx_str)
                            .text_color_opt(self.ev_idx_str_col)
                    );
                    if response.changed() {
                        match self.ev_idx_str.parse::<usize>() {
                            Ok(ev_idx) if ev_idx > 0 && ev_idx <= self.events.len() => {
                                self.update_ev(ev_idx - 1, frame.tex_allocator());
                                self.ev_idx_str_col = None;
                            },
                            _ => {
                                self.ev_idx_str_col = Some(egui::Color32::RED);
                            }
                        };
                    }
                    ui.label(format!("/{}", self.events.len()));
                    if ui.add(egui::Button::new("->")).clicked() {
                        self.next_img(frame)
                    }
                });
            });
        }

        if self.first_draw {
            self.y_phi_id = frame
                .tex_allocator()
                .alloc_srgba_premultiplied((640, 480), &self.y_phi.pixels());
            self.y_logpt_id = frame
                .tex_allocator()
                .alloc_srgba_premultiplied((640, 480), &self.y_logpt.pixels());
            self.first_draw = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let img_height = ui.available_height() / 2.;
            let plot_width = ui.available_width() / 2.;
            trace!("nominal size: {} x {}", plot_width, img_height);
            ui.columns(2, |col| {
                col[0].image(self.y_phi_id, [plot_width, img_height]);
                col[1].image(self.y_logpt_id, [plot_width, img_height]);
            });
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }

        let size = ctx.used_size();
        self.window_size = (size.x, size.y);
    }

    fn on_exit(&mut self) {
        let cfg = Config{
            window_size: Some(self.window_size)
        };
        if let Err(err) = confy::store("evil", &cfg) {
            error!("Failed to save config: {}", err);
        }
    }
}
