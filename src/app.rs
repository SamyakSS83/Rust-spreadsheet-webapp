use axum::{
    Json, Router,
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::saving;
use crate::spreadsheet::Spreadsheet;

pub struct AppState {
    sheet: Mutex<Box<Spreadsheet>>,
}

#[derive(Deserialize)]
struct CellUpdate {
    cell: String,
    formula: String,
}

#[derive(Serialize)]
struct CellResponse {
    status: String,
    value: Option<i32>,
}

#[derive(Deserialize)]
struct SaveQuery {
    filename: String,
}

#[derive(Deserialize)]
struct SheetQuery {
    rows: Option<i32>,
    cols: Option<i32>,
}

#[derive(Serialize)]
struct SaveResponse {
    status: String,
    message: Option<String>,
}

pub async fn run(rows: i32, cols: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Create spreadsheet
    let sheet = Spreadsheet::spreadsheet_create(rows, cols).expect("Failed to create spreadsheet");

    // Setup app state
    let app_state = Arc::new(AppState {
        sheet: Mutex::new(sheet),
    });

    // Build router
    let app = Router::new()
        .route("/", get(serve_landing))
        .route("/sheet", get(serve_sheet))
        .route("/api/sheet", get(get_sheet_data))
        .route("/api/cell/:cell_name", get(get_cell))
        .route("/api/update_cell", post(update_cell))
        .route("/api/save", post(save_spreadsheet))
        .route("/api/export", post(export_spreadsheet))
        .route("/api/load", post(load_spreadsheet))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Start server
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_landing() -> Html<&'static str> {
    Html(include_str!("./static/landing.html"))
}

async fn serve_sheet(
    Query(params): Query<SheetQuery>,
    State(state): State<Arc<AppState>>,
) -> Html<&'static str> {
    // If dimensions are provided, create a new sheet with those dimensions
    if let (Some(rows), Some(cols)) = (params.rows, params.cols) {
        if rows > 0 && rows <= 1000 && cols > 0 && cols <= 18278 {
            let new_sheet = Spreadsheet::spreadsheet_create(rows, cols)
                .expect("Failed to create spreadsheet with specified dimensions");

            let mut current_sheet = state.sheet.lock().unwrap();
            *current_sheet = new_sheet;
        }
    }

    Html(include_str!("./static/sheet.html"))
}

async fn get_sheet_data(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();
    let rows = sheet.rows;
    let cols = sheet.cols;

    let mut cell_data = Vec::new();

    for r in 1..=rows {
        for c in 1..=cols {
            let index = ((r - 1) * cols + (c - 1)) as usize;
            if let Some(cell) = &sheet.cells[index] {
                cell_data.push(serde_json::json!({
                    "row": r,
                    "col": c,
                    "name": Spreadsheet::get_cell_name(r, c),
                    "value": cell.value,
                    "formula": cell.formula,
                    "error": cell.error,
                }));
            }
        }
    }

    Json(serde_json::json!({
        "rows": rows,
        "cols": cols,
        "cells": cell_data,
    }))
}

async fn get_cell(
    Path(cell_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(&cell_name) {
        let index = ((row - 1) * sheet.cols + (col - 1)) as usize;
        if let Some(cell) = &sheet.cells[index] {
            return Json(serde_json::json!({
                "name": cell_name,
                "value": cell.value,
                "formula": cell.formula,
                "error": cell.error,
            }))
            .into_response();
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn update_cell(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CellUpdate>,
) -> impl IntoResponse {
    let mut sheet = state.sheet.lock().unwrap();
    let mut status = String::new();

    sheet.spreadsheet_set_cell_value(&payload.cell, &payload.formula, &mut status);

    // Get updated cell value
    let mut value = None;
    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(&payload.cell) {
        let index = ((row - 1) * sheet.cols + (col - 1)) as usize;
        if let Some(cell) = &sheet.cells[index] {
            value = Some(cell.value);
        }
    }

    Json(CellResponse { status, value })
}

async fn save_spreadsheet(
    Query(params): Query<SaveQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    match saving::save_spreadsheet(&sheet, &params.filename) {
        Ok(_) => Json(SaveResponse {
            status: "ok".to_string(),
            message: None,
        })
        .into_response(),
        Err(e) => Json(SaveResponse {
            status: "error".to_string(),
            message: Some(e.to_string()),
        })
        .into_response(),
    }
}

async fn export_spreadsheet(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    // Prepare a memory buffer to receive the serialized data
    let mut buffer = Vec::new();

    // Try to serialize the spreadsheet to the buffer
    match serialize_to_memory(&sheet, &mut buffer) {
        Ok(_) => {
            // Return the serialized data as a downloadable file
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/gzip")
                .body(axum::body::Body::from(Bytes::from(buffer)))
                .unwrap()
        }
        Err(e) => {
            // Return error response
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&SaveResponse {
                        status: "error".to_string(),
                        message: Some(e.to_string()),
                    })
                    .unwrap(),
                ))
                .unwrap()
        }
    }
}

async fn load_spreadsheet(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Process the multipart form data
    let mut file_data = Vec::new();
    let mut field_name = String::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        field_name = field.name().unwrap_or("unknown").to_string();

        if field_name == "spreadsheet" {
            file_data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }

    if file_data.is_empty() {
        return Json(SaveResponse {
            status: "error".to_string(),
            message: Some("No file data received".to_string()),
        })
        .into_response();
    }

    // Try to deserialize the spreadsheet
    match deserialize_from_memory(&file_data) {
        Ok(loaded_sheet) => {
            // Update the application's spreadsheet
            let mut sheet = state.sheet.lock().unwrap();
            *sheet = loaded_sheet;

            Json(SaveResponse {
                status: "ok".to_string(),
                message: None,
            })
            .into_response()
        }
        Err(e) => Json(SaveResponse {
            status: "error".to_string(),
            message: Some(format!("Failed to load spreadsheet: {}", e)),
        })
        .into_response(),
    }
}

// Helper function to serialize a spreadsheet to a memory buffer
fn serialize_to_memory(spreadsheet: &Spreadsheet, buffer: &mut Vec<u8>) -> std::io::Result<()> {
    use bincode::serialize_into;
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let encoder = GzEncoder::new(buffer, Compression::default());
    let mut writer = std::io::BufWriter::new(encoder);

    serialize_into(&mut writer, spreadsheet)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

// Helper function to deserialize a spreadsheet from a memory buffer
fn deserialize_from_memory(buffer: &[u8]) -> std::io::Result<Box<Spreadsheet>> {
    use bincode::deserialize_from;
    use flate2::read::GzDecoder;
    use std::io::Cursor;

    let cursor = Cursor::new(buffer);
    let decoder = GzDecoder::new(cursor);
    let mut reader = std::io::BufReader::new(decoder);

    let spreadsheet: Spreadsheet = deserialize_from(&mut reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(Box::new(spreadsheet))
}
