use egui::{Context, ViewportCommand, DragValue};
use jetty::PseudoJet;
use log::{debug, trace};

use crate::clustering::{ClusterSettings, cluster};
use crate::event::Event;
use crate::plotter::Plotter;
use crate::windows::{PlotterSettings, DetectorWin, YPhiWin, YLogPtWin, ParticleStyleChoiceWin};

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
        self.recluster();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu(ctx, ui, frame));

        let dummy = Event::default();
        let event = &self.events.get(self.event_idx).unwrap_or(&dummy);

        if self.plotter_settings.changed(ctx) {
            self.plotter.font = self.plotter_settings.font.clone();
        }

        let selected_logpt = self.y_log_pt.show(ctx, &mut self.plotter, event, &self.jets);
        let selected_phi = self.y_phi.show(ctx, &mut self.plotter, event, &self.jets);
        if let Some(particle) = selected_logpt.or(selected_phi) {
            self.particle_style_choice_win.id = particle.id;
            self.particle_style_choice_win.set_pos(ctx.pointer_interact_pos());
            self.particle_style_choice_win.is_open = true;
        }

        self.detector.show(ctx, &mut self.plotter, event, &self.jets);

        self.particle_style_choice_win.show(ctx, &mut self.plotter.settings);

        if self.clustering.changed(ctx) {
            debug!("Clustering changed to {:?}", self.clustering);
        }

        self.draw_bottom_panel(ctx);
    }

}
