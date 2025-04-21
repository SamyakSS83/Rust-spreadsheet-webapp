#![cfg(not(tarpaulin_include))]

use bincode::{deserialize_from, serialize_into};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;

use crate::spreadsheet::Spreadsheet;

/// Saves a spreadsheet to a file
///
/// This function serializes a spreadsheet and saves it to a file with compression.
/// The spreadsheet is first serialized using bincode, then compressed using gzip.
///
/// # Arguments
/// * `spreadsheet` - Reference to the spreadsheet to save
/// * `filename` - Path to the file where the spreadsheet should be saved
///
/// # Returns
/// * `std::io::Result<()>` - Success or an IO error
///
/// # Examples
/// ```
/// use cop::spreadsheet::Spreadsheet;
/// use cop::saving::save_spreadsheet;
///
/// let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
/// let result = save_spreadsheet(&sheet, "my_spreadsheet.bin.gz");
/// ```
pub fn save_spreadsheet(spreadsheet: &Spreadsheet, filename: &str) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut writer = std::io::BufWriter::new(encoder);

    serialize_into(&mut writer, spreadsheet)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

/// Loads a spreadsheet from a file
///
/// This function deserializes a spreadsheet from a compressed file.
/// The file is first decompressed using gzip, then deserialized using bincode.
///
/// # Arguments
/// * `filename` - Path to the file containing the saved spreadsheet
///
/// # Returns
/// * `std::io::Result<Spreadsheet>` - The loaded spreadsheet or an IO error
///
/// # Examples
/// ```
/// use cop::saving::load_spreadsheet;
///
/// match load_spreadsheet("my_spreadsheet.bin.gz") {
///     Ok(sheet) => println!("Loaded spreadsheet with {} rows and {} columns", sheet.rows, sheet.cols),
///     Err(e) => eprintln!("Failed to load spreadsheet: {}", e),
/// }
/// ```
pub fn load_spreadsheet(filename: &str) -> std::io::Result<Spreadsheet> {
    let file = File::open(filename)?;
    let decoder = GzDecoder::new(file);
    let mut reader = std::io::BufReader::new(decoder);

    let spreadsheet: Spreadsheet = deserialize_from(&mut reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(spreadsheet)
}
