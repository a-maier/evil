mod asy;

use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{Context, Result};
use jetty::PseudoJet;

use crate::{
    export::asy::export_asy,
    plotter::{self, ExportFormat, PlotKind},
    Event,
};

pub(crate) fn export(
    path: &Path,
    event: &Event,
    jets: &[PseudoJet],
    r_jet: f64,
    kind: PlotKind,
    format: ExportFormat,
    settings: &plotter::Settings,
) -> Result<()> {
    use ExportFormat::*;
    let out = File::create(path)
        .with_context(|| format!("Failed to open {path:?}"))?;
    let out = BufWriter::new(out);
    match format {
        Asymptote => export_asy(out, event, jets, r_jet, kind, settings),
    }
}
