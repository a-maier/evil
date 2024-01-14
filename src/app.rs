use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;

use event_file_reader::EventFileReader as Reader;
use egui::{Context, ViewportCommand, DragValue, KeyboardShortcut, Modifiers, Vec2};
use jetty::PseudoJet;
use log::{debug, trace, error};
use resvg::tiny_skia::PixmapMut;
use usvg::TreeParsing;

use crate::clustering::{ClusterSettings, cluster};
use crate::event::Event;
use crate::export::export;
use crate::plotter::{Plotter, PlotResponse};
use crate::windows::{YPhiWin, YLogPtWin, ParticleStyleChoiceWin, ExportDialogue, ImportDialogue};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct TemplateApp {
    y_log_pt: YLogPtWin,
    y_phi: YPhiWin,
    plotter: Plotter,
    clustering: ClusterSettings,
    #[serde(skip)]
    particle_style_choice_win: ParticleStyleChoiceWin,
    #[serde(skip)]
    open_file_win: ImportDialogue,
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
    s_file: Option<Sender<String>>, // have to use Option to derive Default
    #[serde(skip)]
    r_ev: Option<Receiver<Event>>, // have to use Option to derive Default
    #[serde(skip)]
    r_msg: Option<Receiver<String>>, // have to use Option to derive Default

    #[serde(skip)]
    plot_3d: Option<egui::TextureHandle>,
}

struct BottomPanelData {
    space: f32,
}

impl Default for BottomPanelData {
    fn default() -> Self {
        Self {
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


        // Disable feathering as it allegedly causes artifacts with egui-plotter
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

        let (s_file, r_file) = channel();
        let (s_ev, r_ev) = channel();
        let (s_msg, r_msg) = channel();
        spawn(move || while let Ok(file) = r_file.recv() {
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
        for file in std::env::args().skip(1) {
            if s_file.send(file).is_err() {
                break;
            }
        }
        res.r_msg = Some(r_msg);
        res.r_ev = Some(r_ev);
        res.s_file = Some(s_file);
        res
    }

    fn menu(
        &mut self,
        ctx: &Context,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        egui::menu::bar(ui, |ui| {
            #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
            ui.menu_button("File", |ui| {
                if ui.button("Open (Ctrl+O)").clicked() {
                    self.open_file_win.open();
                }
                // if ui.button("Quit (Ctrl+Q)").clicked() {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            });
            ui.menu_button("Settings", |ui| {
                if ui.button("Jet clustering").clicked() {
                    self.clustering.is_open = true;
                }
            });
            ui.menu_button("Windows", |ui| {
                ui.checkbox(&mut self.y_log_pt.is_open, "Transverse momentum over rapidity");
                ui.checkbox(&mut self.y_phi.is_open, "Azimuthal angle over rapidity");
            });
            egui::global_dark_light_mode_switch(ui)
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

    fn draw_central_panel(
        &mut self,
        ctx: &Context,
        event: &Event,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.weak(&self.msg);
            let Vec2{x, y}= ui.available_size();
            let [width, height] = [x as usize, y as usize];
            let mut img = String::new();
            self.plotter.plot_3d(event, &self.jets, &mut img, [width, height]).unwrap();
            let tree = usvg::Tree::from_str(&img, &Default::default()).unwrap();
            let tree = resvg::Tree::from_usvg(&tree);
            let mut data = vec![0u8; width  * height * resvg::tiny_skia::BYTES_PER_PIXEL];
            let mut img = PixmapMut::from_bytes(&mut data, width as u32, height as u32).unwrap();
            tree.render(Default::default(), &mut img);
            let img = egui::ColorImage::from_rgba_premultiplied([width, height], img.data_mut());
            let img = egui::ImageData::from(img);
            let texture = self.plot_3d.get_or_insert_with(
                || ctx.load_texture("3D Plot", img.clone(), egui::TextureOptions::default())
            );
            texture.set(img, egui::TextureOptions::default());
            let img = egui::load::SizedTexture::from_handle(&texture);
            ui.image(img)
        });
    }

    fn check_input(&mut self, ctx: &Context) {
        ctx.input_mut(|i| {
            let ctrl_q = KeyboardShortcut::new(Modifiers::CTRL, egui::Key::Q);
            if i.consume_shortcut(&ctrl_q) {
                // TODO: this makes the application hang
                // ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            let ctrl_o = KeyboardShortcut::new(Modifiers::CTRL, egui::Key::O);
            if i.consume_shortcut(&ctrl_o) {
                self.open_file_win.open();
            }
            let right = KeyboardShortcut::new(Modifiers::NONE, egui::Key::ArrowRight);
            if i.consume_shortcut(&right) && !self.events.is_empty() {
                self.event_idx = (self.event_idx + 1) % self.events.len();
            };
            let left = KeyboardShortcut::new(Modifiers::NONE, egui::Key::ArrowLeft);
            if i.consume_shortcut(&left) && !self.events.is_empty() {
                if self.event_idx == 0 {
                    self.event_idx = self.events.len() - 1;
                } else {
                    self.event_idx -= 1;
                }
            };
        })
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
        let event = self.events.get(self.event_idx).unwrap_or(&dummy).clone();

        let response_logpt = self.y_log_pt.show(ctx, &mut self.plotter, &event, &self.jets);
        let response_phi = self.y_phi.show(ctx, &mut self.plotter, &event, &self.jets);
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

        self.particle_style_choice_win.show(ctx, &mut self.plotter.settings);

        if self.clustering.changed(ctx) {
            debug!("Clustering changed to {:?}", self.clustering);
        }

        let kind = self.export_win.kind;
        let format = self.export_win.format;
        if let Some(path) = self.export_win.show(ctx) {
            if let Err(err) = export(
                path,
                &event,
                &self.jets,
                self.plotter.r_jet,
                kind,
                format,
                &self.plotter.settings
            ) {
                error!("{err}");
                self.msg = err.to_string();
            }
        }

        if let Some(path) = self.open_file_win.show(ctx) {
            if let Some(path) = path.to_str() {
                self.events.clear();
                let _ = self.s_file.as_mut().unwrap().send(path.to_owned());
            } else {
                self.msg = format!("Failed to open {path:?}: Cannot convert to UTF-8");
            }
        }

        self.draw_bottom_panel(ctx);

        self.draw_central_panel(ctx, &event);

        self.check_input(ctx);
    }

}
