use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

use crate::auto_decompress::auto_decompress;
use crate::event::Event;

use anyhow::{anyhow, Context, Result};
use log::debug;

pub fn import(filename: &Path, events: &mut Vec<Event>) -> Result<()> {
    let file = File::open(filename)?;
    let mut reader = auto_decompress(BufReader::new(file));
    let buf = reader.fill_buf()?;

    if starts_with(buf, b"<LesHouchesEvents") {
        debug!("trying to import {:?} as LHEF file", filename);
        import_lhef(reader, events).with_context(
            || format!("Failed to import {:?}", filename)
        )
    } else if starts_with(buf, b"HepMC") {
        debug!("trying to import {:?} as HepMC file", filename);
        import_hepmc(reader, events).with_context(
            || format!("Failed to import {:?}", filename)
        )
    } else {
        Err(anyhow!("Failed to import {:?}: Unknown file format", filename))
    }
}

fn starts_with<T: std::cmp::PartialEq>(slice: &[T], prefix: &[T]) -> bool {
    if prefix.len() > slice.len() {
        return false;
    }
    &slice[..prefix.len()] == prefix
}

fn import_lhef<R: BufRead>(reader: R, events: &mut Vec<Event>) -> Result<()> {
    let mut reader = lhef::Reader::new(reader).map_err(
        |err| anyhow!("Error construction LHEF reader: {}", err)
    )?;
    while let Some(event) = reader.hepeup().map_err(
        |err| anyhow!("Error reading LHEF event: {}", err)
    )? {
        events.push(Event::from(&event))
    }
    Ok(())
}

fn import_hepmc<R: BufRead>(reader: R, events: &mut Vec<Event>) -> Result<()> {
    let mut reader = hepmc2::reader::Reader::new(reader);
    while let Some(event) = reader.next() {
        let event = event.map_err(
            |err| anyhow!("Error reading HepMC event: {}", err)
        )?;
        events.push(Event::from(&event))
    }
    Ok(())
}
