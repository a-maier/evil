use std::cmp::Ordering;
use std::default::Default;
use std::ops::Range;

use crate::image::Image;
use crate::event::Event;
use crate::config::Config;
use crate::plotter::Plotter;
use crate::jets::{JetAlgorithm, JetDefinition};

use jetty::PseudoJet;
use log::{error, debug, trace};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct ClusteringSettings {
    enable: bool,
    jet_def: JetDefinition
}

impl Default for ClusteringSettings {
    fn default() -> Self {
        Self {
            enable: false,
            jet_def: JetDefinition{
                algorithm: JetAlgorithm::AntiKt,
                radius: 0.4,
                min_pt: 0.
            }
        }
    }
}

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
    y_phi_id: eframe::egui::TextureId,

    #[serde(skip)]
    y_logpt: Image,

    #[serde(skip)]
    y_logpt_id: eframe::egui::TextureId,

    #[serde(skip)]
    first_draw: bool,

    #[serde(skip)]
    ev_idx_str: String,

    #[serde(skip)]
    ev_idx_str_col: Option<eframe::egui::Color32>,

    #[serde(skip)]
    lpt_range: Range<f64>,

    clustering: ClusteringSettings,

    window_size: (f32, f32),

    #[serde(skip)]
    clustering_settings_open: bool,
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
        let jets = Vec::new();
        if let Some(event) = events.first() {
            y_phi = plotter.plot_y_phi(event, &jets);
            y_logpt = plotter.plot_y_logpt(event, &jets, lpt_range.clone());
        } else {
            let ev = Event::default();
            y_phi = plotter.plot_y_phi(&ev, &jets);
            y_logpt = plotter.plot_y_logpt(&ev, &jets, lpt_range.clone());
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
            clustering: Default::default(),
            window_size: Default::default(),
            clustering_settings_open: false,
        }
    }

    fn cluster_jets(&self, event: &Event) -> Vec<PseudoJet> {
        if self.clustering.enable {
            self.clustering.jet_def.cluster_event(event)
        } else {
            vec![]
        }
    }

    fn update_ev(&mut self, ev_idx: usize, allocator: &mut dyn eframe::epi::TextureAllocator) {
        self.cur_ev_idx = ev_idx;
        self.ev_idx_str = format!("{}", ev_idx + 1);
        self.update_img(allocator);
    }

    fn update_img(&mut self, allocator: &mut dyn eframe::epi::TextureAllocator) {
        debug_assert!(self.cur_ev_idx < self.events.len());
        debug!("Update image to event {}/{}", self.cur_ev_idx, self.events.len());
        let event = &self.events[self.cur_ev_idx];
        let jets = self.cluster_jets(event);
        let svg = self.plotter.plot_y_phi(event, &jets).unwrap();
        self.y_phi = Image::new(svg, (1280, 960));
        allocator.free(self.y_phi_id);
        self.y_phi_id = allocator
            .alloc_srgba_premultiplied(self.y_phi.size(), &self.y_phi.pixels());

        let svg = self.plotter.plot_y_logpt(
            event,
            &jets,
            self.lpt_range.clone(),
        ).unwrap();
        self.y_logpt = Image::new(svg, (1280, 960));
        allocator.free(self.y_logpt_id);
        self.y_logpt_id = allocator
            .alloc_srgba_premultiplied(self.y_logpt.size(), &self.y_logpt.pixels());
    }

    fn prev_img(&mut self, frame: &mut eframe::epi::Frame<'_>) {
        if !self.events.is_empty() {
            let new_idx = if self.cur_ev_idx == 0 {
                self.events.len() - 1
            } else {
                self.cur_ev_idx - 1
            };
            self.update_ev(new_idx, frame.tex_allocator())
        }
    }

    fn next_img(&mut self, frame: &mut eframe::epi::Frame<'_>) {
        if !self.events.is_empty() {
            self.update_ev(
                self.cur_ev_idx.wrapping_add(1) % self.events.len(),
                frame.tex_allocator()
            )
        }
    }

    fn handle_keys(&mut self, input: &eframe::egui::InputState, frame: &mut eframe::epi::Frame<'_>) {
        use eframe::egui::Key;
        use eframe::egui::Event;

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

    fn show_jet_cluster_settings(&mut self, ctx: &eframe::egui::CtxRef) {
        let clustering = &mut self.clustering;
        eframe::egui::Window::new("Jet clustering")
            .open(&mut self.clustering_settings_open)
            .show(ctx, |ui| {
                ui.checkbox(&mut clustering.enable, "Enable jet clustering");
                ui.scope(|ui| {
                    ui.set_enabled(clustering.enable);

                    use JetAlgorithm::*;
                    let algo = &mut clustering.jet_def.algorithm;
                    eframe::egui::ComboBox::from_label( "Jet algorithm")
                        .selected_text(
                            match algo {
                                AntiKt => "anti-kt",
                                Kt => "kt",
                                CambridgeAachen => "Cambridge/Aachen"
                            }
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(algo, AntiKt, "anti-kt");
                            ui.selectable_value(algo, Kt, "kt");
                            ui.selectable_value(algo, CambridgeAachen, "Cambridge/Aachen");
                        });
                    let jet_def =  &mut clustering.jet_def;
                    ui.horizontal(|ui| {
                        ui.add(
                            eframe::egui::DragValue::new(&mut jet_def.radius)
                                .clamp_range(0.0..=6.5)
                                .speed(0.1)
                        );
                        ui.label("Jet radius");
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            eframe::egui::DragValue::new(&mut jet_def.min_pt)
                                .clamp_range(0.0..=f64::MAX)
                        );
                        ui.label("Minimum jet transverse momentum");
                    });
                });
            });
    }

    fn draw_bottom_panel(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // TODO: this doesn't work
            // ui.vertical_centered_justified(|ui| {
            eframe::egui::Grid::new("bottom_panel_grid").show(ui, |ui| {
                if ui.add(eframe::egui::Button::new("<-")).clicked() {
                    self.prev_img(frame)
                }
                let response = ui.add(
                    eframe::egui::TextEdit::singleline(&mut self.ev_idx_str)
                        .text_color_opt(self.ev_idx_str_col)
                );
                if response.changed() {
                    match self.ev_idx_str.parse::<usize>() {
                        Ok(ev_idx) if ev_idx > 0 && ev_idx <= self.events.len() => {
                            self.update_ev(ev_idx - 1, frame.tex_allocator());
                            self.ev_idx_str_col = None;
                        },
                        _ => {
                            self.ev_idx_str_col = Some(eframe::egui::Color32::RED);
                        }
                    };
                }
                ui.label(format!("/{}", self.events.len()));
                if ui.add(eframe::egui::Button::new("->")).clicked() {
                    self.next_img(frame)
                }
            })
            // })
        });
    }

    fn draw_menu_panel(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            eframe::egui::menu::bar(ui, |ui| {
                eframe::egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit\t(Ctrl+q)").clicked() {
                        frame.quit();
                    }
                });
                eframe::egui::menu::menu(ui, "Settings", |ui| {
                    if ui.button("Jet clustering").clicked() {
                        self.clustering_settings_open = true;
                    }
                });
            });
        });
    }

}

impl eframe::epi::App for App {
    fn name(&self) -> &str {
        "evil"
    }

    /// Called by the framework to load old app state (if any).
    fn setup(
        &mut self,
        _ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            *self = epi::get_value(storage, eframe::epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        epi::set_value(storage, eframe::epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        self.handle_keys(ctx.input(), frame);

        self.draw_menu_panel(ctx, frame);

        if !self.events.is_empty() {
            self.draw_bottom_panel(ctx, frame);
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

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            let (width, height) = self.y_phi.size();
            let aspect_ratio = width as f32 / height as f32;
            let mut plot_width = ui.available_width() / 2.;
            let mut img_height = ui.available_height() / 2.;
            match (plot_width / img_height).partial_cmp(&aspect_ratio) {
                Some(Ordering::Less) => img_height = plot_width / aspect_ratio,
                Some(Ordering::Greater) => plot_width = aspect_ratio * img_height,
                _ => {}
            };
            trace!("nominal size: {} x {}", plot_width, img_height);
            ui.columns(2, |col| {
                col[0].vertical_centered(
                    |ui| ui.image(self.y_phi_id, [plot_width, img_height])
                );
                col[1].vertical_centered(
                    |ui| ui.image(self.y_logpt_id, [plot_width, img_height])
                );
            });
            eframe::egui::warn_if_debug_build(ui);
        });

        if self.clustering_settings_open {
            let old = self.clustering;
            self.show_jet_cluster_settings(ctx);
            if self.clustering != old {
                self.plotter.r_jet = self.clustering.jet_def.radius;
                self.update_img(frame.tex_allocator())
            }
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
