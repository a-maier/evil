use crate::particle::Particle;

#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Event {
    pub out: Vec<Particle>
}
