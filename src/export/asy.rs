// TODO: opacity
use std::{io::Write, collections::HashSet, borrow::Cow};

use anyhow::Result;
use jetty::PseudoJet;

use crate::{Event, plotter::{PlotKind, self, y_min_max}, particle::Particle};

pub(crate) fn export_asy(
    out: impl Write,
    event: &Event,
    jets: &[PseudoJet],
    r_jet: f64,
    kind: PlotKind,
    settings: &plotter::Settings,
) -> Result<()> {
    use PlotKind::*;
    //todo!("write common code");
    match kind {
        YPhi => export_asy_y_phi(out, event, jets, r_jet, settings),
        YLogPt => export_asy_y_logpt(out, event, jets, r_jet, settings),
    }
}

pub(crate) fn export_asy_y_phi(
    mut out: impl Write,
    event: &Event,
    jets: &[PseudoJet],
    r_jet: f64,
    settings: &plotter::Settings,
) -> Result<()> {
    out.write_all(HEADER)?;
    out.write_all(Y_PHI_HEADER)?;
    let [y_min, y_max] = y_min_max(&event.out);
    writeln!(out, "real xmin = {y_min};
real xmax = {y_max};")?;
    let mut seen = HashSet::new();
    let r = settings.jets.r() as f32 / u8::MAX as f32;
    let g = settings.jets.g() as f32 / u8::MAX as f32;
    let b = settings.jets.b() as f32 / u8::MAX as f32;
    for jet in jets {
        let y = jet.y();
        let phi = jet.phi();
        writeln!(out, "for(int i = -1; i <= 1; ++i) {{
   fill(shift(0, 2*i*pi) * jet_guide({y}, {phi}, {r_jet}), rgb({r},{g},{b}) + opacity(0.2));
}}")?;
    }
    for particle in &event.out {
        let Particle {
            id,
            y,
            phi,
            ..
        } = particle;
        let style = settings.particles.get(&id).unwrap();
        let size = style.size;
        let shape = style.shape;
        let r = style.colour.r() as f32 / u8::MAX as f32;
        let g = style.colour.g() as f32 / u8::MAX as f32;
        let b = style.colour.b() as f32 / u8::MAX as f32;
        if seen.insert(id) {
            let name = id.latex_symbol().map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(id.id().to_string()));
            writeln!(out, "draw(({y:.3}, {phi:.3}), p=invisible, marker=marker(scale({size})*{shape}, FillDraw(fillpen=rgb({r:.3},{g:.3},{b:.3}))), legend=\"${name}$\");")?;
        } else {
            writeln!(out, "draw(({y:.3}, {phi:.3}), p=invisible, marker=marker(scale({size})*{shape}, FillDraw(fillpen=rgb({r:.3},{g:.3},{b:.3}))));")?;
        }
    }
    out.write_all(Y_PHI_AXIS)?;
    Ok(())
}

pub(crate) fn export_asy_y_logpt(
    mut out: impl Write,
    event: &Event,
    jets: &[PseudoJet],
    r_jet: f64,
    settings: &plotter::Settings,
) -> Result<()> {
    let [y_min, y_max] = y_min_max(&event.out);
    writeln!(out, "real xmin = {y_min};
real xmax = {y_max};")?;
    let mut ptmin = f64::MAX;
    let mut ptmax = 0.;
    for particle in &event.out {
        if particle.pt < ptmin {
            ptmin = particle.pt;
        }
        if particle.pt > ptmax {
            ptmax = particle.pt;
        }
    }
    for jet in jets {
        if jet.pt() < ptmin {
            ptmin = jet.pt().into();
        }
        if jet.pt() > ptmax {
            ptmax = jet.pt().into();
        }
    }
    if ptmin > ptmax {
        // some default values to avoid a crash
        ptmin = 1.;
        ptmax = 10.;
    }
    let ptmin = ptmin.powf(0.9);
    let ptmax = ptmax.powf(1.1);

    out.write_all(HEADER)?;
    writeln!(out, "real ptmin = {ptmin:.3};
real ptmax = {ptmax:.3};
scale(Linear,Log);")?;
    let mut seen = HashSet::new();
    let r = settings.jets.r() as f32 / u8::MAX as f32;
    let g = settings.jets.g() as f32 / u8::MAX as f32;
    let b = settings.jets.b() as f32 / u8::MAX as f32;
    for jet in jets {
        let y = jet.y();
        let pt = jet.pt();
        let y_min = y - r_jet;
        let y_max = y + r_jet;
        writeln!(out, "fill(box(({y_min:.3}, log10(ptmin)), ({y_max:.3}, log10({pt:.3}))), rgb({r:.3},{g:.3},{b:.3}) + opacity(0.2));")?;
    }
    for particle in &event.out {
        let logpt = particle.pt.log10();
        let Particle {
            id,
            y,
            ..
        } = particle;
        let style = settings.particles.get(&id).unwrap();
        let size = style.size;
        let shape = style.shape;
        let r = style.colour.r() as f32 / u8::MAX as f32;
        let g = style.colour.g() as f32 / u8::MAX as f32;
        let b = style.colour.b() as f32 / u8::MAX as f32;
        if seen.insert(id) {
            let name = id.latex_symbol().map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(id.id().to_string()));
            writeln!(out, "draw(({y:.3}, {logpt:.3}), p=invisible, marker=marker(scale({size})*{shape}, FillDraw(fillpen=rgb({r:.3},{g:.3},{b:.3}))), legend=\"${name}$\");")?;
        } else {
            writeln!(out, "draw(({y:.3}, {logpt:.3}), p=invisible, marker=marker(scale({size})*{shape}, FillDraw(fillpen=rgb({r:.3},{g:.3},{b:.3}))));")?;
        }
    }
    writeln!(out, r#"xaxis(Label("$y$",0.5),YEquals(ptmin),xmin,xmax,LeftTicks);
xaxis(YEquals(ptmax),xmin,xmax,RightTicks("%"));
yaxis(Label("$p_\perp\,$[GeV]",0.5),XEquals(xmin),ptmin,ptmax,RightTicks);
yaxis(XEquals(xmax),ptmin,ptmax,LeftTicks("%"));
add(legend(invisible),(3.5, log10(ptmin) + 0.9*log10(ptmax/ptmin)));
"#)?;

    Ok(())
}

const HEADER: &[u8] = include_bytes!("header.asy");
const Y_PHI_HEADER: &[u8] = include_bytes!("y_phi.asy");

const Y_PHI_AXIS: &[u8] =  br#"clip((xmin,phimin)--(xmax,phimin)--(xmax,phimax)--(xmin,phimax)--cycle);
xaxis(Label("$y$",0.5),YEquals(phimin),xmin,xmax,LeftTicks);
xaxis(YEquals(phimax),xmin,xmax,RightTicks("%"));
yaxis(Label("$\phi$",0.5),XEquals(xmin),phimin,phimax,RightTicks(phi_label, Step=pi/2,step=pi/8));
yaxis(XEquals(xmax),phimin,phimax,LeftTicks("%",Step=pi/4,step=pi/8));
add(legend(invisible),(3.5,2.6));
"#;
