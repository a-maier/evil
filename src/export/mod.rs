mod asy;

use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{Context, Result};

use crate::{Event, plotter::{PlotKind, ExportFormat}, windows::PlotterSettings, export::asy::export_asy};

pub(crate) fn export(
    path: &Path,
    event: &Event,
    kind: PlotKind,
    format: ExportFormat,
    settings: &PlotterSettings,
) -> Result<()> {
    use ExportFormat::*;
    let out =
        File::open(path).with_context(|| format!("Failed to open {path:?}"))?;
    let out = BufWriter::new(out);
    match format {
        Asymptote => export_asy(out, event, kind, settings),
    }
}
