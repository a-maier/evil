use std::io::Write;

use anyhow::{Context, Result};

use crate::{Event, plotter::PlotKind, windows::PlotterSettings};

pub(crate) fn export_asy(
    mut out: impl Write,
    event: &Event,
    kind: PlotKind,
    settings: &PlotterSettings,
) -> Result<()> {
    use PlotKind::*;
    //todo!("write common code");
    match kind {
        YPhi => export_asy_y_phi(out, event, settings),
        YLogPt => export_asy_y_logpt(out, event, settings),
    }
}

pub(crate) fn export_asy_y_phi(
    mut out: impl Write,
    event: &Event,
    settings: &PlotterSettings,
) -> Result<()> {
    todo!()
}

pub(crate) fn export_asy_y_logpt(
    mut out: impl Write,
    event: &Event,
    settings: &PlotterSettings,
) -> Result<()> {
    todo!()
}
