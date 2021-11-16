use crate::event::Event;

use jetty::{anti_kt_f, cambridge_aachen_f, cluster_if, kt_f, PseudoJet};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum JetAlgorithm {
    AntiKt,
    CambridgeAachen,
    Kt,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct JetDefinition {
    pub algorithm: JetAlgorithm,
    pub radius: f64,
    pub min_pt: f64,
}

impl JetDefinition {
    pub fn cluster_event(&self, event: &Event) -> Vec<PseudoJet> {
        let partons: Vec<PseudoJet> = event.out.iter()
            .filter(|p| p.is_parton())
            .map(|p| [
                p.pt * p.y.cosh(),
                p.pt * p.phi.cos(),
                p.pt * p.phi.sin(),
                p.pt * p.y.sinh(),
            ].into()).collect();
        self.cluster_partons(partons)
    }

    pub fn cluster_partons(&self, partons: Vec<PseudoJet>) -> Vec<PseudoJet> {
        let minpt2 = self.min_pt * self.min_pt;
        let cut = |jet: PseudoJet| jet.pt2() > minpt2;
        let r = self.radius;
        match self.algorithm {
            JetAlgorithm::AntiKt => cluster_if(partons, &anti_kt_f(r), cut),
            JetAlgorithm::Kt => cluster_if(partons, &kt_f(r), cut),
            JetAlgorithm::CambridgeAachen => {
                cluster_if(partons, &cambridge_aachen_f(r), cut)
            }
        }
    }
}
