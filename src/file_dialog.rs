use std::default::Default;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use crate::dir_entries::dir_entries;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileDialog {
    pub open: bool,
    inner: FileDialogInner,
}

#[derive(Clone, Debug, PartialEq)]
struct FileDialogInner {
    path: PathBuf
}

impl Default for FileDialogInner {
    fn default() -> Self {
        Self {
            path: current_dir().unwrap_or_else(
                |_| dirs::home_dir().unwrap() // TODO
            ),
        }
    }
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
        let dir = dir(&self.path).unwrap();
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
            ui.separator();
            let mut path = self.path.to_str()
                .expect("Failed to convert path to str")
                .to_owned();
            ui.horizontal(|ui| {
                ui.label("Name");
                let changed = ui.add(
                    eframe::egui::TextEdit::singleline(&mut path)
                ).changed();
                if changed {
                }
            });
        });


    }
}

fn dir(p: &Path) -> Option<&Path> {
    if p.is_dir() {
        Some(p)
    } else {
        p.parent()
    }
}
