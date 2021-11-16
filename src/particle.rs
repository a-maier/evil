#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Particle {
    pub id: i32,
    pub y: f64,
    pub phi: f64,
    pub pt: f64,
}

impl Particle {
    pub fn spin_type(&self) -> SpinType {
        spin_type(self.id)
    }

    pub fn is_antiparticle(&self) -> bool {
        self.id < 0
    }

    pub fn name(&self) -> &'static str {
        particle_name(self.id)
    }

    pub fn is_parton(&self) -> bool {
        self.id == 21 || self.id.abs() <= 5
    }
}

pub fn particle_name(id: i32) -> &'static str {
    match id {
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

pub fn spin_type(id: i32) -> SpinType {
    use SpinType::*;
    match id.abs() {
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
