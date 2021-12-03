use std::convert::From;

use crate::particle::Particle;

const OUTGOING_STATUS: i32 = 1;

#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Event {
    pub out: Vec<Particle>
}

impl From<&hepmc2::event::Event> for Event {
    fn from(event: &hepmc2::event::Event) -> Self {
        let out = event.vertices.iter().map(
            |vx| vx.particles_out.iter()
                .filter(|p| p.status == OUTGOING_STATUS)
        )
            .flatten()
            .map(
                |out| {
                    Particle::new(out.id, out.p.0)
                }
            )
            .collect();
        Event{ out }
    }
}

impl From<&lhef::HEPEUP> for Event {
    fn from(event: &lhef::HEPEUP) -> Self {
        let out = event.PUP.iter()
            .enumerate()
            .filter_map(
                |(n, p)| if event.ISTUP[n] == OUTGOING_STATUS {
                    let p = [p[3], p[0], p[1], p[2]];
                    Some(Particle::new(event.IDUP[n], p))
                } else {
                    None
                }
            )
            .collect();
        Event{ out }
    }
}
