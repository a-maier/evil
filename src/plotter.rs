use crate::event::Event;
use crate::font::Font;
use crate::particle::{Particle, particle_name, SpinType, spin_type};

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::iter::{self, FromIterator};
use std::f64::consts::PI;
use std::ops::{Range, RangeInclusive};

use anyhow::Result;
use egui::{Ui, Stroke};
use egui_plot::{Plot, PlotPoints, Legend, Points};
use jetty::PseudoJet;
use lazy_static::lazy_static;
use num_traits::float::Float;
use particle_id::ParticleID;
use plotters::prelude::*;
use plotters::coord::Shift;
use plotters::style::{
    RGBAColor,
    text_anchor::{HPos, Pos, VPos}
};
use log::debug;
use serde::{Deserialize, Serialize};

const PHI_MIN: f64 = -PI;
const PHI_MAX: f64 = PI;
const PHI_SCALE: f64 = PI / 2.;
const PHI_AXIS_MIN: f64 = -2.2;
const PHI_AXIS_MAX: f64 = -PHI_AXIS_MIN;
const Y_MIN: f64 = -5.;
const Y_MAX: f64 = 5.;
const Y_AXIS_MIN: f64 = 1.1 * Y_MIN;
const Y_AXIS_MAX: f64 = 1.1 * Y_MAX;
const MAJOR_TICK_SIZE: i32 = 5;
const MINOR_TICK_SIZE: i32 = MAJOR_TICK_SIZE / 2;
const N_MAJOR_PHI_TICKS: usize = 5;
const N_MINOR_PHI_TICKS: usize = 3;

const X_AXIS_LABEL_OFFSET: i32 = 25;
const Y_AXIS_LABEL_OFFSET: i32 = 30;
const TICK_LABEL_OFFSET: i32 = 7;

const BOX_CORNER: (f64, f64) = (0.05, 0.05);
const CIRCLE_SIZE: i32 = 3;

const LEGEND_X_POS: f64 = 4.;
const LEGEND_START_REL: f64 = 0.95;
const LEGEND_REL_STEP: f64 = 0.05;

pub const REL_SUB_FONT_SIZE: f64 = 0.6;

lazy_static!{
    static ref CYAN: egui::Color32 = egui::Color32::from_rgb(0, 159, 223);
    static ref ORANGE: egui::Color32 = egui::Color32::from_rgb(241, 143, 31);
    static ref MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);
    static ref PINK: egui::Color32 = egui::Color32::from_rgb(200, 127, 200);
    static ref VIOLET: egui::Color32 = egui::Color32::from_rgb(82, 0, 127);
    static ref GREY: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
    static ref DARK_GREY: egui::Color32 = egui::Color32::from_rgb(80, 80, 80);

    static ref MAJOR_PHI_TICK_POS: [f64; N_MAJOR_PHI_TICKS] = {
        let mut arr = [0.0; N_MAJOR_PHI_TICKS];
        for (n, e) in arr.iter_mut().enumerate() {
            *e = PHI_MIN + n as f64 / (N_MAJOR_PHI_TICKS - 1) as f64 * (PHI_MAX - PHI_MIN);
        }
        arr
    };
    static ref MINOR_PHI_TICK_POS: [f64; (N_MAJOR_PHI_TICKS - 1) * N_MINOR_PHI_TICKS] = {
        let mut pos = Vec::with_capacity((N_MAJOR_PHI_TICKS - 1) * N_MINOR_PHI_TICKS);
        for major_tick in 0..(N_MAJOR_PHI_TICKS - 1) {
            let stepsize = (
                MAJOR_PHI_TICK_POS[major_tick + 1] - MAJOR_PHI_TICK_POS[major_tick]
            ) / (N_MINOR_PHI_TICKS + 1) as f64;
            for minor_tick in 1..=N_MINOR_PHI_TICKS {
                pos.push(MAJOR_PHI_TICK_POS[major_tick] + minor_tick as f64 * stepsize)
            }
        }
        let mut res = [0.0; (N_MAJOR_PHI_TICKS - 1) * N_MINOR_PHI_TICKS];
        res.copy_from_slice(&pos);
        res
    };

    static ref MAJOR_Y_TICK_POS: Vec<f64> =
        iter::once(Y_MIN).chain(
            (Y_MIN as i32 ..=(Y_MAX as i32)).map(|i| y_to_coord(i as f64))
        ).chain(iter::once(Y_MAX)).collect();
    static ref MINOR_Y_TICK_POS: Vec<f64> = {
        let mut pos = vec![-4.5, -3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5, 4.5];
        for y in &mut pos {
            *y = y_to_coord(*y);
        }
        pos
    };
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ColourSettings {
    pub frame: egui::Color32,
    pub background: egui::Color32,
    pub particles: HashMap<i32, egui::Color32>,
    pub jets: egui::Color32,
}

impl Default for ColourSettings {
    fn default() -> Self {
        Self {
            frame: egui::Color32::GRAY,
            background: egui::Color32::TRANSPARENT,
            particles: HashMap::from_iter([
                (1,  *CYAN),
                (2,  *PINK),
                (3,  egui::Color32::BLUE),
                (4,  *MAGENTA),
                (5,  egui::Color32::DARK_BLUE),
                (6,  *VIOLET),
                (11, egui::Color32::YELLOW),
                (12, egui::Color32::WHITE),
                (13, *ORANGE),
                (14, *GREY),
                (15, egui::Color32::BROWN),
                (16, *DARK_GREY),
                (21, egui::Color32::BLUE),
                (22, egui::Color32::YELLOW),
                (23, egui::Color32::RED),
                (24, egui::Color32::DARK_GREEN),
                (25, egui::Color32::WHITE),
            ]),
            jets: egui::Color32::from_rgba_premultiplied(130, 130, 130, 80),
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

    pub colour: ColourSettings,
    pub settings_3d: Settings3D,
}

impl Plotter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plot_y_phi(
        &self,
        ui: &mut Ui,
        event: &Event,
        jets: &[PseudoJet],
    ) {
        Plot::new("y phi plot")
            .include_x(Y_AXIS_MIN)
            .include_x(Y_AXIS_MAX)
            .include_y(PHI_AXIS_MIN)
            .include_y(PHI_AXIS_MAX)
            .x_axis_label("y")
            .y_axis_label("φ")
            .x_axis_formatter(rap_tick_label)
            .y_axis_formatter(phi_tick_label)
            .show_grid([false, false])
            .legend(Legend::default())
            .label_formatter(|_name, val|{
                let phi_coord = clamp_phi_coord(val.y);
                format!("y = {:.2}\nφ = {:.2}", coord_to_y(val.x), phi_coord * PHI_SCALE)
            })
            .show(ui, |ui| {
                for particle in &event.out {
                    self.draw_y_phi(ui, particle);
                }
                for jet in jets {
                    self.draw_y_phi_jet(ui, jet);
                }
            });
    }

    pub fn plot_y_logpt(
        &self,
        ui: &mut Ui,
        event: &Event,
        jets: &[PseudoJet],
        logpt_range: Range<f64>,
    ) {
        let logpt_start = logpt_range.start - 0.05 * logpt_range.start.abs();
        let logpt_end = logpt_range.end + 0.05 * logpt_range.end.abs();
        Plot::new("y logpt plot")
            .include_x(Y_AXIS_MIN)
            .include_x(Y_AXIS_MAX)
            .include_y(logpt_start)
            .include_y(logpt_end)
            .x_axis_label("y")
            .y_axis_label("pT")
            .x_axis_formatter(rap_tick_label)
            .y_axis_formatter(logpt_tick_label)
            .show_grid([false, false])
            .legend(Legend::default())
            .label_formatter(|_name, val|{
                // TODO: what is name?
                format!("y = {:.2}\npT = {:.2}", coord_to_y(val.x), 10f64.powf(val.y))
            })
            .show(ui, |ui| {
                for particle in &event.out {
                    self.draw_y_logpt(ui, particle);
                }
                for jet in jets {
                    self.draw_y_logpt_jet(ui, jet);
                }
            });
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

    // TODO: return Result
    fn draw_x_tick<DB, X, Y, S>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: f64,
        size: i32,
        align: VerticalPosition,
        style: S,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
        S: Into<ShapeStyle>,
    {
        let y_start = match align {
            VerticalPosition::Bottom => chart.y_range().start,
            VerticalPosition::Top => chart.y_range().end,
        };
        let pos_start = chart.backend_coord(&(pos, y_start));
        let pos_end = match align {
            VerticalPosition::Bottom => (pos_start.0, pos_start.1 - size),
            VerticalPosition::Top => (pos_start.0, pos_start.1 + size),
        };
        root.draw(
            &PathElement::new([pos_start, pos_end], style)
        ).unwrap();
    }

    // TODO: return Result
    fn draw_y_tick<DB, X, Y, S>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: f64,
        size: i32,
        align: HorizontalPosition,
        style: S,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
        S: Into<ShapeStyle>,
    {
        let x_start = match align {
            HorizontalPosition::Left => chart.x_range().start,
            HorizontalPosition::Right => chart.x_range().end,
        };
        let pos_start = chart.backend_coord(&(x_start, pos));
        let pos_end = match align {
            HorizontalPosition::Left => (pos_start.0 + size, pos_start.1),
            HorizontalPosition::Right => (pos_start.0 - size, pos_start.1),
        };
        root.draw(
            &PathElement::new([pos_start, pos_end], style)
        ).unwrap();
    }

    fn draw_tick<DB, X, Y, S>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: f64,
        size: i32,
        align: Position,
        style: S,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
        S: Into<ShapeStyle>,
    {
        use HorizontalPosition::{Left, Right};
        use VerticalPosition::{Bottom, Top};
        match align {
            Position::Left => self.draw_y_tick(root, chart, pos, size, Left, style),
            Position::Right => self.draw_y_tick(root, chart, pos, size, Right, style),
            Position::Bottom => self.draw_x_tick(root, chart, pos, size, Bottom, style),
            Position::Top => self.draw_x_tick(root, chart, pos, size, Top, style),
        }
    }

    fn draw_ticks<DB, X, Y, P, I>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: P,
        size: i32,
        align: Position,
    )
    where
        DB: DrawingBackend,
        P: IntoIterator<Item=I>,
        I: Borrow<f64>,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        todo!()
        // let col: RGBAColor = to_plotters_col(self.colour.frame);
        // let style: ShapeStyle = col.into();
        // for pos in pos.into_iter() {
        //     self.draw_tick(root, chart, *pos.borrow(), size, align, style.clone());
        // }
    }

    fn draw_phi_ticks<DB: DrawingBackend, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>
    )
    where
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        use Position::{Left, Right};
        for align in [Left, Right] {
            self.draw_ticks(root, chart, MAJOR_PHI_TICK_POS.iter(), MAJOR_TICK_SIZE, align);
            self.draw_ticks(root, chart, MINOR_PHI_TICK_POS.iter(), MINOR_TICK_SIZE, align);
        }
    }

    fn draw_logpt_ticks<DB: DrawingBackend, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        range: Range<i64>
    )
    where
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        use Position::{Left, Right};
        for align in [Left, Right] {
            let major_tick_pos = range.clone().into_iter().map(|logpt| logpt as f64);
            self.draw_ticks(root, chart, major_tick_pos, MAJOR_TICK_SIZE, align);
            let mut range = range.clone();
            range.end += 1;
            let y_range = chart.y_range();
            let minor_tick_pos = range.into_iter().map(
                |pos| (1..10).map(move |step| pos as f64 + (step as f64).log10() - 1.)
            ).flatten()
                .filter(|pos| y_range.contains(pos));
            self.draw_ticks(root, chart, minor_tick_pos, MINOR_TICK_SIZE, align);
        }
    }

    fn draw_rap_ticks<DB: DrawingBackend, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>
    )
    where
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        use Position::{Bottom, Top};
        for align in [Bottom, Top] {
            self.draw_ticks(root, chart, MAJOR_Y_TICK_POS.iter(), MAJOR_TICK_SIZE, align);
            self.draw_ticks(root, chart, MINOR_Y_TICK_POS.iter(), MINOR_TICK_SIZE, align);
        }
    }

    fn phi_tick_label<S: AsRef<str>, DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        text: S,
        pos: f64,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        self.draw_text(
            root,
            chart,
            text,
            (Y_AXIS_MIN , pos),
            (- TICK_LABEL_OFFSET, 0),
            Pos{ h_pos: HPos::Right, v_pos: VPos::Center }
        );
    }

    fn rap_tick_label<S: AsRef<str>, DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        text: S,
        pos: f64,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        let ymin = chart.y_range().start;
        self.draw_text(
            root,
            chart,
            text,
            (pos, ymin),
            (0, TICK_LABEL_OFFSET),
            Pos{ h_pos: HPos::Center, v_pos: VPos::Top }
        );
    }

    fn draw_text<S: AsRef<str>, DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        text: S,
        pos: (f64, f64),
        offset: (i32, i32),
        align: Pos,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        todo!()
        // let col = to_plotters_col(self.colour.frame);
        // let pos = add(chart.backend_coord(&pos), offset);
        // root.draw_text(
        //     text.as_ref(),
        //     &TextStyle {
        //         font: (&self.font).into(),
        //         color: col.to_backend_color(),
        //         pos: align
        //     },
        //     pos,
        // ).unwrap()
    }

    fn dress_phi_axis<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        self.draw_phi_ticks(root, chart);
        self.phi_tick_label(root, chart, "π", PI);
        self.phi_tick_label(root, chart, "π/2", PI / 2.);
        self.phi_tick_label(root, chart, "0", 0.);
        self.phi_tick_label(root, chart, "-π/2", -PI / 2.);
        self.phi_tick_label(root, chart, "-π", -PI);
        self.draw_text(
            root, chart,
            "φ",
            (Y_AXIS_MIN, 0.0),
            (- Y_AXIS_LABEL_OFFSET, 0),
            Pos{ h_pos: HPos::Right, v_pos: VPos::Center }
        );
    }

    fn dress_logpt_axis<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        mut range: Range<i64>
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        todo!()
        // range.end += 1;
        // self.draw_logpt_ticks(root, chart, range.clone());
        // let col = to_plotters_col(self.colour.frame);
        // let align = Pos{ h_pos: HPos::Right, v_pos: VPos::Center };
        // let style = TextStyle {
        //     font: (&self.font).into(),
        //     color: col.to_backend_color(),
        //     pos: align
        // };
        // let sup_style = TextStyle {
        //     font: style.font.resize(REL_SUB_FONT_SIZE * self.font.size),
        //     color: style.color,
        //     pos: align
        // };
        // // TODO: how to calculate this properly?
        // let s = self.font.size as i32;
        // for logpt in range {
        //     let sup_pos = if logpt < 0 {
        //         (3 * s / 5, -(s / 4))
        //     } else {
        //         (s / 5, -(s / 4))
        //     };
        //     let pos = (Y_AXIS_MIN, logpt as f64);
        //     let offset = ( - TICK_LABEL_OFFSET, 0);
        //     let mut pos = chart.backend_coord(&pos);
        //     pos.0 += offset.0;
        //     pos.1 += offset.1;
        //     root.draw(
        //         &(
        //             EmptyElement::at(pos)
        //                 + Text::new("10", (-s / 4, 0), &style)
        //                 + Text::new(logpt.to_string(), sup_pos, &sup_style)
        //         )
        //     ).unwrap()
        // }
        // let y_range = chart.y_range();

        // let pos = (Y_AXIS_MIN, (y_range.start + y_range.end) / 2.);
        // let offset = ( - Y_AXIS_LABEL_OFFSET, 0);
        // let pos = add(chart.backend_coord(&pos), offset);
        // root.draw(
        //     &(
        //         EmptyElement::at(pos)
        //             + Text::new("p", (-s / 4, 0), &style)
        //             + Text::new("T", (s / 10, s / 4), &sup_style)
        //     )
        // ).unwrap()
    }

    fn dress_rap_axis<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        self.draw_rap_ticks(root, chart);
        for y in (Y_MIN as i32)..=(Y_MAX as i32) {
            self.rap_tick_label(root, chart, &format!("{}", y), y_to_coord(y as f64));
        }
        // fudge slightly to avoid label collision
        self.rap_tick_label(root, chart, "-∞", Y_MIN - 0.1);
        self.rap_tick_label(root, chart, "∞", Y_MAX);
        let ymin = chart.y_range().start;
        self.draw_text(
            root, chart,
            "y",
            (0., ymin),
            (0, X_AXIS_LABEL_OFFSET),
            Pos{ h_pos: HPos::Center, v_pos: VPos::Top }
        );
    }


    fn get_particle_colour(&self, pid: ParticleID) -> egui::Color32 {
        *self.colour.particles.get(
            &pid.id().abs()
        ).unwrap_or(&egui::Color32::GRAY)
    }

    fn draw_particle_at(
        &self,
        ui: &mut egui_plot::PlotUi,
        particle_id: ParticleID,
        centre: [f64; 2]
    ) {
        use egui_plot::MarkerShape::*;
        let col = self.get_particle_colour(particle_id.abs());
        let mut pt = Points::new(centre).color(col).radius(3.).highlight(true);
        if let Some(name) = particle_id.symbol() {
            pt = pt.name(name);
        }
        let shape = match spin_type(particle_id) {
            SpinType::Boson => Circle,
            SpinType::Fermion => Square,
            _ => Asterisk,
        };
        ui.points(pt.shape(shape));
    }

    fn draw_y_phi(
        &self,
        ui: &mut egui_plot::PlotUi,
        particle: &Particle
    ) {
        let Particle {id, y, phi, ..} = particle;

        debug!("Drawing particle {} at (y, φ) = ({y}, {phi})", id.id());
        let mut phi_min = ui.plot_bounds().min()[1].floor() as i64;
        phi_min -= phi_min % 4;
        let phi_max = ui.plot_bounds().max()[1];
        let mut centre = [y_to_coord(*y), phi_min as f64 + *phi / PHI_SCALE];
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
        let mut phi: f64 = jet.phi().into();
        if phi > PI {
            phi -= 2.*PI;
        }
        debug!("Drawing jet with radius {} at (y, φ) = ({}, {})", self.r_jet, jet.rap(), phi);
        let centre = (y_to_coord(jet.rap().into()), phi / PHI_SCALE);
        self.draw_jet_circle(ui, centre);
        // TODO: repeat at +- 2*pi in visible region
    }

    fn draw_jet_circle(
        &self,
        ui: &mut egui_plot::PlotUi,
        centre: (f64, f64)
    ) {
        todo!()
        // let jet_col = to_plotters_col(self.colour.jets);
        // chart.draw_series(
        //     AreaSeries::new(
        //         (0..101).map(
        //             |x| {
        //                 let x = x as f64;
        //                 let phi = x*2.*PI / 100.;
        //                 (
        //                     y_to_coord(centre.0 + self.r_jet*phi.cos()),
        //                     centre.1 + self.r_jet*phi.sin()
        //                 )
        //             }
        //         ),
        //         0.,
        //         ShapeStyle::from(jet_col).filled()
        //     )
        // ).unwrap();
    }

    fn draw_y_logpt(
        &self,
        ui: &mut egui_plot::PlotUi,
        particle: &Particle
    ) {
        let Particle { id, y, pt, .. } = particle;
        debug!("Drawing particle {} at (y, log(pt)) = ({y}, {})", id.id(), pt.log10());
        let centre = [y_to_coord(*y), pt.log10()];
        self.draw_particle_at(ui, *id, centre);
    }

    fn draw_y_logpt_jet(
        &self,
        ui: &mut egui_plot::PlotUi,
        jet: &PseudoJet
    ) {
        debug!("Drawing jet at (y, log(pt)) = ({}, {})", jet.rap(), jet.pt2().log10()/2.);
        let centre = (y_to_coord(jet.rap().into()), (jet.pt2().log10()/2.).into());
        let jet_col = self.colour.jets;
        let coord = [
            (y_to_coord(centre.0 - self.r_jet), f64::MIN),
            (y_to_coord(centre.0 + self.r_jet), centre.1),
        ];
        let rectangle = rectangle(coord).fill_color(jet_col);
        ui.polygon(rectangle);
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum VerticalPosition {
    Bottom,
    Top
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum HorizontalPosition {
    Left,
    Right
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Position {
    Left,
    Right,
    Bottom,
    Top
}

// at which point we transition between linear and logarithmic
// relation between y and plot coordinate
const Y_CUT: f64 = 4.;
const DY: f64 = Y_MAX - Y_CUT;

fn y_to_coord(y: f64) -> f64 {
    if y.abs() <= Y_CUT {
        y
    } else {
        y.signum() * (Y_CUT + DY * (1. - ((- y.abs() + Y_CUT) / DY).exp()))
    }
}

fn coord_to_y(coord: f64) -> f64 {
    match coord.abs() {
        c if c <= Y_CUT => coord,
        c if c <= Y_MAX => coord.signum() * (Y_CUT + DY * (DY / (Y_MAX - c)).ln()),
        _ => coord.signum() * f64::INFINITY
    }
}

fn add<T: std::ops::Add>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 + t2.0, t1.1 + t2.1)
}

fn sub<T: std::ops::Sub>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 - t2.0, t1.1 - t2.1)
}

fn rap_tick_label(
    coord: f64,
    max_chars: usize,
    _axis_range: &RangeInclusive<f64>
) -> String {
    match coord {
        c if c < Y_MIN => String::new(),
        c if c == Y_MIN => "-∞".to_string(),
        c if c < Y_MAX => {
            let res = format!("{}", coord_to_y(c));
            let res = String::from_iter(res.chars().take(max_chars));
            if let Some(end) = res.rfind(|c| c != '0') {
                res.chars().take(end + 1).collect()
            } else {
                "0".to_owned()
            }
        },
        c if c == Y_MAX => "∞".to_string(),
        _  => String::new(),
    }
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
