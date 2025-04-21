/// A spreadsheet application library supporting cell formulas, dependencies, and operations.
///
/// This library provides functionality for creating and manipulating spreadsheets with
/// formula evaluation, dependency tracking, and various spreadsheet operations like
/// arithmetic calculations and aggregation functions.
///
/// # Main Components
///
/// * [`spreadsheet`](spreadsheet) - Core spreadsheet functionality including evaluation and management
/// * [`cell`](cell) - Individual cell representation and dependency handling
/// * [`downloader`](downloader) - Utilities for downloading spreadsheet data
/// * [`graph`](graph) - Visualization of cell dependency relationships
/// * [`login`](login) - User authentication and session management
/// * [`mailer`](mailer) - Email functionality for sharing spreadsheets
/// * [`saving`](saving) - Persistence and serialization of spreadsheet data
///
/// # Usage Examples
///
/// ## Creating a Spreadsheet
///
/// ```rust
/// use cop::spreadsheet::Spreadsheet;
///
/// // Create a 10x10 spreadsheet
/// let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
///
/// // Set a simple numeric value in cell A1
/// let mut status = String::new();
/// let parsed_rhs = cop::spreadsheet::ParsedRHS::SingleValue(
///     cop::spreadsheet::Operand::Number(42)
/// );
/// sheet.spreadsheet_set_cell_value(1, 1, parsed_rhs, &mut status);
/// ```
///
/// ## Creating Cell Dependencies
///
/// ```rust
/// use cop::spreadsheet::{Spreadsheet, ParsedRHS, Operand, FunctionName};
///
/// let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
/// let mut status = String::new();
///
/// // Set cell A1 to 10
/// sheet.spreadsheet_set_cell_value(1, 1,
///     ParsedRHS::SingleValue(Operand::Number(10)),
///     &mut status
/// );
///
/// // Set cell A2 to reference A1
/// sheet.spreadsheet_set_cell_value(2, 1,
///     ParsedRHS::SingleValue(Operand::Cell(1, 1)),
///     &mut status
/// );
///
/// // Now A2 depends on A1 and will update when A1 changes
/// ```
pub use cell::*;
pub use spreadsheet::*;

pub mod cell;
pub mod downloader;
pub mod graph;
pub mod login;
pub mod mailer;
pub mod saving;
pub mod spreadsheet;
// Only include app module when building with web feature
#[cfg(feature = "web")]
pub mod app;
