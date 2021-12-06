use std::cmp::Ordering;
use std::collections::HashMap;
use std::default::Default;
use std::ops::Range;
use std::string::ToString;

use crate::particle::particle_name;
use crate::image::Image;
use crate::event::Event;
use crate::font::{FontFamily, FontStyle};
use crate::plotter::Plotter;
use crate::jets::{JetAlgorithm, JetDefinition};

use font_loader::system_fonts;
use jetty::PseudoJet;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
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
#[derive(Debug, Default, Deserialize, Serialize)]
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
    plot_3d: Image,

    #[serde(skip)]
    plot_3d_id: eframe::egui::TextureId,

    #[serde(skip)]
    first_draw: bool,

    #[serde(skip)]
    ev_idx_str: String,

    #[serde(skip)]
    ev_idx_str_col: Option<eframe::egui::Color32>,

    #[serde(skip)]
    lpt_range: Range<f64>,

    clustering: ClusteringSettings,

    #[serde(skip)]
    clustering_settings_open: bool,

    #[serde(skip)]
    plotter_settings_open: bool,

    #[serde(skip)]
    font_names: Vec<String>,

    bottom_panel_space: f32,
}

impl App {

    pub fn new(events: Vec<Event>) -> Self {

        let mut app = App {
            events,
            ..Default::default()
        };
        app.init();
        app
    }

    // Initialise everything that we don't store
    fn init(&mut self) -> &mut Self {
        // TODO: itertools::minmax is more efficient
        let mut min = f64::MAX;
        let mut max = f64::MIN_POSITIVE;
        for ev in &self.events {
            for p in &ev.out {
                if p.pt < min {
                    min = p.pt
                }
                if p.pt > max {
                    max = p.pt
                }
            }
        }
        self.lpt_range = min.log10() .. max.log10();
        debug!("logpt range: {}..{}", self.lpt_range.start, self.lpt_range.end);

        self.cur_ev_idx = 0;
        self.y_phi_id = Default::default();
        self.y_logpt_id = Default::default();
        self.first_draw = true;
        self.ev_idx_str = "1".to_string();
        self.ev_idx_str_col = None;
        self.clustering_settings_open = false;
        self.plotter_settings_open = false;
        self.font_names = system_fonts::query_all();

        let y_phi;
        let y_logpt;
        let plot_3d;
        if let Some(event) = self.events.first() {
            let jets = self.cluster_jets(event);
            y_phi = self.plotter.plot_y_phi(event, &jets);
            y_logpt = self.plotter.plot_y_logpt(event, &jets, self.lpt_range.clone());
            plot_3d = self.plotter.plot_3d(event, &jets);
        } else {
            let ev = Event::default();
            y_phi = self.plotter.plot_y_phi(&ev, &[]);
            y_logpt = self.plotter.plot_y_logpt(&ev, &[], self.lpt_range.clone());
            plot_3d = self.plotter.plot_3d(&ev, &[]);
        };

        self.y_phi = Image::new(y_phi.unwrap(), (640, 480));
        self.y_logpt = Image::new(y_logpt.unwrap(), (640, 480));
        self.plot_3d = Image::new(plot_3d.unwrap(), (640, 480));

        self
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
        self.ev_idx_str_col = None;
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
            .alloc_srgba_premultiplied(self.y_phi.size(), self.y_phi.pixels());

        let svg = self.plotter.plot_y_logpt(
            event,
            &jets,
            self.lpt_range.clone(),
        ).unwrap();
        self.y_logpt = Image::new(svg, (1280, 960));
        allocator.free(self.y_logpt_id);
        self.y_logpt_id = allocator
            .alloc_srgba_premultiplied(self.y_logpt.size(), self.y_logpt.pixels());


        let svg = self.plotter.plot_3d(
            event,
            &jets,
        ).unwrap();
        self.plot_3d = Image::new(svg, (1280, 960));
        allocator.free(self.plot_3d_id);
        self.plot_3d_id = allocator
            .alloc_srgba_premultiplied(self.plot_3d.size(), self.plot_3d.pixels());
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
            } else if *pressed && modifiers.is_none() {
                match key {
                    Key::ArrowRight | Key::PageDown => self.next_img(frame),
                    Key::ArrowLeft | Key::PageUp => self.prev_img(frame),
                    _ => {}
                }
            }
        }
    }

    fn jet_cluster_settings_changed(
        &mut self,
        ctx: &eframe::egui::CtxRef
    ) -> bool {
        let mut changed = false;
        let clustering = &mut self.clustering;
        eframe::egui::Window::new("Jet clustering")
            .open(&mut self.clustering_settings_open)
            .show(ctx, |ui| {
                changed |= ui.checkbox(
                    &mut clustering.enable,
                    "Enable jet clustering"
                ).changed();
                ui.scope(|ui| {
                    ui.set_enabled(clustering.enable);

                    use JetAlgorithm::*;
                    let algo = &mut clustering.jet_def.algorithm;
                    changed |= eframe::egui::ComboBox::from_label( "Jet algorithm")
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
                        }).inner.is_some();
                    let jet_def =  &mut clustering.jet_def;
                    ui.horizontal(|ui| {
                        changed |= ui.add(
                            eframe::egui::DragValue::new(&mut jet_def.radius)
                                .clamp_range(0.0..=6.5)
                                .speed(0.1)
                        ).changed();
                        ui.label("Jet radius");
                    });

                    ui.horizontal(|ui| {
                        changed |= ui.add(
                            eframe::egui::DragValue::new(&mut jet_def.min_pt)
                                .clamp_range(0.0..=f64::MAX)
                        ).changed();
                        ui.label("Minimum jet transverse momentum");
                    });
                });
            });
        changed
    }

    fn plotter_settings_changed(&mut self, ctx: &eframe::egui::CtxRef) -> bool {
        let font_names = &self.font_names;
        let plotter = &mut self.plotter;
        let mut changed = false;
        eframe::egui::Window::new("Plotting")
            .open(&mut self.plotter_settings_open)
            .show(ctx, |ui| {
                changed |= font_settings_changed(ui, plotter, font_names);

                ui.separator();

                changed |= colour_settings_changed(ui, &mut plotter.colour);
            });
        changed
    }

    fn draw_bottom_panel(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(self.bottom_panel_space);
                if ui.add(eframe::egui::Button::new("<-")).clicked() {
                    self.prev_img(frame)
                }
                let width = 10. * (std::cmp::max(self.events.len(), 10) as f32).log10();
                let response = ui.add(
                    eframe::egui::TextEdit::singleline(&mut self.ev_idx_str)
                        .desired_width(width)
                        .text_color_opt(self.ev_idx_str_col)
                );
                if response.changed() {
                    match self.ev_idx_str.parse::<usize>() {
                        Ok(ev_idx) if ev_idx > 0 && ev_idx <= self.events.len() => {
                            self.update_ev(ev_idx - 1, frame.tex_allocator());
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
                self.bottom_panel_space = (
                    self.bottom_panel_space + ui.available_width()
                ) / 2.;
            })
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
                    if ui.button("Plotting").clicked() {
                        self.plotter_settings_open = true;
                    }
                });
                dark_light_mode_switch(ui);
            });
        });
    }

}

fn font_settings_changed(
    ui: &mut egui::Ui,
    plotter: &mut Plotter,
    font_names: &[String]
) -> bool {
    let mut changed = false;
    ui.horizontal(
        |ui| {
            use FontFamily::*;
            let mut family_name = match &plotter.font.family {
                Serif => "serif".to_owned(),
                SansSerif => "sans serif".to_owned(),
                Monospace => "monospace".to_owned(),
                Name(s) => s.clone(),
            };
            ui.label("Font");
            let font_changed = eframe::egui::ComboBox::from_id_source(0)
                .width(150.)
                .selected_text(&family_name)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut family_name, "serif".to_string(), "serif");
                    ui.selectable_value(&mut family_name, "sans serif".to_string(), "sans serif");
                    ui.selectable_value(&mut family_name, "monospace".to_string(), "monospace");
                    for name in font_names {
                        ui.selectable_value(&mut family_name, name.to_string(), name);
                    }
                }).inner.is_some();

            if font_changed {
                plotter.font.family = match family_name.as_str() {
                    "serif"       => Serif,
                    "sans serif"  => SansSerif,
                    "monospace"   => Monospace,
                    s    => Name(s.to_string()),
                };
                changed = true;
            }

            use FontStyle::*;
            let style = &mut plotter.font.style;
            changed |= eframe::egui::ComboBox::from_id_source(1)
                .width(70.)
                .selected_text(style.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(style, Normal, "Normal");
                    ui.selectable_value(style, Oblique, "Oblique");
                    ui.selectable_value(style, Italic, "Italic");
                    ui.selectable_value(style, Bold, "Bold");
                }).inner.is_some();

            changed |= ui.add(
                eframe::egui::DragValue::new(&mut plotter.font.size)
                    .clamp_range(0.0..=f64::MAX)
            ).changed();
        });
    changed
}

fn particle_colour_changed(
    ui: &mut egui::Ui,
    col_mapping: &mut HashMap<i32, egui::Color32>,
    pid: i32,
) -> bool {
    let mut changed = false;
    ui.horizontal(
        |ui| {
            let mut col = *col_mapping.get(&pid).unwrap();
            changed = ui.color_edit_button_srgba(&mut col).changed();
            if changed {
                *col_mapping.get_mut(&pid).unwrap() = col;
            }
            ui.label(particle_name(pid));
        }
    );
    changed
}

fn colour_settings_changed(
    ui: &mut egui::Ui,
    colours: &mut crate::plotter::ColourSettings,
) -> bool {
    let mut changed = false;
    ui.label("Colours");
    ui.horizontal(
        |ui| {
            ui.label("Frame");
            changed |= ui.color_edit_button_srgba(&mut colours.frame).changed();
            ui.label("Background");
            changed |= ui.color_edit_button_srgba(&mut colours.background).changed();
            ui.label("Jets");
            changed |= ui.color_edit_button_srgba(&mut colours.jets).changed();
        }
    );
    let particle_cols = &mut colours.particles;
    ui.columns(
        4, |col| {
            col[0].with_layout(
                egui::Layout::top_down_justified(egui::Align::RIGHT),
                |ui| {
                    changed |= particle_colour_changed(ui, particle_cols, 2);
                    changed |= particle_colour_changed(ui, particle_cols, 1);
                    changed |= particle_colour_changed(ui, particle_cols, 12);
                    changed |= particle_colour_changed(ui, particle_cols, 11);
                }
            );
            col[1].with_layout(
                egui::Layout::top_down_justified(egui::Align::RIGHT),
                |ui| {
                    changed |= particle_colour_changed(ui, particle_cols, 4);
                    changed |= particle_colour_changed(ui, particle_cols, 3);
                    changed |= particle_colour_changed(ui, particle_cols, 14);
                    changed |= particle_colour_changed(ui, particle_cols, 13);
                }
            );
            col[2].with_layout(
                egui::Layout::top_down_justified(egui::Align::RIGHT),
                |ui| {
                    changed |= particle_colour_changed(ui, particle_cols, 6);
                    changed |= particle_colour_changed(ui, particle_cols, 5);
                    changed |= particle_colour_changed(ui, particle_cols, 16);
                    changed |= particle_colour_changed(ui, particle_cols, 15);
                }
            );
            col[3].with_layout(
                egui::Layout::top_down_justified(egui::Align::RIGHT),
                |ui| {
                    changed |= particle_colour_changed(ui, particle_cols, 21);
                    changed |= particle_colour_changed(ui, particle_cols, 22);
                    changed |= particle_colour_changed(ui, particle_cols, 23);
                    changed |= particle_colour_changed(ui, particle_cols, 24);
                    changed |= particle_colour_changed(ui, particle_cols, 25);
                }
            );
        }
    );

    changed
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
            let events = std::mem::take(&mut self.events);
            *self = epi::get_value(storage, eframe::epi::APP_KEY).unwrap_or_default();
            self.events = events;
            self.init();
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        epi::set_value(storage, eframe::epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        //debug!("{:#?}", self);
        self.handle_keys(ctx.input(), frame);

        self.draw_menu_panel(ctx, frame);


        if !self.events.is_empty() {
            self.draw_bottom_panel(ctx, frame);
        }

        if self.first_draw {
            self.y_phi_id = frame
                .tex_allocator()
                .alloc_srgba_premultiplied((640, 480), self.y_phi.pixels());
            self.y_logpt_id = frame
                .tex_allocator()
                .alloc_srgba_premultiplied((640, 480), self.y_logpt.pixels());
            self.plot_3d_id = frame
                .tex_allocator()
                .alloc_srgba_premultiplied((640, 480), self.plot_3d.pixels());
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
                    |ui| {
                        ui.image(self.y_phi_id, [plot_width, img_height]);
                        ui.image(self.y_logpt_id, [plot_width, img_height])
                    }
                );
                col[1].vertical_centered(
                    |ui| {
                        ui.add_space(img_height / 2.);
                        ui.image(self.plot_3d_id, [plot_width, img_height])
                    }
                );
            });

            eframe::egui::warn_if_debug_build(ui);
        });

        if self.clustering_settings_open && self.jet_cluster_settings_changed(ctx){
            self.plotter.r_jet = self.clustering.jet_def.radius;
            self.update_img(frame.tex_allocator())
        }

        if self.plotter_settings_open && self.plotter_settings_changed(ctx) {
            self.update_img(frame.tex_allocator())
        }

    }
}

// taken from egui demo app
fn dark_light_mode_switch(ui: &mut egui::Ui) {
    let style: egui::Style = (*ui.ctx().style()).clone();
    let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
    if let Some(visuals) = new_visuals {
        ui.ctx().set_visuals(visuals);
    }
}
