use particle_id::ParticleID;
use particle_id::sm_elementary_particles::gluon;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Particle {
    pub id: ParticleID,
    pub p: [f64; 4],
    pub y: f64,
    pub phi: f64,
    pub pt: f64,
}

impl Particle {
    pub fn new(id: ParticleID, p: [f64; 4]) -> Self {
        Particle {
            id,
            p,
            y: y(&p),
            phi: phi(&p),
            pt: pt(&p)
        }
    }

    pub fn spin_type(&self) -> SpinType {
        spin_type(self.id)
    }

    pub fn is_antiparticle(&self) -> bool {
        self.id.is_anti_particle()
    }

    pub fn name(&self) -> &'static str {
        particle_name(self.id)
    }

    pub fn is_parton(&self) -> bool {
        self.id == gluon || self.id.id().abs() <= 5
    }
}

pub fn particle_name(id: ParticleID) -> &'static str {
    match id.id() {
        1 => "d",
        2 => "u",
        3 => "s",
        4 => "c",
        5 => "b",
        6 => "t",
        11 => "e¯",
        12 => "νₑ",
        13 => "μ",
        14 => "ν(μ)",
        15 => "τ",
        16 => "ν(τ)",
        21 => "g",
        22 => "γ",
        23 => "Z",
        24 => "W⁺",
        25 => "h",
        -1 => " ̅d",
        -2 => " ̅u",
        -3 => " ̅s",
        -4 => " ̅c",
        -5 => " ̅b",
        -6 => " ̅t",
        -11 => "e⁺",
        -12 => " ̅νₑ",
        -13 => "μ⁺",
        -14 => " ̅ν(μ)",
        -15 => "τ⁺",
        -16 => " ̅ν(τ)",
        -24 => "W¯",
        _ => "N/A",
    }
}

pub fn spin_type(id: ParticleID) -> SpinType {
    use SpinType::*;
    match id.id().abs() {
        1..=16 => Fermion,
        21..=25 => Boson,
        _ => Unknown
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SpinType {
    Boson,
    Fermion,
    Unknown,
}

fn y(p: &[f64; 4]) -> f64 {
    (p[3] / p[0]).atanh()
}

fn phi(p: &[f64; 4]) -> f64 {
    p[2].atan2(p[1])
}

fn pt2(p: &[f64; 4]) -> f64 {
    p[1] * p[1] + p[2] * p[2]
}

fn pt(p: &[f64; 4]) -> f64 {
    pt2(p).sqrt()
}
