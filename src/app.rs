use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;

use event_file_reader::EventFileReader as Reader;
use egui::{Context, ViewportCommand, DragValue};
use jetty::PseudoJet;
use log::{debug, trace, error};

use crate::clustering::{ClusterSettings, cluster};
use crate::event::Event;
use crate::export::export;
use crate::plotter::{Plotter, PlotResponse};
use crate::windows::{PlotterSettings, DetectorWin, YPhiWin, YLogPtWin, ParticleStyleChoiceWin, ExportDialogue};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct TemplateApp {
    y_log_pt: YLogPtWin,
    y_phi: YPhiWin,
    detector: DetectorWin,
    plotter: Plotter,
    plotter_settings: PlotterSettings,
    clustering: ClusterSettings,
    #[serde(skip)]
    particle_style_choice_win: ParticleStyleChoiceWin,
    #[serde(skip)]
    export_win: ExportDialogue,
    #[serde(skip)]
    events: Vec<Event>,
    #[serde(skip)]
    jets: Vec<PseudoJet>,
    #[serde(skip)]
    event_idx: usize,
    #[serde(skip)]
    bottom_panel: BottomPanelData,
    #[serde(skip)]
    msg: String,
    #[serde(skip)]
    r_ev: Option<Receiver<Event>>, // have to use Option to derive Default
    #[serde(skip)]
    r_msg: Option<Receiver<String>>, // have to use Option to derive Default
}

struct BottomPanelData {
    ev_idx_str: String,
    space: f32,
}

impl Default for BottomPanelData {
    fn default() -> Self {
        Self {
            ev_idx_str: "1".to_string(),
            space: 0.
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "DejaVuSans".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/DejaVuSans.ttf")),
        );
        fonts.font_data.insert(
            "DejaVuSansMono".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/DejaVuSansMono.ttf")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "DejaVuSans".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "DejaVuSansMono".to_owned());

        // Tell egui to use these fonts:
        cc.egui_ctx.set_fonts(fonts);


        // Disable feathering as it allegedly causes artifacts
        let context = &cc.egui_ctx;

        context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });

        // Load previous app state (if any).
        let mut res = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        // TODO: load new event files
        // let (s_file, r_file) = channel();
        let (s_ev, r_ev) = channel();
        let (s_msg, r_msg) = channel();
        spawn(move || for file in std::env::args().skip(1) {
              // while let Ok(file) = r_file.recv()
            if s_msg.send(format!("Loading events from {file}")).is_err() {
                break;
            }
            let reader = match Reader::new(&file) {
                Ok(reader) => reader,
                Err(err) => if s_msg.send(format!("Failed to read from {file}: {err}")).is_err() {
                    break;
                } else {
                    continue;
                }
            };
            for event in reader {
                match event {
                    Ok(event) => {
                        if s_ev.send(event.into()).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = s_msg.send(format!("Failed to read from {file}: {err}"));
                    }
                }
            }
            if s_msg.send(String::new()).is_err() {
                break;
            }
        });
        res.r_msg = Some(r_msg);
        res.r_ev = Some(r_ev);
        res
    }

    fn menu(
        &mut self,
        ctx: &Context,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        egui::menu::bar(ui, |ui| {
            #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            });
            ui.menu_button("Settings", |ui| {
                if ui.button("Jet clustering").clicked() {
                    self.clustering.is_open = true;
                }
                if ui.button("Plotting").clicked() {
                    self.plotter_settings.is_open = true;
                }
            });
            ui.menu_button("Windows", |ui| {
                ui.checkbox(&mut self.y_log_pt.is_open, "y-log(pt) plot");
                ui.checkbox(&mut self.y_phi.is_open, "y-φ plot");
                ui.checkbox(&mut self.detector.is_open, "Detector view");
            });
        });
    }

    fn draw_bottom_panel(&mut self, ctx: &Context) {
        eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(self.bottom_panel.space);
                // TODO: use black arrows, but the rightwards one is missing in DejaVu
                let back_button = ui.add_enabled(self.event_idx > 0, egui::Button::new("⇦"));
                if back_button.clicked() {
                    self.event_idx -= 1;
                }

                let mut ev_nr = self.event_idx + 1;
                ui.add(
                    DragValue::new(&mut ev_nr)
                        .clamp_range(1..=self.events.len())
                        .suffix(format!("/{}", self.events.len()))
                );
                self.event_idx = ev_nr - 1;
                let can_forward = 1 + self.event_idx < self.events.len();
                let forward_button = ui.add_enabled(can_forward, egui::Button::new("⇨"));
                if forward_button.clicked() {
                    self.event_idx += 1;
                }
                self.bottom_panel.space = (
                    self.bottom_panel.space + ui.available_width()
                ) / 2.;
            })
        });
    }

    fn recluster(&mut self) {
        if !self.clustering.clustering_enabled {
            self.jets.clear();
            return;
        }
        if let Some(event) = self.events.get(self.event_idx) {
            self.jets = cluster(event, &self.clustering.jet_def);
        } else {
            self.jets.clear()
        }
        self.plotter.r_jet = self.clustering.jet_def.radius;
        trace!("recluster: {:#?}", self.jets);
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        while let Ok(msg) = self.r_msg.as_mut().unwrap().try_recv() {
            self.msg = msg;
        }
        while let Ok(ev) = self.r_ev.as_mut().unwrap().try_recv() {
            self.events.push(ev);
        }
        self.recluster();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu(ctx, ui, frame));

        let dummy = Event::default();
        let event = self.events.get(self.event_idx).unwrap_or(&dummy);

        if self.plotter_settings.changed(ctx) {
            self.plotter.font = self.plotter_settings.font.clone();
        }

        let response_logpt = self.y_log_pt.show(ctx, &mut self.plotter, event, &self.jets);
        let response_phi = self.y_phi.show(ctx, &mut self.plotter, event, &self.jets);
        let response = response_logpt.or(response_phi);
        match response {
            Some(PlotResponse::Selected(particle)) => {
                self.particle_style_choice_win.id = particle.id;
                self.particle_style_choice_win.set_pos(ctx.pointer_interact_pos());
                self.particle_style_choice_win.is_open = true;
            }
            Some(PlotResponse::Export{ kind, format }) => {
                self.export_win.kind = kind;
                self.export_win.format = format;
                self.export_win.event_id = self.event_idx;
                self.export_win.open();
            }
            None => { },
        }

        self.detector.show(ctx, &mut self.plotter, event, &self.jets);

        self.particle_style_choice_win.show(ctx, &mut self.plotter.settings);

        if self.clustering.changed(ctx) {
            debug!("Clustering changed to {:?}", self.clustering);
        }

        let kind = self.export_win.kind;
        let format = self.export_win.format;
        if let Some(path) = self.export_win.show(ctx) {
            if let Err(err) = export(
                path,
                event,
                &self.jets,
                self.plotter.r_jet,
                kind,
                format,
                &self.plotter.settings
            ) {
                error!("{err}"); // TODO: message window
            }
        }

        self.draw_bottom_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.weak(&self.msg);
        });

    }

}
