use std::ops::Range;

use egui_plotter::EguiBackend;
use serde::{Deserialize, Serialize};

use crate::event::Event;
use crate::plotter::Plotter;

const GOLDEN_RATIO: f32 = 1.618033988749895;

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
                let root = EguiBackend::new(ui);
                let plot = plotter.plot_y_logpt(
                    event,
                    &[],
                    self.logpt.clone(),
                    root,
                ).unwrap();
                plot.present().unwrap();
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
                let root = EguiBackend::new(ui);
                let plot = plotter.plot_y_phi(
                    event,
                    &[],
                    root,
                ).unwrap();
                plot.present().unwrap();
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
                let root = EguiBackend::new(ui);
                let plot = plotter.plot_3d(
                    event,
                    &[],
                    root,
                ).unwrap();
                plot.present().unwrap();
            });
    }
}
