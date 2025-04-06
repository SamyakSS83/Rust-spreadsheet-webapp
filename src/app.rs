use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
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
        .route("/", get(serve_index))
        .route("/api/sheet", get(get_sheet_data))
        .route("/api/cell/:cell_name", get(get_cell))
        .route("/api/update_cell", post(update_cell))
        .route("/api/save", post(save_spreadsheet))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Start server
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("./static/index.html"))
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
