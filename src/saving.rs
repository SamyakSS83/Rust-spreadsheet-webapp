use bincode::{deserialize_from, serialize_into};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;

use crate::spreadsheet::Spreadsheet;

pub fn save_spreadsheet(spreadsheet: &Spreadsheet, filename: &str) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut writer = std::io::BufWriter::new(encoder);

    serialize_into(&mut writer, spreadsheet)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

pub fn load_spreadsheet(filename: &str) -> std::io::Result<Spreadsheet> {
    let file = File::open(filename)?;
    let decoder = GzDecoder::new(file);
    let mut reader = std::io::BufReader::new(decoder);

    let spreadsheet: Spreadsheet = deserialize_from(&mut reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(spreadsheet)
}
