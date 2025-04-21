#![cfg(not(tarpaulin_include))]

use axum::{
    Form, Json, Router,
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    middleware,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::downloader;
use crate::graph::{GraphOptions, GraphType, create_graph};
use crate::login::{
    self, User, UserCredentials, serve_change_password_page, serve_forgot_password_page,
    serve_reset_password_page,
};
use crate::saving;
use crate::spreadsheet::{FunctionName, Operand, ParsedRHS, Spreadsheet};

pub struct AppState {
    pub sheet: Mutex<Box<Spreadsheet>>,
    pub original_path: Mutex<Option<String>>,
    pub public_sheets: Mutex<HashSet<String>>,
}

#[derive(Debug, Deserialize)]
struct CellUpdate {
    rhs: String,
    cell: String,
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

#[derive(Debug, Deserialize)]
struct FileNameQuery {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GraphRequest {
    x_range: String,
    y_range: String,
    title: String,
    x_label: String,
    y_label: String,
    graph_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SheetEntry {
    name: String,
    status: String, // "public" or "private"
}

#[derive(Debug, Deserialize)]
struct ChangeStatusForm {
    status: String,
}

pub async fn run(rows: i16, cols: i16) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database
    login::init_database()?;

    // Create spreadsheet
    let sheet = Spreadsheet::spreadsheet_create(rows, cols).expect("Failed to create spreadsheet");

    // Setup app state
    let app_state = Arc::new(AppState {
        sheet: Mutex::new(sheet),
        original_path: Mutex::new(None),
        public_sheets: Mutex::new(HashSet::new()),
    });

    // 1) Build the public (no‐auth) routes
    let public = Router::new()
        .route("/", get(serve_landing))
        .route(
            "/login",
            get(login::serve_login_page).post(login::handle_login),
        )
        .route(
            "/signup",
            get(login::serve_signup_page).post(login::handle_signup),
        )
        .route("/logout", get(login::handle_logout))
        .route(
            "/forgot-password",
            get(serve_forgot_password_page).post(login::handle_forgot_password),
        )
        .route(
            "/reset-password",
            get(serve_reset_password_page).post(login::handle_reset_password),
        )
        .route(
            "/change-password",
            get(serve_change_password_page).post(login::handle_change_password),
        )
        // Public routes for accessing sheets
        .route("/:username/:sheet_name", get(load_user_file))
        // Add these API endpoints to public routes for public sheets
        .route("/api/sheet", get(get_sheet_data))
        .route("/api/cell/:cell_name", get(get_cell))
        .route("/api/sheet_info", get(get_sheet_info))
        .nest_service("/static", ServeDir::new("static"));

    // 2) Build the protected routes and apply auth‐middleware
    let protected = Router::new()
        // spreadsheet endpoints
        .route("/sheet", get(serve_sheet))
        // .route("/api/sheet", get(get_sheet_data))
        // .route("/api/cell/:cell_name", get(get_cell))
        // .route("/api/sheet_info", get(get_sheet_info))
        .route("/api/update_cell", post(update_cell))
        .route("/api/save", post(save_spreadsheet))
        .route("/api/export", post(export_spreadsheet))
        .route("/api/load", post(load_spreadsheet))
        .route("/api/graph", post(generate_graph))
        .route("/api/download/csv", get(download_csv))
        .route("/api/download/xlsx", get(download_xlsx))
        .route("/api/save_with_name", post(save_spreadsheet_with_name))
        // user file routes
        .route("/:username", get(login::list_files))
        .route(
            "/:username/create",
            get(login::serve_create_sheet_form).post(login::handle_create_sheet),
        )
        .route("/:username/:sheet_name/status", post(change_sheet_status))
        .route(
            "/:username/:sheet_name/delete",
            post(login::handle_delete_sheet),
        )
        // only these get require_auth
        .layer(middleware::from_fn(login::require_auth));

    // 3) Merge and attach shared state
    let app = Router::new()
        .merge(public)
        .merge(protected)
        .with_state(app_state);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn generate_graph(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GraphRequest>,
) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    let graph_type = match payload.graph_type.as_str() {
        "Line" => GraphType::Line,
        "Bar" => GraphType::Bar,
        "Scatter" => GraphType::Scatter,
        "Area" => GraphType::Area,
        _ => GraphType::Line,
    };

    let options = GraphOptions {
        title: payload.title,
        x_label: payload.x_label,
        y_label: payload.y_label,
        width: 800,
        height: 600,
        graph_type,
    };

    match create_graph(&sheet, &payload.x_range, &payload.y_range, options) {
        Ok(img_data) => ([("Content-Type", "image/png")], img_data).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Failed to create graph: {}", e),
        )
            .into_response(),
    }
}

async fn serve_landing() -> impl IntoResponse {
    // Redirect to login page
    Redirect::to("/login")
}

async fn serve_sheet(
    Query(params): Query<SheetQuery>,
    State(state): State<Arc<AppState>>,
) -> Html<&'static str> {
    if let (Some(rows), Some(cols)) = (params.rows, params.cols) {
        if rows > 0 && rows <= 1000 && cols > 0 && cols <= 18278 {
            let new_sheet = Spreadsheet::spreadsheet_create(rows as i16, cols as i16)
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
                    "formula": formula_to_string(&cell.formula),  // Convert to string
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
                "formula": formula_to_string(&cell.formula),  // Convert to string
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
    // println!("(DEBUG) Received update_cell payload: {:?}", payload);
    let mut sheet = state.sheet.lock().unwrap();
    let mut status = String::new();

    // Parse the cell name
    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(&payload.cell) {
        // println!("(DEBUG) Parsed cell name: row={}, col={}", row, col);

        // Parse the formula string into ParsedRHS using is_valid_command
        let (is_valid, _, _, parsed_rhs) = sheet.is_valid_command(&payload.cell, &payload.rhs);

        if is_valid {
            // println!("(DEBUG) Valid formula parsed: {:?}", parsed_rhs);
            sheet.spreadsheet_set_cell_value(row, col, parsed_rhs, &mut status);
        } else {
            status = format!("Invalid formula: {}", payload.rhs);
            // println!("(DEBUG) {}", status);
        }
    } else {
        status = format!("Invalid cell identifier: {}", payload.cell);
        // println!("(DEBUG) {}", status);
    }

    // Retrieve the updated cell value and print its state
    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(&payload.cell) {
        let index = ((row - 1) * sheet.cols + (col - 1)) as usize;
        if let Some(cell) = &sheet.cells[index] {
            // println!(
            //     "(DEBUG) Final state of cell {}: value = {}, formula = {:?}, error = {}",
            //     payload.cell, cell.value, cell.formula, cell.error
            // );
            Json(CellResponse {
                status,
                value: Some(cell.value),
            })
            .into_response()
        } else {
            // println!("(DEBUG) Missing cell at index {}", index);
            Json(CellResponse {
                status: "Cell not found".into(),
                value: None,
            })
            .into_response()
        }
    } else {
        // println!(
        //     "(DEBUG) Second parsing of cell identifier failed for '{}'",
        //     payload.cell
        // );
        Json(CellResponse {
            status,
            value: None,
        })
        .into_response()
    }
}

async fn save_spreadsheet(
    Query(params): Query<SaveQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Get the sheet and original path
    let sheet = state.sheet.lock().unwrap();
    let mut original_path = state.original_path.lock().unwrap();

    // Get filename from query params or use original path if none provided
    let filename = if params.filename.is_empty() {
        // Try to use the original path
        match original_path.as_ref() {
            Some(path) => path.clone(),
            None => {
                return Json(SaveResponse {
                    status: "error".to_string(),
                    message: Some(
                        "No filename provided and no original path available".to_string(),
                    ),
                })
                .into_response();
            }
        }
    } else {
        // For new sheets, update the original path with the provided filename
        let new_filename = params.filename.clone();
        *original_path = Some(new_filename.clone());
        new_filename
    };

    match saving::save_spreadsheet(&sheet, &filename) {
        Ok(_) => Json(SaveResponse {
            status: "ok".to_string(),
            message: None,
        })
        .into_response(),
        Err(e) => {
            // If save fails, don't keep the path for new sheets
            if original_path.as_ref().unwrap() == &filename && params.filename == filename {
                *original_path = None;
            }

            Json(SaveResponse {
                status: "error".to_string(),
                message: Some(e.to_string()),
            })
            .into_response()
        }
    }
}

// Modified save function to save to user directory
async fn save_spreadsheet_with_name(
    State(state): State<Arc<AppState>>,
    username: axum::extract::Extension<String>,
    Form(query): Form<FileNameQuery>,
) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    // Create user directory if it doesn't exist
    let user_dir = format!("database/{}", username.0);
    let _ = std::fs::create_dir_all(&user_dir);

    // Build the filename
    let filename = if query.name.trim().is_empty() {
        "spreadsheet.bin.gz".to_string()
    } else {
        if !query.name.ends_with(".bin.gz") {
            format!("{}.bin.gz", query.name)
        } else {
            query.name
        }
    };

    let path = format!("{}/{}", user_dir, filename);

    // Update original path
    let mut original_path = state.original_path.lock().unwrap();
    *original_path = Some(path.clone());

    // Save the file
    match saving::save_spreadsheet(&sheet, &path) {
        Ok(_) => Json(SaveResponse {
            status: "ok".to_string(),
            message: None,
        })
        .into_response(),
        Err(e) => {
            *original_path = None;
            Json(SaveResponse {
                status: "error".to_string(),
                message: Some(e.to_string()),
            })
            .into_response()
        }
    }
}

// Load user file by path
async fn load_user_file(
    axum::extract::Path((username, filename)): axum::extract::Path<(String, String)>,
    jar: CookieJar, // New parameter
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let path = format!("database/{}/{}.bin.gz", username, filename);

    // Check if file exists
    if !std::path::Path::new(&path).exists() {
        return Html(format!("<h1>File not found</h1><p>Path: {}</p>", path)).into_response();
    }

    // Try to validate the session directly from the cookie.
    let current_user = jar
        .get("session")
        .and_then(|cookie| crate::login::validate_session(cookie.value()));

    // If current user exists and matches the owner, then mark as owner.
    let is_owner = current_user.as_deref() == Some(&username);

    // If not owner, check if the sheet is public.
    let mut is_public = false;
    if !is_owner {
        let list_path = format!("database/{}/list.json", username);
        if let Ok(data) = std::fs::read_to_string(&list_path) {
            if let Ok(entries) = serde_json::from_str::<Vec<crate::login::SheetEntry>>(&data) {
                is_public = entries
                    .iter()
                    .any(|entry| entry.name == filename && entry.status == "public");
                if !is_public {
                    return Redirect::to("/login").into_response();
                }
            } else {
                return Redirect::to("/login").into_response();
            }
        } else {
            return Redirect::to("/login").into_response();
        }
    }

    // Load the file as before
    match std::fs::read(&path) {
        Ok(file_data) => {
            match deserialize_from_memory(&file_data) {
                Ok(loaded_sheet) => {
                    {
                        let mut sheet_guard = state.sheet.lock().unwrap();
                        *sheet_guard = loaded_sheet;
                        let mut path_guard = state.original_path.lock().unwrap();
                        *path_guard = Some(path);
                    }
                    // If the sheet is public, record it.
                    if is_public {
                        let mut public_sheets = state.public_sheets.lock().unwrap();
                        public_sheets.insert(format!("{}/{}", username, filename));
                    }

                    // Serve the sheet
                    serve_sheet(
                        Query(SheetQuery {
                            rows: None,
                            cols: None,
                        }),
                        State(Arc::clone(&state)),
                    )
                    .await
                    .into_response()
                }
                Err(_) => Html("<h1>Error loading spreadsheet</h1>".to_string()).into_response(),
            }
        }
        Err(_) => Html("<h1>Error reading file</h1>".to_string()).into_response(),
    }
}

async fn change_sheet_status(
    axum::extract::Path((username, filename)): axum::extract::Path<(String, String)>,
    current_user: axum::extract::Extension<String>,
    Form(form): Form<ChangeStatusForm>,
) -> impl IntoResponse {
    // Security check - only owner can change status
    if username != current_user.0 {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // Ensure status is valid
    if form.status != "public" && form.status != "private" {
        return (StatusCode::BAD_REQUEST, "Invalid status").into_response();
    }

    // Update list.json
    let list_path = format!("database/{}/list.json", username);
    let mut entries = if std::path::Path::new(&list_path).exists() {
        match std::fs::read_to_string(&list_path) {
            Ok(data) => match serde_json::from_str::<Vec<SheetEntry>>(&data) {
                Ok(entries) => entries,
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // Find and update the entry
    let mut found = false;
    for entry in &mut entries {
        if entry.name == filename {
            entry.status = form.status.clone();
            found = true;
            break;
        }
    }

    // If not found, add a new entry
    if !found {
        entries.push(SheetEntry {
            name: filename,
            status: form.status,
        });
    }

    // Save the updated list
    if let Ok(json) = serde_json::to_string_pretty(&entries) {
        if std::fs::write(&list_path, json).is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update sheet status",
            )
                .into_response();
        }
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize sheet data",
        )
            .into_response();
    }

    // Redirect back to the user's sheet list
    Redirect::to(&format!("/{}", username)).into_response()
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
    let _field_name = String::new();
    let mut file_path = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("unknown").to_string();

        if field_name == "spreadsheet" {
            // First get the filename before consuming the field with bytes()
            file_path = field.file_name().map(|s| s.to_string());

            // Now get the bytes (this consumes the field)
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

            // Store the original file path
            if let Some(path) = file_path {
                let mut original_path = state.original_path.lock().unwrap();
                *original_path = Some(path);
            }

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

async fn download_csv(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    match downloader::to_csv(&sheet) {
        Ok(csv_content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/csv")
            .header(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"spreadsheet.csv\"",
            )
            .body(axum::body::Body::from(csv_content))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(axum::body::Body::from(format!(
                "Error generating CSV: {}",
                e
            )))
            .unwrap(),
    }
}

async fn download_xlsx(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sheet = state.sheet.lock().unwrap();

    match downloader::to_xlsx(&sheet) {
        Ok(xlsx_data) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )
            .header(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"spreadsheet.xlsx\"",
            )
            .body(axum::body::Body::from(Bytes::from(xlsx_data)))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(axum::body::Body::from(format!(
                "Error generating XLSX: {}",
                e
            )))
            .unwrap(),
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

// Add this function to convert ParsedRHS to a display string
fn formula_to_string(formula: &ParsedRHS) -> String {
    match formula {
        ParsedRHS::Function {
            name,
            args: (arg1, arg2),
        } => {
            let func_name = match name {
                FunctionName::Min => "MIN",
                FunctionName::Max => "MAX",
                FunctionName::Avg => "AVG",
                FunctionName::Sum => "SUM",
                FunctionName::Stdev => "STDEV",
                FunctionName::Cut => "CUT",
                FunctionName::Copy => "COPY",
            };

            let cell1 = match arg1 {
                Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
                Operand::Number(n) => n.to_string(),
            };

            let cell2 = match arg2 {
                Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
                Operand::Number(n) => n.to_string(),
            };

            format!("{}({}:{})", func_name, cell1, cell2)
        }
        ParsedRHS::Arithmetic { lhs, operator, rhs } => {
            let left = match lhs {
                Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
                Operand::Number(n) => n.to_string(),
            };

            let right = match rhs {
                Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
                Operand::Number(n) => n.to_string(),
            };

            format!("{}{}{}", left, operator, right)
        }
        ParsedRHS::Sleep(operand) => {
            let value = match operand {
                Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
                Operand::Number(n) => n.to_string(),
            };

            format!("SLEEP({})", value)
        }
        ParsedRHS::SingleValue(operand) => match operand {
            Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
            Operand::Number(n) => n.to_string(),
        },
        ParsedRHS::None => String::new(),
    }
}

async fn get_sheet_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let original_path = state.original_path.lock().unwrap();

    Json(serde_json::json!({
        "is_loaded": original_path.is_some(),
        "original_path": original_path.clone().unwrap_or_default(),
    }))
}
