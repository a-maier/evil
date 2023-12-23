use std::ops::Range;
use std::path::Path;

use egui::{Context, DragValue, Pos2};
use jetty::PseudoJet;
use lazy_static::lazy_static;
use particle_id::ParticleID;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::event::Event;
use crate::font::{Font, FontFamily, FontStyle};
use crate::particle::Particle;
use crate::plotter::{self, Plotter, PlotResponse, ExportFormat, PlotKind};

lazy_static!{
    static ref FONT_NAMES: Vec<String> = {
        egui::FontDefinitions::default()
            .families
            .values()
            .flatten()
            .cloned()
            .collect()
    };
}

#[derive(Deserialize, Serialize)]
pub(crate) struct YLogPtWin {
    pub(crate) is_open: bool,
    y: Range<f64>,
    logpt: Range<f64>,
}

impl Default for YLogPtWin {
    fn default() -> Self {
        Self {
            is_open: true,
            y: -5.0..5.0,
            logpt: -2.0..3.0,
        }
    }
}

impl YLogPtWin {
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        plotter: &mut Plotter,
        event: &Event,
        jets: &[PseudoJet],
    ) -> Option<PlotResponse> {
        if !self.is_open { return None }

        egui::Window::new("Transverse momentum over rapidity")
            .title_bar(true)
            .min_width(100.)
            .min_height(100.)
            .show(ctx, |ui| {
                plotter.plot_y_logpt(
                    ui,
                    event,
                    jets,
                    self.logpt.clone(),
                )
            }).map(|e| e.inner.flatten()).flatten()

    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct YPhiWin {
    pub(crate) is_open: bool,
    y: Range<f64>,
}

impl Default for YPhiWin {
    fn default() -> Self {
        Self {
            is_open: true,
            y: -5.0..5.0,
        }
    }
}

impl YPhiWin {
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        plotter: &mut Plotter,
        event: &Event,
        jets: &[PseudoJet],
    ) -> Option<PlotResponse> {
        if !self.is_open { return None }

        egui::Window::new("Azimuthal angle over rapidity")
            .title_bar(true)
            .min_width(100.)
            .min_height(100.)
            .show(ctx, |ui| {
                plotter.plot_y_phi(
                    ui,
                    event,
                    &jets,
                )
            }).map(|e| e.inner.flatten()).flatten()
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct DetectorWin {
    pub(crate) is_open: bool,
}

impl Default for DetectorWin {
    fn default() -> Self {
        Self {
            is_open: true,
        }
    }
}

impl DetectorWin {
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        plotter: &Plotter,
        event: &Event,
        jets: &[PseudoJet],
    ) {
        if !self.is_open { return }

        egui::Window::new("DetectorWin")
            .title_bar(true)
            .min_width(100.)
            .min_height(100.)
            .show(ctx, |ui| {
                // let root = EguiBackend::new(ui);
                // let plot = plotter.plot_3d(
                //     event,
                //     &[],
                //     root,
                // ).unwrap();
                // plot.present().unwrap();
            });
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ParticleStyleChoiceWin {
    pub(crate) is_open: bool,
    pub(crate) id: ParticleID,
    pos: Option<Pos2>,
}

impl ParticleStyleChoiceWin {
    pub(crate) fn show(
        &mut self,
        ctx: &Context,
        settings: &mut plotter::Settings
    ) {
        let name = self.id.name().or(self.id.symbol());
        let title = if let Some(name) = name {
            format!("Plot style for {name}")
        } else {
            format!("Plot style for particle id {}", self.id.id())
        };
        let mut is_open = self.is_open;
        let mut win = egui::Window::new(title)
            .open(&mut is_open)
            .title_bar(true);
        if let Some(pos) = self.pos.take() {
            win = win.current_pos(pos);
        }
        win.show(ctx, |ui| {
            let style = settings.get_particle_style_mut(self.id);
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut style.colour);
                ui.label("Marker colour");
            });
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Shape")
                    .selected_text(style.shape.to_string())
                    .show_ui(ui, |ui| {
                        for shape in crate::plotter::MarkerShape::iter() {
                            ui.selectable_value(
                                &mut style.shape,
                                shape,
                                shape.to_string(),
                            );
                        }
                    });
                ui.label("Marker shape");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut style.size));
                ui.label("Marker size");
            });
        });
        self.is_open = is_open;
    }

    pub(crate) fn set_pos(&mut self, pos: Option<Pos2>) {
        self.pos = pos;
    }
}

impl Default for ParticleStyleChoiceWin {
    fn default() -> Self {
        Self {
            is_open: false,
            id: ParticleID::new(0),
            pos: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct PlotterSettings {
    pub is_open: bool,
    pub font: Font,
}

impl PlotterSettings {
    pub(crate) fn changed(
        &mut self,
        ctx: &Context,
    ) -> bool {
        let mut changed = false;

        let mut is_open = self.is_open;
        egui::Window::new("Plotting")
            .open(&mut is_open)
            .title_bar(true)
            .show(ctx, |ui| {
                changed |= self.font_settings_changed(ui);

                ui.separator();

                // changed |= self.colour_settings_changed(ui);
            });
        self.is_open = is_open;
        changed
    }

    fn font_settings_changed(
        &mut self,
        ui: &mut egui::Ui,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(
            |ui| {
                use FontFamily::*;
                let mut family_name = match &self.font.family {
                    Serif => "serif".to_owned(),
                    SansSerif => "sans serif".to_owned(),
                    Monospace => "monospace".to_owned(),
                    Name(s) => s.clone(),
                };
                ui.label("Font");
                let font_changed = egui::ComboBox::from_id_source(0)
                    .width(150.)
                    .selected_text(&family_name)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut family_name, "serif".to_string(), "serif");
                        ui.selectable_value(&mut family_name, "sans serif".to_string(), "sans serif");
                        ui.selectable_value(&mut family_name, "monospace".to_string(), "monospace");
                        for name in FONT_NAMES.iter() {
                            ui.selectable_value(&mut family_name, name.to_string(), name);
                        }
                    }).inner.is_some();

                if font_changed {
                    self.font.family = match family_name.as_str() {
                        "serif"       => Serif,
                        "sans serif"  => SansSerif,
                        "monospace"   => Monospace,
                        s    => Name(s.to_string()),
                    };
                    changed = true;
                }

                use FontStyle::*;
                let style = &mut self.font.style;
                changed |= egui::ComboBox::from_id_source(1)
                    .width(70.)
                    .selected_text(style.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(style, Normal, "Normal");
                        ui.selectable_value(style, Oblique, "Oblique");
                        ui.selectable_value(style, Italic, "Italic");
                        ui.selectable_value(style, Bold, "Bold");
                    }).inner.is_some();

                changed |= ui.add(
                    egui::DragValue::new(&mut self.font.size)
                        .clamp_range(0.1..=f64::MAX)
                ).changed();
            });
        changed
    }

}

#[derive(Debug)]
pub struct ExportDialogue {
    pub format: ExportFormat,
    pub kind: PlotKind,
    pub event_id: usize,
    dialogue: egui_file::FileDialog,
}

impl Default for ExportDialogue {
    fn default() -> Self {
        Self {
            format: ExportFormat::Asymptote, // some default, doesn't matter which
            kind: PlotKind::YLogPt,
            event_id: Default::default(),
            dialogue: egui_file::FileDialog::save_file(None).title("Export event")
        }
    }
 }


impl ExportDialogue {
    pub(crate) fn show(&mut self, ctx: &Context) -> Option<&Path> {
        self.dialogue.show(ctx);
        if self.dialogue.selected() {
            self.dialogue.path()
        } else {
            None
        }
    }

    pub(crate) fn open(&mut self) {
        self.dialogue = egui_file::FileDialog::save_file(None)
            .title("Export event")
            .default_filename(format!(
                "event_{}_{:?}.{}",
                self.event_id,
                self.kind,
                self.format.suffix()
        ));
        self.dialogue.open();
    }
}
