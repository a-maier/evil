use egui::{Context, DragValue};
use jetty::{anti_kt_f, cambridge_aachen_f, kt_f, Cluster, PseudoJet};
use particle_id::hadrons::HADRONS;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::Event;

#[derive(
    Display,
    EnumIter,
    Copy,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Deserialize,
    Serialize,
)]
pub enum JetAlgorithm {
    #[default]
    #[strum(to_string = "anti-kt")]
    AntiKt,
    #[strum(to_string = "kt")]
    Kt,
    #[strum(to_string = "Cambridge/Aachen")]
    CambridgeAachen,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct JetDefinition {
    pub algorithm: JetAlgorithm,
    pub radius: f64,
    pub min_pt: f64,
}

pub fn cluster(event: &Event, jet_def: &JetDefinition) -> Vec<PseudoJet> {
    let r = jet_def.radius;
    let out = Vec::from_iter(event.out.iter().filter_map(|p| {
        if p.is_parton() || HADRONS.contains(&p.id) {
            Some(p.p)
        } else {
            None
        }
    }));
    let pt_cut = |p: PseudoJet| p.pt() > jet_def.min_pt;
    match jet_def.algorithm {
        JetAlgorithm::AntiKt => out.cluster_if(anti_kt_f(r), pt_cut),
        JetAlgorithm::CambridgeAachen => {
            out.cluster_if(cambridge_aachen_f(r), pt_cut)
        }
        JetAlgorithm::Kt => out.cluster_if(kt_f(r), pt_cut),
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Default, Debug)]
pub struct ClusterSettings {
    pub is_open: bool,
    pub clustering_enabled: bool,
    pub jet_def: JetDefinition,
}

impl ClusterSettings {
    pub(crate) fn changed(&mut self, ctx: &Context) -> bool {
        let mut changed = false;

        let mut is_open = self.is_open;
        egui::Window::new("Jet clustering")
            .open(&mut is_open)
            .title_bar(true)
            .show(ctx, |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.clustering_enabled,
                        "Enable jet clustering",
                    )
                    .changed();
                ui.add_enabled_ui(self.clustering_enabled, |ui| {
                    let jet_def = &mut self.jet_def;
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_source("Jet algorithm")
                            .selected_text(jet_def.algorithm.to_string())
                            .show_ui(ui, |ui| {
                                for algo in JetAlgorithm::iter() {
                                    changed |= ui
                                        .selectable_value(
                                            &mut jet_def.algorithm,
                                            algo,
                                            algo.to_string(),
                                        )
                                        .changed();
                                }
                            });
                        ui.label("Jet algorithm");
                    });
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                DragValue::new(&mut jet_def.radius).speed(0.01),
                            )
                            .changed();
                        ui.label("Jet radius");
                    });
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(DragValue::new(&mut jet_def.min_pt))
                            .changed();
                        ui.label("Minimum jet transverse momentum");
                    });
                })
            });
        self.is_open = is_open;
        changed
    }
}
