use std::ops::Range;

use egui::Context;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::event::Event;
use crate::font::{Font, FontFamily, FontStyle};
use crate::plotter::Plotter;

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
        plotter: &Plotter,
        event: &Event,
    ) {
        if !self.is_open { return }

        egui::Window::new("YLogPtWin")
            .title_bar(true)
            .min_width(100.)
            .min_height(100.)
            .show(ctx, |ui| {
                plotter.plot_y_logpt(
                    ui,
                    event,
                    &[],
                    self.logpt.clone(),
                );
            });
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
        plotter: &Plotter,
        event: &Event,
    ) {
        if !self.is_open { return }

        egui::Window::new("YPhiWin")
            .title_bar(true)
            .min_width(100.)
            .min_height(100.)
            .show(ctx, |ui| {
                plotter.plot_y_phi(
                    ui,
                    event,
                    &[],
                );
            });
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
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct PlotterSettings {
    pub is_open: bool,
    pub font: Font,
    // pub colour: ColourSettings,
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
