use std::env::current_dir;

use crate::dir_entries::dir_entries;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileDialog {
    pub open: bool,
    inner: FileDialogInner,
}

#[derive(Clone, Debug, PartialEq, Default)]
struct FileDialogInner {
}

impl FileDialog {
    pub fn name(&self) -> &'static str {
        "Select File"
    }

    pub fn show(&mut self, ctx: &egui::CtxRef) {
        let window = egui::Window::new(self.name())
            .default_width(600.0)
            .default_height(400.0)
            .vscroll(false)
            .open(&mut self.open);
        window.show(ctx, |ui| self.inner.ui(ui));
    }
}

impl FileDialogInner {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Note that the order we add the panels is very important!

        egui::TopBottomPanel::top(
            "file_dialog_top_panel"
        ).show_inside(ui, |_ui| {});
        egui::SidePanel::left("file_dialog_left_panel")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Left Panel");
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    lorem_ipsum(ui);
                });
            });

        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                let dir = current_dir().unwrap_or_else(
                    |_| dirs::home_dir().unwrap() // TODO
                );
                let entries = dir_entries(dir).unwrap_or_default();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for dir in entries.dirs {
                        ui.add(
                            egui::widgets::Button::new(format!("🗖 {:?}", dir))
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false)
                        );
                    }
                    for file in entries.files {
                        ui.add(
                            egui::widgets::Button::new(format!("🖹 {:?}", file))
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false)
                        );
                    }
                });
        });

        egui::SidePanel::right("file_dialog_right_panel")
            .resizable(false)
            .width_range(8.0..=16.0)
            .show_inside(ui, |_ui| {});
    }
}

fn lorem_ipsum(ui: &mut egui::Ui) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
        |ui| {
            ui.add(egui::Label::new("wot").small().weak());
        },
    );
}
