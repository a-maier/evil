use avery::event::Status;

use crate::particle::Particle;

#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Event {
    pub out: Vec<Particle>
}

// TODO: treat errors
impl From<avery::Event> for Event {
    fn from(event: avery::Event) -> Self {
        let out = event.particles
            .into_iter()
            .filter_map(|p| if p.status == Some(Status::Outgoing) {
                Some(Particle::new(p.id.unwrap(), p.p.unwrap()))
            }  else {
                    None
            }).collect();
        Event { out }
    }
}
