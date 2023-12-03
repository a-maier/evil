use egui::{Context, ViewportCommand};
use jetty::PseudoJet;
use log::{debug, trace};

use crate::clustering::{ClusterSettings, cluster};
use crate::event::Event;
use crate::plotter::Plotter;
use crate::windows::{PlotterSettings, DetectorWin, YPhiWin, YLogPtWin};

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
    events: Vec<Event>,
    #[serde(skip)]
    jets: Vec<PseudoJet>,
    #[serde(skip)]
    event_idx: usize,
    #[serde(skip)]
    bottom_panel: BottomPanelData,
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
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        events: Vec<Event>
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Disable feathering as it allegedly causes artifacts
        let context = &cc.egui_ctx;

        context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            let mut res: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            res.events = events;
            return res;
        }

        let mut res = Self{
            events,
            ..Default::default()
        };
        res.recluster();
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
                if ui.add(egui::Button::new("<-")).clicked() {
                    // TODO
                    //self.prev_img(frame)
                }
                let width = 10. * (std::cmp::max(self.events.len(), 100) as f32).log10();

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.bottom_panel.ev_idx_str)
                        .desired_width(width)
                        // TODO
                        //.text_color_opt(self.ev_idx_str_col)
                );
                if response.changed() {
                    match self.bottom_panel.ev_idx_str.parse::<usize>() {
                        Ok(ev_idx) if ev_idx > 0 && ev_idx <= self.events.len() => {
                            // TODO
                            //self.update_ev(ev_idx - 1);
                        },
                        _ => { }
                    };
                }
                ui.label(format!("/{}", self.events.len()));
                if ui.add(eframe::egui::Button::new("->")).clicked() {
                    // self.next_img(frame)
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
        self.recluster();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu(ctx, ui, frame));

        let dummy = Event::default();
        let event = &self.events.get(self.event_idx).unwrap_or(&dummy);

        if self.plotter_settings.changed(ctx) {
            self.plotter.font = self.plotter_settings.font.clone();
        }

        self.y_log_pt.show(ctx, &self.plotter, event, &self.jets);
        self.y_phi.show(ctx, &self.plotter, event, &self.jets);
        self.detector.show(ctx, &self.plotter, event, &self.jets);

        if self.clustering.changed(ctx) {
            debug!("Clustering changed to {:?}", self.clustering);
        }

        self.draw_bottom_panel(ctx);
    }

}
