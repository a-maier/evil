use crate::event::Event;
use crate::font::Font;
use crate::particle::{Particle, SpinType, spin_type};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::f64::consts::PI;
use std::ops::{Range, RangeInclusive};

use anyhow::Result;
use egui::{Ui, Stroke, Response};
use egui_plot::{Plot, Legend, Points};
use jetty::PseudoJet;
use num_traits::float::Float;
use particle_id::ParticleID;
use plotters::prelude::*;
use plotters::coord::Shift;
use log::debug;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, Display};

const PHI_SCALE: f64 = PI / 2.;
const PHI_AXIS_MIN: f64 = -2.2;
const PHI_AXIS_MAX: f64 = -PHI_AXIS_MIN;

#[derive(Copy, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ParticleStyle {
    pub colour: egui::Color32,
    pub shape: MarkerShape,
    pub size: f32,
}

impl ParticleStyle {
    pub fn default_for(p: ParticleID) -> Self {
        const DEFAULT_MARKER_SIZE: f32 = 3.;
        Self {
            colour: default_colour_for(p),
            shape: default_shape_for(p),
            size: DEFAULT_MARKER_SIZE
        }
    }
}

fn default_shape_for(p: ParticleID) -> MarkerShape {
    use MarkerShape::*;
    match spin_type(p) {
        SpinType::Boson => Circle,
        SpinType::Fermion => if p.is_anti_particle() {
            Diamond
        } else {
            Square
        },
        _ => Asterisk,
    }
}

fn default_colour_for(p: ParticleID) -> egui::Color32 {
    const CYAN: egui::Color32 = egui::Color32::from_rgb(0, 159, 223);
    const ORANGE: egui::Color32 = egui::Color32::from_rgb(241, 143, 31);
    const MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);
    const PINK: egui::Color32 = egui::Color32::from_rgb(200, 127, 200);
    const VIOLET: egui::Color32 = egui::Color32::from_rgb(82, 0, 127);
    const GREY: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
    const DARK_GREY: egui::Color32 = egui::Color32::from_rgb(80, 80, 80);

    const DEFAULT_COLOR: egui::Color32 = egui::Color32::GRAY;
    use particle_id::sm_elementary_particles as sm;
    match p {
        sm::down =>  CYAN,
        sm::up =>  PINK,
        sm::strange =>  egui::Color32::BLUE,
        sm::charm =>  MAGENTA,
        sm::bottom =>  egui::Color32::DARK_BLUE,
        sm::top =>  VIOLET,
        sm::electron => egui::Color32::YELLOW,
        sm::electron_neutrino => egui::Color32::WHITE,
        sm::muon => ORANGE,
        sm::muon_neutrino => GREY,
        sm::tau => egui::Color32::BROWN,
        sm::tau_neutrino => DARK_GREY,
        sm::gluon => egui::Color32::BLUE,
        sm::photon => egui::Color32::YELLOW,
        sm::Z => egui::Color32::RED,
        sm::W_plus => egui::Color32::DARK_GREEN,
        sm::Higgs => egui::Color32::WHITE,
        _ => DEFAULT_COLOR
    }
}

// egui MarkerShape doesn't derive Deserialize/Serialize
#[derive(Deserialize, Serialize)]
#[derive(Display, EnumIter)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MarkerShape {
    Circle,
    Diamond,
    Square,
    Cross,
    Plus,
    Up,
    Down,
    Left,
    Right,
    Asterisk,
}

impl From<MarkerShape> for egui_plot::MarkerShape {
    fn from(source: MarkerShape) -> Self {
        match source {
            MarkerShape::Circle => egui_plot::MarkerShape::Circle,
            MarkerShape::Diamond => egui_plot::MarkerShape::Diamond,
            MarkerShape::Square => egui_plot::MarkerShape::Square,
            MarkerShape::Cross => egui_plot::MarkerShape::Cross,
            MarkerShape::Plus => egui_plot::MarkerShape::Plus,
            MarkerShape::Up => egui_plot::MarkerShape::Up,
            MarkerShape::Down => egui_plot::MarkerShape::Down,
            MarkerShape::Left => egui_plot::MarkerShape::Left,
            MarkerShape::Right => egui_plot::MarkerShape::Right,
            MarkerShape::Asterisk => egui_plot::MarkerShape::Asterisk,
        }
    }
}

impl From<egui_plot::MarkerShape> for MarkerShape {
    fn from(source: egui_plot::MarkerShape) -> Self {
        match source {
            egui_plot::MarkerShape::Circle => MarkerShape::Circle,
            egui_plot::MarkerShape::Diamond => MarkerShape::Diamond,
            egui_plot::MarkerShape::Square => MarkerShape::Square,
            egui_plot::MarkerShape::Cross => MarkerShape::Cross,
            egui_plot::MarkerShape::Plus => MarkerShape::Plus,
            egui_plot::MarkerShape::Up => MarkerShape::Up,
            egui_plot::MarkerShape::Down => MarkerShape::Down,
            egui_plot::MarkerShape::Left => MarkerShape::Left,
            egui_plot::MarkerShape::Right => MarkerShape::Right,
            egui_plot::MarkerShape::Asterisk => MarkerShape::Asterisk,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Settings {
    // pub frame: egui::Color32,
    // pub background: egui::Color32,
    pub particles: HashMap<ParticleID, ParticleStyle>,
    pub jets: egui::Color32,
}
impl Settings {
    pub fn get_particle_style(&mut self, pid: ParticleID) -> ParticleStyle {
        *self.get_particle_style_mut(pid)
    }

    pub fn get_particle_style_mut(&mut self, pid: ParticleID) -> &mut ParticleStyle {
        self.particles.entry(pid)
            .or_insert_with(|| ParticleStyle::default_for(pid))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // frame: egui::Color32::GRAY,
            // background: egui::Color32::TRANSPARENT,
            particles: HashMap::default(),
            jets: egui::Color32::from_rgba_premultiplied(100, 100, 100, 80),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Projection {
    pub yaw: f64,
    pub pitch: f64,
    pub scale: f64,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Settings3D {
    pub projection: Projection,
}

impl Default for Settings3D {
    fn default() -> Self {
        Self {
            projection: Projection {
                pitch: 0.0,
                yaw: 1.0,
                scale: 1.0,
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct Plotter {
    pub r_jet: f64,

    pub font: Font,

    pub settings: Settings,
    pub settings_3d: Settings3D,
}

impl Plotter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plot_y_phi(
        &mut self,
        ui: &mut Ui,
        event: &Event,
        jets: &[PseudoJet],
    ) -> Option<PlotResponse> {
        use PlotResponse::*;
        let mut response = None;
        let [y_min, y_max] = y_min_max(&event.out);
        Plot::new("y phi plot")
            .include_x(y_min)
            .include_x(y_max)
            .include_y(PHI_AXIS_MIN)
            .include_y(PHI_AXIS_MAX)
            .x_axis_label("y")
            .y_axis_label("φ")
            .y_axis_formatter(phi_tick_label)
            .show_grid([false, false])
            .legend(Legend::default())
            .label_formatter(|name, val|{
                let y = val.x;
                let phi = clamp_phi_coord(val.y) * PHI_SCALE;
                format!("{name}\ny = {y:.2}\nφ = {phi:.2}")
            })
            .show(ui, |ui| {
                for particle in &event.out {
                    self.draw_y_phi(ui, particle);
                }
                for jet in jets {
                    self.draw_y_phi_jet(ui, jet);
                }
                let ui_response = ui.response();
                if ui_response.clicked() {
                    // TODO: better account for zoom levels etc.
                    let click_pos = ui_response.interact_pointer_pos().unwrap();
                    let click_pos = ui.plot_from_screen(click_pos).to_pos2();
                    // TODO: periodicity
                    debug!("Click at {click_pos:?}");
                    let mut closest_dist = f32::MAX;
                    let Some(mut closest) = event.out.first() else {
                        return
                    };
                    for particle in event.out.iter() {
                        let phi_coord = particle.phi / PHI_SCALE;
                        let pos = [particle.y as f32, phi_coord as f32].into();
                        let dist = click_pos.distance_sq(pos);
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest = particle;
                        }
                    }
                    debug!("At distance^2 {closest_dist}: {closest:#?}");
                    const MAX_DIST: f32 = 0.13;
                    if closest_dist < MAX_DIST {
                        response = Some(Selected(*closest));
                    }
                } else {
                    ui_response.clone().context_menu(|ui| {
                        response = export_menu(ui).map(
                            |format| PlotResponse::Export {
                                kind: PlotKind::YPhi,
                                format
                            });
                    });
                }
            });
        response
    }

    pub fn plot_y_logpt(
        &mut self,
        ui: &mut Ui,
        event: &Event,
        jets: &[PseudoJet],
        logpt_range: Range<f64>,
    ) -> Option<PlotResponse> {
        use PlotResponse::*;
        let mut response = None;
        let logpt_start = logpt_range.start - 0.05 * logpt_range.start.abs();
        let logpt_end = logpt_range.end + 0.05 * logpt_range.end.abs();
        let [y_min, y_max] = y_min_max(&event.out);
        Plot::new("y logpt plot")
            .include_x(y_min)
            .include_x(y_max)
            .include_y(logpt_start)
            .include_y(logpt_end)
            .x_axis_label("y")
            .y_axis_label("pT")
            .y_axis_formatter(logpt_tick_label)
            .show_grid([false, false])
            .legend(Legend::default())
            .label_formatter(|name, val|{
                let y = val.x;
                let pt = 10f64.powf(val.y);
                format!("{name}\ny = {y:.2}\npT = {pt:.2}")
            })
            .show(ui, |ui| {
                for jet in jets {
                    self.draw_y_logpt_jet(ui, jet);
                }
                for particle in &event.out {
                    self.draw_y_logpt(ui, particle);
                }
                let ui_response = ui.response();
                if ui_response.clicked() {
                    // TODO: better account for zoom levels etc.
                    let click_pos = ui_response.interact_pointer_pos().unwrap();
                    let click_pos = ui.plot_from_screen(click_pos).to_pos2();
                    debug!("Click at {click_pos:?}");
                    let mut closest_dist = f32::MAX;
                    let Some(mut closest) = event.out.first() else {
                        return
                    };
                    for particle in event.out.iter() {
                        let pt_coord = particle.pt.log10();
                        let pos = [particle.y  as f32, pt_coord as f32].into();
                        let dist = click_pos.distance_sq(pos);
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest = particle;
                        }
                    }
                    debug!("At distance^2 {closest_dist}: {closest:#?}");
                    const MAX_DIST: f32 = 0.13;
                    if closest_dist < MAX_DIST {
                        response = Some(Selected(*closest));
                    }
                } else {
                    ui_response.clone().context_menu(|ui| {
                        response = export_menu(ui).map(
                            |format| PlotResponse::Export {
                                kind: PlotKind::YLogPt,
                                format
                            });
                    });
                }
            });
        response
    }

    pub fn plot_3d<D>(
        &self,
        event: &Event,
        _jets: &[PseudoJet],
        root: D
    ) -> Result<DrawingArea<D, Shift>>
    where
        D: IntoDrawingArea,
        <D as DrawingBackend>::ErrorType: 'static
    {
        todo!()
        // let root = root.into_drawing_area();
        // root.fill(&to_plotters_col(self.colour.background))?;
        // // let root = root.margin(10, 10, 10, 10);
        // let range = (-1.0..1.0).step(0.1);
        // {
        // let mut chart = ChartBuilder::on(&root)
        //     .margin(5)
        //     .set_all_label_area_size(5)
        //     .set_label_area_size(LabelAreaPosition::Left, 110)
        //     .set_label_area_size(LabelAreaPosition::Bottom, 80)
        //     .build_cartesian_3d(range.clone(), range.clone(), range)?;

        // chart.with_projection(|mut pb| {
        //     pb.pitch = self.settings_3d.projection.pitch;
        //     pb.yaw = self.settings_3d.projection.yaw;
        //     pb.scale = self.settings_3d.projection.scale;
        //     pb.into_matrix()
        // });

        // const R: f64 = 0.5;
        // const L: f64 = 0.5;
        // for z in [-L, L] {
        //     chart.draw_series(LineSeries::new(
        //         (0..=100).map(
        //             |t| {
        //                 let phi = 2.*PI*(t as f64) /  100.;
        //                 (R*phi.cos(), R*phi.sin(), z)
        //             }
        //         ),
        //         &to_plotters_col(self.colour.frame),
        //     ))?;
        // }
        // chart.draw_series(LineSeries::new(
        //     (-1..=1).map(|t| {
        //         let t = t as f64;
        //         (R, 0., L*t)
        //     }),
        //     &to_plotters_col(self.colour.frame),
        // ))?;

        // for out in &event.out {
        //     let mut coord = [out.p[1], out.p[2], out.p[3]];
        //     for c in &mut coord {
        //         *c = 2./PI*c.atan()
        //     }

        //     // chart.draw_series(LineSeries::new(
        //     //     (0..=1).map(
        //     //         |t| {
        //     //             let t = t as f64;
        //     //             (t*coord[0], t*coord[1], t*coord[2])
        //     //         }
        //     //     ),
        //     //     &self.get_particle_colour(out.id),
        //     // ))?;
        // }
        // }
        // Ok(root)
    }

    pub(crate) fn get_particle_style(&mut self, pid: ParticleID) -> ParticleStyle {
        self.settings.get_particle_style(pid)
    }

    fn draw_particle_at(
        &mut self,
        ui: &mut egui_plot::PlotUi,
        particle_id: ParticleID,
        centre: [f64; 2]
    ) {
        let ParticleStyle {
            colour,
            shape,
            size,
        } = self.get_particle_style(particle_id);
        let mut pt = Points::new(centre)
            .color(colour)
            .radius(size)
            .shape(shape.into())
            .highlight(true);
        if let Some(name) = particle_id.symbol() {
            pt = pt.name(name);
        }
        ui.points(pt);
    }

    fn draw_y_phi(
        &mut self,
        ui: &mut egui_plot::PlotUi,
        particle: &Particle
    ) {
        let Particle {id, y, phi, ..} = particle;

        debug!("Drawing particle {} at (y, φ) = ({y}, {phi})", id.id());
        let mut phi_min = ui.plot_bounds().min()[1].floor() as i64;
        phi_min -= phi_min % 4;
        let phi_max = ui.plot_bounds().max()[1];
        let mut centre = [*y, phi_min as f64 + *phi / PHI_SCALE];
        while centre[1] < phi_max {
            self.draw_particle_at(ui, *id, centre);
            centre[1] += 4.0
        }
    }

    fn draw_y_phi_jet(
        &self,
        ui: &mut egui_plot::PlotUi,
        jet: &PseudoJet
    ) {
        let y: f64 = jet.rap().into();
        let mut phi: f64 = jet.phi().into();
        if phi > PI {
            phi -= 2.0 * PI;
        }
        debug!("Drawing jet with radius {} at (y, φ) = ({y}, {phi})", self.r_jet);
        let mut phi_min = ui.plot_bounds().min()[1].floor() as i64;
        phi_min -= phi_min % 4;
        let phi_max = ui.plot_bounds().max()[1];
        let mut centre = [y, phi_min as f64 + phi / PHI_SCALE];
        while centre[1] < phi_max {
            self.draw_jet_circle(ui, centre);
            centre[1] += 4.0
        }
    }

    fn draw_jet_circle(
        &self,
        ui: &mut egui_plot::PlotUi,
        centre: [f64; 2]
    ) {
        // TODO: fix shape depending on aspect ratio
        let jet_col = self.settings.jets;
        let r = ui.screen_from_plot([self.r_jet, 0.0].into())[0]
            - ui.screen_from_plot([0.0, 0.0].into())[0];
        let pt = Points::new(centre)
            .color(jet_col)
            .radius(r)
            .highlight(true)
            .name("jet")
            .shape(egui_plot::MarkerShape::Circle);
        ui.points(pt);
    }

    fn draw_y_logpt(
        &mut self,
        ui: &mut egui_plot::PlotUi,
        particle: &Particle
    ) {
        let Particle { id, y, pt, .. } = particle;
        debug!("Drawing particle {} at (y, log(pt)) = ({y}, {})", id.id(), pt.log10());
        let centre = [*y, pt.log10()];
        self.draw_particle_at(ui, *id, centre);
    }

    fn draw_y_logpt_jet(
        &self,
        ui: &mut egui_plot::PlotUi,
        jet: &PseudoJet
    ) {
        debug!("Drawing jet at (y, log(pt)) = ({}, {})", jet.rap(), jet.pt2().log10()/2.);
        let centre = (f64::from(jet.rap()), (jet.pt2().log10()/2.).into());
        let jet_col = self.settings.jets;
        let pt_min = ui.plot_bounds().min()[1];
        let coord = [
            (centre.0 - self.r_jet, pt_min),
            (centre.0 + self.r_jet, centre.1),
        ];
        let rectangle = rectangle(coord)
            .stroke(Stroke::new(0.0, jet_col))
            .fill_color(jet_col);
        ui.polygon(rectangle);
    }

}

pub(crate) fn y_min_max(p: &[Particle]) -> [f64; 2] {
    let y_min = p.iter()
        .map(|p| p.y)
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();
    let y_min = if y_min < 0. {
        1.1 * y_min
    } else {
        0.9 * y_min
    };
    let y_min = f64::min(y_min, -4.5);
    let y_max = p.iter()
        .map(|p| p.y)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();
    let y_max = if y_max < 0. {
        0.9 * y_max
    } else {
        1.1 * y_max
    };
    let y_max = f64::max(y_max, 4.5);
    [y_min, y_max]
}

fn export_menu(ui: &mut Ui) -> Option<ExportFormat> {
    use ExportFormat::*;
    if ui.button("Export to asymptote").clicked() {
        ui.close_menu();
        Some(Asymptote)
    } else {
        None
    }
}

fn rectangle(coord: [(f64, f64); 2]) -> egui_plot::Polygon {
    egui_plot::Polygon::new(vec![
        [coord[0].0, coord[0].1],
        [coord[1].0, coord[0].1],
        [coord[1].0, coord[1].1],
        [coord[0].0, coord[1].1],
    ])
}

fn add<T: std::ops::Add>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 + t2.0, t1.1 + t2.1)
}

fn sub<T: std::ops::Sub>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 - t2.0, t1.1 - t2.1)
}

fn phi_tick_label(
    coord: f64,
    _max_chars: usize,
    _axis_range: &RangeInclusive<f64>
) -> String {
    let c = clamp_phi_coord(coord);
    match c {
        c if c == 2.0 => "π",
        c if c == 1.0 => "π/2",
        c if c == 0.0 => "0",
        c if c == -1.0 => "-π/2",
        c if c == -2.0 => "-π",
        _  => ""
    }.to_string()
}

fn clamp_phi_coord(coord: f64) -> f64 {
    let c = coord % 4.0;
    if c > 2.0 {
        c - 4.0
    } else if c < -2.0 {
        c + 4.0
    } else {
        c
    }
}

fn logpt_tick_label(
    coord: f64,
    _max_chars: usize,
    _axis_range: &RangeInclusive<f64>
) -> String {
    if coord != coord.round() {
        return String::new();
    };
    format!("10{}", fmt_superscript(coord as i64))
}

fn fmt_superscript(mut i: i64) -> String {
    const SUPERSCRIPT_MINUS: char = '⁻';
    const SUPERSCRIPT_DIGITS: &[char] = &['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut res = String::new();
    let neg = match i.cmp(&0) {
        Ordering::Less => {
            i = -i;
            true
        },
        Ordering::Equal => return SUPERSCRIPT_DIGITS[0].to_string(),
        Ordering::Greater => false,
    };
    let mut i = i as usize;
    while i > 0 {
        res.push(SUPERSCRIPT_DIGITS[i % 10]);
        i /= 10;
    }
    if neg {
        res.push(SUPERSCRIPT_MINUS)
    }
    res.chars().rev().collect()
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PlotResponse {
    Export{kind: PlotKind, format: ExportFormat},
    Selected(Particle),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PlotKind {
    YPhi,
    YLogPt,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ExportFormat {
    Asymptote,
}

impl ExportFormat {
    pub(crate) fn suffix(&self) -> &'static str {
        match self {
            ExportFormat::Asymptote => "asy",
        }
    }
}
