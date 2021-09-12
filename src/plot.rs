use crate::event::Event;
use crate::particle::{Particle, particle_name, SpinType, spin_type};

use std::collections::BTreeSet;
use std::iter;
use std::f64::consts::PI;
use std::path::Path;

use anyhow::Result;
use lazy_static::lazy_static;
use plotters::prelude::*;
use plotters::coord::Shift;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters_backend::BackendColor;
use log::{debug, trace};

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

const AXIS_LABEL_OFFSET: f64 = 0.9;
const TICK_LABEL_OFFSET: f64 = 0.25;
const LABEL_FONT_SIZE: f64 = 40.;

const BOX_CORNER: (f64, f64) = (0.05, 0.05);
const CIRCLE_SIZE: i32 = 5;

const Y_PHI_LEGEND_START_POS: (f64, f64) = (4., 3.);
const Y_PHI_LEGEND_STEP: f64 = 0.35;

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

fn y_to_coord(y: f64) -> f64 {
    const CUT: f64 = 4.;

    if y.abs() <= CUT {
        y
    } else {
        y.signum() * (CUT + (Y_MAX - CUT) * (1. - ((- y.abs() + CUT) / (Y_MAX - CUT)).exp()))
    }
}

fn draw_phi_ticks<DB: DrawingBackend, CT: CoordTranslate<From=(f64, f64)>>(
    chart: &mut ChartContext<'_, DB, CT>
) {
   for &phi in MAJOR_PHI_TICK_POS.iter() {
        trace!("Major tick at φ = {}", phi);
        chart.draw_series(
            LineSeries::new([(Y_AXIS_MIN, phi), (Y_AXIS_MIN + MAJOR_TICK_SIZE, phi)], &BLACK)
        ).unwrap();
        chart.draw_series(
            LineSeries::new([(Y_AXIS_MAX, phi), (Y_AXIS_MAX - MAJOR_TICK_SIZE, phi)], &BLACK)
        ).unwrap();
   }

    for &phi in MINOR_PHI_TICK_POS.iter() {
        trace!("Minor tick at φ = {}", phi);
        chart.draw_series(
            LineSeries::new([(Y_AXIS_MIN, phi), (Y_AXIS_MIN + MINOR_TICK_SIZE, phi)], &BLACK)
        ).unwrap();
        chart.draw_series(
            LineSeries::new([(Y_AXIS_MAX, phi), (Y_AXIS_MAX - MINOR_TICK_SIZE, phi)], &BLACK)
        ).unwrap();
    }
}

fn draw_y_ticks<DB: DrawingBackend, CT: CoordTranslate<From=(f64, f64)>>(
    chart: &mut ChartContext<'_, DB, CT>
) {
   for &y in MAJOR_Y_TICK_POS.iter() {
        trace!("Major tick at y = {}", y);
        chart.draw_series(
            LineSeries::new([(y, PHI_AXIS_MIN), (y, PHI_AXIS_MIN + MAJOR_TICK_SIZE)], &BLACK)
        ).unwrap();
        chart.draw_series(
            LineSeries::new([(y, PHI_AXIS_MAX), (y, PHI_AXIS_MAX - MAJOR_TICK_SIZE)], &BLACK)
        ).unwrap();
   }

    for &y in MINOR_Y_TICK_POS.iter() {
        trace!("Minor tick at y = {}", y);
        chart.draw_series(
            LineSeries::new([(y, PHI_AXIS_MIN), (y, PHI_AXIS_MIN + MINOR_TICK_SIZE)], &BLACK)
        ).unwrap();
        chart.draw_series(
            LineSeries::new([(y, PHI_AXIS_MAX), (y, PHI_AXIS_MAX - MINOR_TICK_SIZE)], &BLACK)
        ).unwrap();
    }
}

fn phi_tick_label<S: AsRef<str>, DB, X, Y>(
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

    draw_text(
        &root,
        &chart,
        text,
        (Y_AXIS_MIN - TICK_LABEL_OFFSET, pos + V_CORR),
        Pos{ h_pos: HPos::Right, v_pos: VPos::Center }
    );
}

fn y_tick_label<S: AsRef<str>, DB, X, Y>(
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

    draw_text(
        &root,
        &chart,
        text,
        (pos - H_CORR, PHI_AXIS_MIN - TICK_LABEL_OFFSET + V_CORR),
        Pos{ h_pos: HPos::Center, v_pos: VPos::Top }
    );
}

fn draw_text<S: AsRef<str>, DB, X, Y>(
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
            color: BackendColor{ alpha: 1., rgb: (0 ,0, 0) },
            pos: align
        },
        chart.backend_coord(&pos),
    ).unwrap()
}

fn dress_phi_axis<DB, X, Y>(
    root: & DrawingArea<DB, Shift>,
    mut chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
)
where
    DB: DrawingBackend,
    X: Ranged<ValueType = f64>,
    Y: Ranged<ValueType = f64>,
{
    draw_phi_ticks(&mut chart);
    phi_tick_label(&root, &chart, "π", PI);
    phi_tick_label(&root, &chart, "π/2", PI / 2.);
    phi_tick_label(&root, &chart, "0", 0.);
    phi_tick_label(&root, &chart, "-π/2", -PI / 2.);
    phi_tick_label(&root, &chart, "-π", -PI);
    draw_text(&root, &chart, "φ", (Y_AXIS_MIN - AXIS_LABEL_OFFSET, 0.1), Pos{ h_pos: HPos::Right, v_pos: VPos::Center });
}

fn dress_y_axis<DB, X, Y>(
    root: & DrawingArea<DB, Shift>,
    mut chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
)
where
    DB: DrawingBackend,
    X: Ranged<ValueType = f64>,
    Y: Ranged<ValueType = f64>,
{
    draw_y_ticks(&mut chart);
    for y in (Y_MIN as i32)..=(Y_MAX as i32) {
        y_tick_label(&root, &chart, &format!("{}", y), y_to_coord(y as f64));
    }
    // fudge slightly to avoid label collision
    y_tick_label(&root, &chart, "-∞", Y_MIN - 0.1);
    y_tick_label(&root, &chart, "∞", Y_MAX);
    draw_text(&root, &chart, "y", (-0.1, PHI_AXIS_MIN - AXIS_LABEL_OFFSET + 0.45), Pos{ h_pos: HPos::Center, v_pos: VPos::Top });
}

fn add<T: std::ops::Add>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 + t2.0, t1.1 + t2.1)
}

fn sub<T: std::ops::Sub>(t1: (T, T), t2: (T, T)) -> (T::Output, T::Output) {
    (t1.0 - t2.0, t1.1 - t2.1)
}

fn draw_particle_at<DB, X, Y>(
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
                    Into::<ShapeStyle>::into(&BLACK),
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
                    Into::<ShapeStyle>::into(&BLACK),
                )).unwrap();
        },
        _ => panic!("Cannot draw particle with type {}", particle_id)
    }
}

fn draw_y_phi<DB, X, Y>(
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
    draw_particle_at(&root, &chart, particle.id, centre);
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

pub fn plot<P: AsRef<Path>>(event: &Event, out: P) -> Result<()> {
    let root = SVGBackend::new(out.as_ref(), (1024, 768)).into_drawing_area();
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

    dress_y_axis(&root, &mut chart);
    dress_phi_axis(&root, &mut chart);

    let mut legend_ids = BTreeSet::new();
    for particle in &event.out {
        draw_y_phi(&root, &chart, &particle);
        legend_ids.insert(particle.id);
    }

    let mut pos = Y_PHI_LEGEND_START_POS;
    for id in legend_ids {
        draw_particle_at(&root, &chart, id, pos);
        draw_text(
            &root,
            &chart,
            particle_name(id),
            (pos.0 + 0.2, pos.1),
            Pos{ h_pos: HPos::Left, v_pos: VPos::Center }
        );

        pos.1 -= Y_PHI_LEGEND_STEP;
    }

     Ok(())
}
