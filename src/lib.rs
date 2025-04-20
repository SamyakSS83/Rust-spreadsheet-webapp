pub mod cell;
pub mod downloader;
pub mod graph;
pub mod login;
pub mod saving;
pub mod spreadsheet;

// Only include app module when building with web feature
#[cfg(feature = "web")]
pub mod app;
