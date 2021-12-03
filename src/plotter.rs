use crate::event::Event;
use crate::font::Font;
use crate::particle::{Particle, particle_name, SpinType, spin_type};

use std::borrow::Borrow;
use std::collections::{BTreeSet, HashMap};
use std::iter::{self, FromIterator};
use std::f64::consts::PI;
use std::ops::Range;

use anyhow::Result;
use jetty::PseudoJet;
use lazy_static::lazy_static;
use num_traits::float::Float;
use plotters::prelude::*;
use plotters::coord::Shift;
use plotters::style::{
    RGBAColor,
    text_anchor::{HPos, Pos, VPos}
};
use log::{debug};
use serde::{Deserialize, Serialize};

const GOLDEN_RATIO: f64 = 1.618033988749894848;
const PHI_MIN: f64 = -PI;
const PHI_MAX: f64 = PI;
const PHI_AXIS_MIN: f64 = -3.25;
const PHI_AXIS_MAX: f64 = -PHI_AXIS_MIN;
const Y_MIN: f64 = -5.;
const Y_MAX: f64 = 5.;
const Y_AXIS_MIN: f64 = GOLDEN_RATIO * PHI_AXIS_MIN;
const Y_AXIS_MAX: f64 = GOLDEN_RATIO * PHI_AXIS_MAX;
const MAJOR_TICK_SIZE: i32 = 14;
const MINOR_TICK_SIZE: i32 = MAJOR_TICK_SIZE / 2;
const N_MAJOR_PHI_TICKS: usize = 5;
const N_MINOR_PHI_TICKS: usize = 3;

const X_AXIS_LABEL_OFFSET: i32 = 50;
const Y_AXIS_LABEL_OFFSET: i32 = 60;
const Y_AXIS_LABEL_OFFSET_LOGPT: i32 = 80;
const TICK_LABEL_OFFSET: i32 = 10;
const TICK_LABEL_OFFSET_LOGPT: i32 = 20;

const BOX_CORNER: (i32, i32) = (5, 5);
const CIRCLE_SIZE: i32 = 5;

const LEGEND_X_POS: f64 = 4.;
const LEGEND_START_REL: f64 = 0.95;
const LEGEND_REL_STEP: f64 = 0.05;

pub const REL_SUB_FONT_SIZE: f64 = 0.6;

lazy_static!{
    static ref CYAN: egui::Color32 = egui::Color32::from_rgb(0, 159, 223);
    static ref ORANGE: egui::Color32 = egui::Color32::from_rgb(241, 143, 31);
    static ref MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);
}

lazy_static!(
    static ref MAJOR_PHI_TICK_POS: [f64; N_MAJOR_PHI_TICKS] = {
        let mut arr = [0.0; N_MAJOR_PHI_TICKS];
        for (n, e) in arr.iter_mut().enumerate() {
            *e = PHI_MIN + n as f64 / (N_MAJOR_PHI_TICKS - 1) as f64 * (PHI_MAX - PHI_MIN);
        }
        arr
    };
);

lazy_static!(
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
        for n in 0..res.len() {
            res[n] = pos[n];
        }
        res
    };
);

lazy_static!(
    static ref MAJOR_Y_TICK_POS: Vec<f64> =
        iter::once(Y_MIN).chain(
            (Y_MIN as i32 ..=(Y_MAX as i32)).map(|i| y_to_coord(i as f64))
        ).chain(iter::once(Y_MAX)).collect();
);

lazy_static!(
    static ref MINOR_Y_TICK_POS: Vec<f64> = {
        let mut pos = vec![-4.5, -3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5, 4.5];
        for y in &mut pos {
            *y = y_to_coord(*y);
        }
        pos
    };
);

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
                (1,  egui::Color32::BLUE),
                (2,  egui::Color32::DARK_GREEN),
                (3,  *CYAN),
                (4,  *MAGENTA),
                (5,  egui::Color32::BLACK),
                (6,  egui::Color32::BROWN),
                (11, egui::Color32::YELLOW),
                (12, egui::Color32::WHITE),
                (13, *ORANGE),
                (14, egui::Color32::LIGHT_GRAY),
                (15, egui::Color32::RED),
                (16, egui::Color32::GRAY),
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

#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct Plotter {
    pub r_jet: f64,

    pub font: Font,

    pub colour: ColourSettings,
}

impl Plotter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plot_y_phi(
        &self,
        event: &Event,
        jets: &[PseudoJet]
    ) -> Result<String> {
        let mut plot = String::new();
        self.plot_y_phi_to(event, jets, &mut plot)?;
        Ok(plot)
    }

    pub fn plot_y_logpt(
        &self,
        event: &Event,
        jets: &[PseudoJet],
        logpt_range: Range<f64>
    ) -> Result<String> {
        let mut plot = String::new();
        self.plot_y_logpt_to(event, jets, logpt_range, &mut plot)?;
        Ok(plot)
    }

    pub fn plot_y_phi_to(
        &self,
        event: &Event,
        jets: &[PseudoJet],
        result: &mut String
    ) -> Result<()> {

        let root = SVGBackend::with_string(result, (1024, 768)).into_drawing_area();
        root.fill(&to_plotters_col(self.colour.background))?;
        //let root = root.margin(10, 10, 10, 10);
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .set_all_label_area_size(10)
            .set_label_area_size(LabelAreaPosition::Left, 100)
            .set_label_area_size(LabelAreaPosition::Bottom, 80)
            .build_cartesian_2d(Y_AXIS_MIN..Y_AXIS_MAX, PHI_AXIS_MIN..PHI_AXIS_MAX)?;

        chart.configure_mesh()
            .disable_mesh()
            .x_labels(0)
            .y_labels(0)
            .axis_style(to_plotters_col(self.colour.frame))
            .draw()?;

        self.dress_rap_axis(&root, &mut chart);
        self.dress_phi_axis(&root, &mut chart);

        for jet in jets {
            self.draw_y_phi_jet(&root, &mut chart, &jet);
        }
        let mut legend_ids = BTreeSet::new();
        for particle in &event.out {
            self.draw_y_phi(&root, &chart, &particle);
            legend_ids.insert(particle.id);
        }

        let y_size = chart.y_range().end - chart.y_range().start;
        let y_legend = chart.y_range().start + LEGEND_START_REL * y_size;
        let mut pos = (LEGEND_X_POS, y_legend);
        for id in legend_ids {
            self.draw_particle_at(&root, &chart, id, pos);
            self.draw_text(
                &root,
                &chart,
                particle_name(id),
                (pos.0 + 0.2, pos.1),
                (0, 0),
                Pos{ h_pos: HPos::Left, v_pos: VPos::Center }
            );

            pos.1 -= LEGEND_REL_STEP * y_size;
        }

        Ok(())
    }

    pub fn plot_y_logpt_to(
        &self,
        event: &Event,
        jets: &[PseudoJet],
        logpt_range: Range<f64>,
        result: &mut String
    ) -> Result<()> {

        let root = SVGBackend::with_string(result, (1024, 768)).into_drawing_area();
        root.fill(&to_plotters_col(self.colour.background))?;
        // let root = root.margin(10, 10, 10, 10);
        let logpt_start = logpt_range.start - 0.05 * logpt_range.start.abs();
        let logpt_end = logpt_range.end + 0.05 * logpt_range.end.abs();
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .set_all_label_area_size(5)
            .set_label_area_size(LabelAreaPosition::Left, 110)
            .set_label_area_size(LabelAreaPosition::Bottom, 80)
            .build_cartesian_2d(Y_AXIS_MIN..Y_AXIS_MAX, logpt_start..logpt_end)?;

        chart.configure_mesh()
            .disable_mesh()
            .x_labels(0)
            .y_labels(0)
            .axis_style(to_plotters_col(self.colour.frame))
            .draw()?;

        self.dress_rap_axis(&root, &mut chart);
        let logpt_range = logpt_start.ceil() as i64 .. logpt_end.floor() as i64;
        self.dress_logpt_axis(&root, &mut chart, logpt_range.clone());

        for jet in jets {
            self.draw_y_logpt_jet(&root, &mut chart, &jet);
        }
        let mut legend_ids = BTreeSet::new();
        for particle in &event.out {
            self.draw_y_logpt(&root, &chart, &particle);
            legend_ids.insert(particle.id);
        }

        let y_size = chart.y_range().end - chart.y_range().start;
        let y_legend = chart.y_range().start + LEGEND_START_REL * y_size;
        let mut pos = (LEGEND_X_POS, y_legend);
        for id in legend_ids {
            self.draw_particle_at(&root, &chart, id, pos);
            self.draw_text(
                &root,
                &chart,
                particle_name(id),
                (pos.0 + 0.2, pos.1),
                (0, 0),
                Pos{ h_pos: HPos::Left, v_pos: VPos::Center }
            );

            pos.1 -= LEGEND_REL_STEP * y_size;
        }

        Ok(())
    }

    // TODO: return Result
    fn draw_x_tick<'b, DB, X, Y, S>(
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
    fn draw_y_tick<'b, DB, X, Y, S>(
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

    fn draw_tick<'b, DB, X, Y, S>(
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

    fn draw_ticks<'b, DB, X, Y, P, I>(
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
        let col: RGBAColor = to_plotters_col(self.colour.frame);
        let style: ShapeStyle = col.into();
        for pos in pos.into_iter() {
            self.draw_tick(root, chart, *pos.borrow(), size, align, style.clone());
        }
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
            &root,
            &chart,
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
            &root,
            &chart,
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
        let col = to_plotters_col(self.colour.frame);
        let pos = add(chart.backend_coord(&pos), offset);
        root.draw_text(
            text.as_ref(),
            &TextStyle {
                font: (&self.font).into(),
                color: col.to_backend_color(),
                pos: align
            },
            pos,
        ).unwrap()
    }

    fn dress_phi_axis<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        mut chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        self.draw_phi_ticks(root, &mut chart);
        self.phi_tick_label(&root, &chart, "π", PI);
        self.phi_tick_label(&root, &chart, "π/2", PI / 2.);
        self.phi_tick_label(&root, &chart, "0", 0.);
        self.phi_tick_label(&root, &chart, "-π/2", -PI / 2.);
        self.phi_tick_label(&root, &chart, "-π", -PI);
        self.draw_text(
            &root, &chart,
            "φ",
            (Y_AXIS_MIN, 0.0),
            (- Y_AXIS_LABEL_OFFSET, 0),
            Pos{ h_pos: HPos::Right, v_pos: VPos::Center }
        );
    }

    fn dress_logpt_axis<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        mut chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        mut range: Range<i64>
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        range.end += 1;
        self.draw_logpt_ticks(root, &mut chart, range.clone());
        let col = to_plotters_col(self.colour.frame);
        let align = Pos{ h_pos: HPos::Right, v_pos: VPos::Center };
        let style = TextStyle {
            font: (&self.font).into(),
            color: col.to_backend_color(),
            pos: align
        };
        let sup_style = TextStyle {
            font: style.font.resize(REL_SUB_FONT_SIZE * self.font.size),
            color: style.color.clone(),
            pos: align
        };
        // TODO: how to calculate this properly?
        let s = self.font.size as i32;
        for logpt in range {
            let sup_pos = if logpt < 0 {
                (s / 3, -(s / 4))
            } else {
                (s / 10, -(s / 4))
            };
            let pos = (Y_AXIS_MIN, logpt as f64);
            let offset = ( - TICK_LABEL_OFFSET_LOGPT, 0);
            let mut pos = chart.backend_coord(&pos);
            pos.0 += offset.0;
            pos.1 += offset.1;
            root.draw(
                &(
                    EmptyElement::at(pos)
                        + Text::new("10", (-s / 4, 0), &style)
                        + Text::new(logpt.to_string(), sup_pos, &sup_style)
                )
            ).unwrap()
        }
        let y_range = chart.y_range();

        let pos = (Y_AXIS_MIN, (y_range.start + y_range.end) / 2.);
        let offset = ( - Y_AXIS_LABEL_OFFSET_LOGPT, 0);
        let pos = add(chart.backend_coord(&pos), offset);
        root.draw(
            &(
                EmptyElement::at(pos)
                    + Text::new("p", (-s / 4, 0), &style)
                    + Text::new("T", (s / 10, s / 4), &sup_style)
            )
        ).unwrap()
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
            self.rap_tick_label(&root, &chart, &format!("{}", y), y_to_coord(y as f64));
        }
        // fudge slightly to avoid label collision
        self.rap_tick_label(&root, &chart, "-∞", Y_MIN - 0.1);
        self.rap_tick_label(&root, &chart, "∞", Y_MAX);
        let ymin = chart.y_range().start;
        self.draw_text(
            &root, &chart,
            "y",
            (0., ymin),
            (0, X_AXIS_LABEL_OFFSET),
            Pos{ h_pos: HPos::Center, v_pos: VPos::Top }
        );
    }


    fn get_particle_colour(&self, pid: i32) -> RGBAColor {
        let col = self.colour.particles.get(
            &pid.abs()
        ).unwrap_or(&egui::Color32::GRAY);
        to_plotters_col(*col)
    }

    fn draw_particle_at<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        particle_id: i32,
        centre: (f64, f64)
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        let fill_col = self.get_particle_colour(particle_id.abs());
        let frame_col = to_plotters_col(self.colour.frame);
        match spin_type(particle_id) {
            SpinType::Boson => {
                let coord = chart.backend_coord(&centre);
                root.draw(
                    &Circle::new(
                        coord,
                        CIRCLE_SIZE,
                        Into::<ShapeStyle>::into(&fill_col).filled(),
                    )).unwrap();
                root.draw(
                    &Circle::new(
                        coord,
                        CIRCLE_SIZE,
                        Into::<ShapeStyle>::into(&frame_col),
                    )).unwrap();
            },
            SpinType::Fermion => {
                let centre = chart.backend_coord(&centre);
                let coord = [
                    sub(centre, BOX_CORNER),
                    add(centre, BOX_CORNER)
                ];
                root.draw(
                    &Rectangle::new(
                        coord,
                        Into::<ShapeStyle>::into(&fill_col).filled(),
                    )).unwrap();
                root.draw(
                    &Rectangle::new(
                        coord,
                        Into::<ShapeStyle>::into(&frame_col),
                    )).unwrap();
            },
            _ => panic!("Cannot draw particle with type {}", particle_id)
        }
    }

    fn draw_y_phi<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        particle: &Particle
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        debug!("Drawing particle {} at (y, φ) = ({}, {})", particle.id, particle.y, particle.phi);
        let centre = (y_to_coord(particle.y), particle.phi);
        self.draw_particle_at(&root, &chart, particle.id, centre);
    }

    fn draw_jet_circle<DB, X, Y>(
        &self,
        chart: & mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        centre: (f64, f64)
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        let jet_col = to_plotters_col(self.colour.jets);
        chart.draw_series(
            AreaSeries::new(
                (0..101).map(
                    |x| {
                        let x = x as f64;
                        let phi = x*2.*PI / 100.;
                        (
                            y_to_coord(centre.0 + self.r_jet*phi.cos()),
                            centre.1 + self.r_jet*phi.sin()
                        )
                    }
                ),
                0.,
                ShapeStyle::from(jet_col).filled()
            )
        ).unwrap();
    }

    fn draw_y_phi_jet<DB, X, Y>(
        &self,
        _root: & DrawingArea<DB, Shift>,
        chart: & mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        jet: &PseudoJet
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        let mut phi: f64 = jet.phi().into();
        if phi > PI {
            phi -= 2.*PI;
        }
        debug!("Drawing jet with radius {} at (y, φ) = ({}, {})", self.r_jet, jet.rap(), phi);
        let centre = (y_to_coord(jet.rap().into()), phi);
        self.draw_jet_circle(chart, centre);
        if centre.1 - self.r_jet < - PI {
            self.draw_jet_circle(chart, (centre.0, centre.1 + 2.*PI));
        }
        if centre.1 + self.r_jet > PI {
            self.draw_jet_circle(chart, (centre.0, centre.1 - 2.*PI));
        }
    }

    fn draw_y_logpt<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        particle: &Particle
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        debug!("Drawing particle {} at (y, log(pt)) = ({}, {})", particle.id, particle.y, particle.pt.log10());
        let centre = (y_to_coord(particle.y), particle.pt.log10());
        self.draw_particle_at(&root, &chart, particle.id, centre);
    }

    fn draw_y_logpt_jet<DB, X, Y>(
        &self,
        _root: & DrawingArea<DB, Shift>,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        jet: &PseudoJet
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        debug!("Drawing jet at (y, log(pt)) = ({}, {})", jet.rap(), jet.pt2().log10()/2.);
        let centre = (y_to_coord(jet.rap().into()), (jet.pt2().log10()/2.).into());
        let jet_col = to_plotters_col(self.colour.jets);
        chart.draw_series(
            AreaSeries::new(
                [
                    (y_to_coord(centre.0 + self.r_jet), centre.1),
                    (y_to_coord(centre.0 - self.r_jet), centre.1),
                    (y_to_coord(centre.0 - self.r_jet), -100.), // TODO: min pt
                    (y_to_coord(centre.0 + self.r_jet), -100.)
                ],
                0.,
                ShapeStyle::from(jet_col).filled(),
            )
        ).unwrap();
    }
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

fn y_to_coord(y: f64) -> f64 {
    const CUT: f64 = 4.;

    if y.abs() <= CUT {
        y
    } else {
        y.signum() * (CUT + (Y_MAX - CUT) * (1. - ((- y.abs() + CUT) / (Y_MAX - CUT)).exp()))
    }
}

fn add<T: std::ops::Add>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 + t2.0, t1.1 + t2.1)
}

fn sub<T: std::ops::Sub>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 - t2.0, t1.1 - t2.1)
}

fn to_plotters_col(col: egui::Color32) -> RGBAColor {
    let (r,g,b,a) = col.to_tuple();
    RGBColor(r, g, b).mix((a as f64) / (u8::MAX as f64))
}
