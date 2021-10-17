use crate::event::Event;
use crate::particle::{Particle, particle_name, SpinType, spin_type};

use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::iter;
use std::f64::consts::PI;
use std::ops::Range;

use anyhow::Result;
use lazy_static::lazy_static;
use plotters::prelude::*;
use plotters::coord::Shift;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters_backend::BackendColor;
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
const MAJOR_TICK_SIZE: f64 = 0.15;
const MINOR_TICK_SIZE: f64 = MAJOR_TICK_SIZE / 2.;
const N_MAJOR_PHI_TICKS: usize = 5;
const N_MINOR_PHI_TICKS: usize = 3;

const X_AXIS_LABEL_OFFSET: f64 = 0.45;
const Y_AXIS_LABEL_OFFSET: f64 = 0.9;
const TICK_LABEL_OFFSET: f64 = 0.25;
const LABEL_FONT_SIZE: f64 = 40.;

const BOX_CORNER: (f64, f64) = (0.05, 0.05);
const CIRCLE_SIZE: i32 = 5;

const LEGEND_X_POS: f64 = 4.;
const LEGEND_START_REL: f64 = 0.95;
const LEGEND_REL_STEP: f64 = 0.05;

pub const GREY: RGBColor = RGBColor(130, 130, 130);

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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize, Serialize)]
pub struct Plotter {
}

impl Plotter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plot_y_phi(&self, event: &Event) -> Result<String> {
        let mut plot = String::new();
        self.plot_y_phi_to(event, &mut plot)?;
        Ok(plot)
    }

    pub fn plot_y_logpt(&self, event: &Event, logpt_range: Range<f64>) -> Result<String> {
        let mut plot = String::new();
        self.plot_y_logpt_to(event, logpt_range, &mut plot)?;
        Ok(plot)
    }

    pub fn plot_y_phi_to(&self, event: &Event, result: &mut String) -> Result<()> {

        let root = SVGBackend::with_string(result, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
        let root = root.margin(10, 10, 10, 10);
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .set_all_label_area_size(5)
            .set_label_area_size(LabelAreaPosition::Left, 80)
            .set_label_area_size(LabelAreaPosition::Bottom, 80)
            .build_cartesian_2d(Y_AXIS_MIN..Y_AXIS_MAX, PHI_AXIS_MIN..PHI_AXIS_MAX)?;

        chart.configure_mesh()
            .disable_mesh()
            .x_labels(0)
            .y_labels(0)
            .draw()?;

        self.dress_rap_axis(&root, &mut chart);
        self.dress_phi_axis(&root, &mut chart);

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
                Pos{ h_pos: HPos::Left, v_pos: VPos::Center }
            );

            pos.1 -= LEGEND_REL_STEP * y_size;
        }

        Ok(())
    }

    pub fn plot_y_logpt_to(
        &self,
        event: &Event,
        logpt_range: Range<f64>,
        result: &mut String
    ) -> Result<()> {

        let root = SVGBackend::with_string(result, (1024, 768)).into_drawing_area();
        let root = root.margin(10, 10, 10, 10);
        let logpt_start = logpt_range.start - 0.05 * logpt_range.start.abs();
        let logpt_end = logpt_range.end + 0.05 * logpt_range.end.abs();
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .set_all_label_area_size(5)
            .set_label_area_size(LabelAreaPosition::Left, 80)
            .set_label_area_size(LabelAreaPosition::Bottom, 80)
            .build_cartesian_2d(Y_AXIS_MIN..Y_AXIS_MAX, logpt_start..logpt_end)?;

        chart.configure_mesh()
            .disable_mesh()
            .x_labels(0)
            .y_labels(0)
            .draw()?;

        self.dress_rap_axis(&root, &mut chart);
        let logpt_range = logpt_start.ceil() as i64 .. logpt_end.floor() as i64;
        self.dress_logpt_axis(&root, &mut chart, logpt_range.clone());

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
                Pos{ h_pos: HPos::Left, v_pos: VPos::Center }
            );

            pos.1 -= LEGEND_REL_STEP * y_size;
        }

        Ok(())
    }

    // TODO: return Result
    fn draw_x_tick<'b, DB, X, Y, S>(
        &self,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: f64,
        size: f64,
        align: VerticalPosition,
        style: S,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
        S: Into<ShapeStyle>,
    {

        let (y_start, y_end) = match align {
            VerticalPosition::Bottom => (chart.y_range().start, chart.y_range().start + size),
            VerticalPosition::Top => (chart.y_range().end, chart.y_range().end - size),
        };
        chart.draw_series(
            LineSeries::new([(pos, y_start), (pos, y_end)], style)
        ).unwrap();
    }

    // TODO: return Result
    fn draw_y_tick<'b, DB, X, Y, S>(
        &self,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: f64,
        size: f64,
        align: HorizontalPosition,
        style: S,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
        S: Into<ShapeStyle>,
    {

        let (x_start, x_end) = match align {
            HorizontalPosition::Left => (chart.x_range().start, chart.x_range().start + size),
            HorizontalPosition::Right => (chart.x_range().end, chart.x_range().end - size),
        };
        chart.draw_series(
            LineSeries::new([(x_start, pos), (x_end, pos)], style)
        ).unwrap();
    }

    fn draw_tick<'b, DB, X, Y, S>(
        &self,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: f64,
        size: f64,
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
            Position::Left => self.draw_y_tick(chart, pos, size, Left, style),
            Position::Right => self.draw_y_tick(chart, pos, size, Right, style),
            Position::Bottom => self.draw_x_tick(chart, pos, size, Bottom, style),
            Position::Top => self.draw_x_tick(chart, pos, size, Top, style),
        }
    }

    fn draw_ticks<'b, DB, X, Y, S, P, I>(
        &self,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
        pos: P,
        size: f64,
        align: Position,
        style: S,
    )
    where
        DB: DrawingBackend,
        P: IntoIterator<Item=I>,
        I: Borrow<f64>,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
        S: Into<ShapeStyle>,
    {
        let style = style.into();
        for pos in pos.into_iter() {
            self.draw_tick(chart, *pos.borrow(), size, align, style.clone())
        }
    }

    fn draw_phi_ticks<DB: DrawingBackend, X, Y>(
        &self,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>
    )
    where
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        use Position::{Left, Right};
        for align in [Left, Right] {
            self.draw_ticks(chart, MAJOR_PHI_TICK_POS.iter(), MAJOR_TICK_SIZE, align, &GREY);
            self.draw_ticks(chart, MINOR_PHI_TICK_POS.iter(), MINOR_TICK_SIZE, align, &GREY);
        }
    }

    fn draw_logpt_ticks<DB: DrawingBackend, X, Y>(
        &self,
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
            self.draw_ticks(chart, major_tick_pos, MAJOR_TICK_SIZE, align, &GREY);
            let mut range = range.clone();
            range.end += 1;
            let minor_tick_pos = range.into_iter().map(
                |pos| (1..10).map(move |step| pos as f64 + (step as f64).log10() - 1.)
            ).flatten();
            self.draw_ticks(chart, minor_tick_pos, MINOR_TICK_SIZE, align, &GREY);
        }
    }

    fn draw_rap_ticks<DB: DrawingBackend, X, Y>(
        &self,
        chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>
    )
    where
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        use Position::{Bottom, Top};
        for align in [Bottom, Top] {
            self.draw_ticks(chart, MAJOR_Y_TICK_POS.iter(), MAJOR_TICK_SIZE, align, &GREY);
            self.draw_ticks(chart, MINOR_Y_TICK_POS.iter(), MINOR_TICK_SIZE, align, &GREY);
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
        // TODO: vertical alignment is broken?
        const V_CORR: f64 = 0.1;

        self.draw_text(
            &root,
            &chart,
            text,
            (Y_AXIS_MIN - TICK_LABEL_OFFSET, pos + V_CORR),
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
        // TODO: alignment is broken?
        const H_CORR: f64 = 0.1;
        const V_CORR: f64 = 0.2;
        let ymin = chart.y_range().start;
        self.draw_text(
            &root,
            &chart,
            text,
            (pos - H_CORR, ymin - TICK_LABEL_OFFSET + V_CORR),
            Pos{ h_pos: HPos::Center, v_pos: VPos::Top }
        );
    }

    fn draw_text<S: AsRef<str>, DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        chart: & ChartContext<'_, DB, Cartesian2d<X, Y>>,
        text: S,
        pos: (f64, f64),
        align: Pos,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        root.draw_text(
            text.as_ref(),
            &TextStyle {
                font: ("serif", LABEL_FONT_SIZE).into_font(),
                color: BackendColor{ alpha: 1., rgb: (GREY.0, GREY.1, GREY.2) },
                pos: align
            },
            chart.backend_coord(&pos),
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
        self.draw_phi_ticks(&mut chart);
        self.phi_tick_label(&root, &chart, "π", PI);
        self.phi_tick_label(&root, &chart, "π/2", PI / 2.);
        self.phi_tick_label(&root, &chart, "0", 0.);
        self.phi_tick_label(&root, &chart, "-π/2", -PI / 2.);
        self.phi_tick_label(&root, &chart, "-π", -PI);
        self.draw_text(&root, &chart, "φ", (Y_AXIS_MIN - Y_AXIS_LABEL_OFFSET, 0.1), Pos{ h_pos: HPos::Right, v_pos: VPos::Center });
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
        self.draw_logpt_ticks(&mut chart, range.clone());
        for logpt in range {
            let logpt = logpt as f64;
            // TODO: vertical alignment is broken?
            const V_CORR: f64 = 0.1;
            self.draw_text(
                &root,
                &chart,
                // TODO: proper superscript
                format!("10^{}", logpt),
                (Y_AXIS_MIN - TICK_LABEL_OFFSET, logpt + V_CORR),
                Pos{ h_pos: HPos::Right, v_pos: VPos::Center }
            );
        }
        let y_range = chart.y_range();
        self.draw_text(&root, &chart, "pt", (Y_AXIS_MIN - Y_AXIS_LABEL_OFFSET, (y_range.start + y_range.end) / 2.), Pos{ h_pos: HPos::Right, v_pos: VPos::Center });
    }

    fn dress_rap_axis<DB, X, Y>(
        &self,
        root: & DrawingArea<DB, Shift>,
        mut chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    )
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = f64>,
        Y: Ranged<ValueType = f64>,
    {
        self.draw_rap_ticks(&mut chart);
        for y in (Y_MIN as i32)..=(Y_MAX as i32) {
            self.rap_tick_label(&root, &chart, &format!("{}", y), y_to_coord(y as f64));
        }
        // fudge slightly to avoid label collision
        self.rap_tick_label(&root, &chart, "-∞", Y_MIN - 0.1);
        self.rap_tick_label(&root, &chart, "∞", Y_MAX);
        let ymin = chart.y_range().start;
        self.draw_text(&root, &chart, "y", (-0.1, ymin - X_AXIS_LABEL_OFFSET), Pos{ h_pos: HPos::Center, v_pos: VPos::Top });
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
        let col = colour(particle_id.abs());
        match spin_type(particle_id) {
            SpinType::Boson => {
                let coord = chart.backend_coord(&centre);
                root.draw(
                    &Circle::new(
                        coord,
                        CIRCLE_SIZE,
                        Into::<ShapeStyle>::into(&col).filled(),
                    )).unwrap();
                root.draw(
                    &Circle::new(
                        coord,
                        CIRCLE_SIZE,
                        Into::<ShapeStyle>::into(&GREY),
                    )).unwrap();
            },
            SpinType::Fermion => {
                let coord = [
                    chart.backend_coord(&sub(centre, BOX_CORNER)),
                    chart.backend_coord(&add(centre, BOX_CORNER))
                ];
                root.draw(
                    &Rectangle::new(
                        coord,
                        Into::<ShapeStyle>::into(&col).filled(),
                    )).unwrap();
                root.draw(
                    &Rectangle::new(
                        coord,
                        Into::<ShapeStyle>::into(&GREY),
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

pub fn colour(id: i32) -> PaletteColor<Palette99> {
    match id {
        1  => Palette99::pick(0),
        2  => Palette99::pick(1),
        3  => Palette99::pick(2),
        4  => Palette99::pick(3),
        5  => Palette99::pick(4),
        11 => Palette99::pick(5),
        12 => Palette99::pick(6),
        13 => Palette99::pick(7),
        14 => Palette99::pick(8),
        15 => Palette99::pick(9),
        16 => Palette99::pick(10),
        21 => Palette99::pick(11),
        22 => Palette99::pick(12),
        23 => Palette99::pick(13),
        24 => Palette99::pick(15),
        25 => Palette99::pick(16),
        _  => Palette99::pick(20),
    }
}
