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
#[cfg(feature = "web")]
// use local_ip_address::local_ip; // Add this import
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
// #[cfg(feature = "web")]
// use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
// use actix_web::post;
use crate::downloader;
use crate::graph::{GraphOptions, GraphType, create_graph};
use crate::login::{
    self, serve_change_password_page, serve_forgot_password_page, serve_reset_password_page,
};
use crate::saving;
use crate::spreadsheet::{FunctionName, Operand, ParsedRHS, Spreadsheet};

/// Application state shared across all requests
///
/// This structure contains the shared state for the web application, including:
/// - The current spreadsheet in memory
/// - The original file path for saving/loading operations
/// - A set of public spreadsheets that can be accessed without authentication
pub struct AppState {
    /// The current spreadsheet data, wrapped in a mutex for thread-safe access
    pub sheet: Mutex<Box<Spreadsheet>>,

    /// The original file path of the loaded spreadsheet (if any)
    /// Used to save the spreadsheet back to its source location
    pub original_path: Mutex<Option<String>>,

    /// A set of publicly accessible spreadsheets, identified by their paths
    /// Format: "username/sheetname"
    pub public_sheets: Mutex<HashSet<String>>,

    /// Version counter for conflict management
    pub version: Mutex<u64>,

    /// Last modified timestamp
    pub last_modified: Mutex<std::time::SystemTime>,
}

/// Data structure for cell updates from the client
#[derive(Debug, Deserialize)]
struct CellUpdate {
    /// The right-hand side formula or value to be parsed
    rhs: String,
    /// The cell identifier (e.g., "A1", "B2")
    cell: String,
    /// Client's version of the sheet (for conflict detection)
    version: Option<u64>,
}

/// Response data structure for cell updates
#[derive(Serialize)]
struct CellResponse {
    /// Status message indicating success or the error that occurred
    status: String,
    /// The calculated cell value (if successful)
    value: Option<i32>,
    /// Current version of the sheet after update
    version: u64,
    /// Whether the client needs to refresh due to conflict
    needs_refresh: bool,
}

/// Query parameters for saving a spreadsheet
#[derive(Deserialize)]
struct SaveQuery {
    /// Filename for the spreadsheet
    filename: String,
}

/// Query parameters for sheet creation/initialization
#[derive(Deserialize)]
struct SheetQuery {
    /// Number of rows to create (optional)
    rows: Option<i32>,
    /// Number of columns to create (optional)
    cols: Option<i32>,
}

/// Response structure for save operations
#[derive(Serialize)]
struct SaveResponse {
    /// Status of the operation ("ok" or "error")
    status: String,
    /// Optional message with additional details, especially for errors
    message: Option<String>,
}

/// Query parameters for operations that require a filename
#[derive(Debug, Deserialize)]
struct FileNameQuery {
    /// Name of the file
    name: String,
}

/// Request data for graph generation
#[derive(Debug, Deserialize)]
struct GraphRequest {
    /// Cell range for X-axis values (e.g., "A1:A10")
    x_range: String,
    /// Cell range for Y-axis values (e.g., "B1:B10")
    y_range: String,
    /// Title for the graph
    title: String,
    /// Label for the X-axis
    x_label: String,
    /// Label for the Y-axis
    y_label: String,
    /// Type of graph ("Line", "Bar", "Scatter", "Area")
    graph_type: String,
}

/// Data structure for listing spreadsheets
#[derive(Debug, Serialize, Deserialize)]
struct SheetEntry {
    /// Name of the spreadsheet
    name: String,
    /// Visibility status ("public" or "private")
    status: String,
}

/// Form data for changing spreadsheet visibility status
#[derive(Debug, Deserialize)]
struct ChangeStatusForm {
    /// New status value ("public" or "private")
    status: String,
}

/// Response structure for sheet status
#[derive(Serialize)]
struct SheetStatusResponse {
    /// Current version of the sheet
    version: u64,
    /// Last modified time as timestamp
    last_modified: u64,
}

/// Query parameters for getting sheet data
#[derive(Deserialize)]
struct SheetDataQuery {
    start_row: Option<i16>,
    start_col: Option<i16>,
    rows: Option<i16>,
    cols: Option<i16>,
}

/// Main application entry point
///
/// Initializes the database, creates the default spreadsheet, and starts the web server.
/// Sets up both public and authenticated routes for the application.
///
/// # Arguments
/// * `rows` - Number of rows for the initial spreadsheet
/// * `cols` - Number of columns for the initial spreadsheet
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
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
        version: Mutex::new(0), // Initialize version to 0
        last_modified: Mutex::new(std::time::SystemTime::now()),
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
        .route("/api/update_cell", post(update_cell))
        .route("/api/sheet_info", get(get_sheet_info))
        .route("/api/save", post(save_spreadsheet))
        .route("/api/export", post(export_spreadsheet))
        .route("/api/load", post(load_spreadsheet))
        .route("/api/graph", post(generate_graph))
        .route("/api/download/csv", get(download_csv))
        .route("/api/download/xlsx", get(download_xlsx))
        .route("/api/sheet_status", get(get_sheet_status))
        .route("/api/undo", get(undo).post(undo))
        .nest_service("/static", ServeDir::new("static"));

    // 2) Build the protected routes and apply auth‐middleware
    let protected = Router::new()
        // spreadsheet endpoints
        .route("/sheet", get(serve_sheet))
        // .route("/api/sheet", get(get_sheet_data))
        // .route("/api/cell/:cell_name", get(get_cell))
        // .route("/api/sheet_info", get(get_sheet_info))
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
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    // let local_ip = local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
    // println!("Listening on http://{}:8000", local_ip);
    println!("Listening on http://localhost:8000");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Generate a graph based on spreadsheet data
///
/// Creates and returns a graph image (PNG format) based on data ranges from the spreadsheet.
///
/// # Arguments
/// * `state` - Application state containing the spreadsheet
/// * `payload` - Graph configuration including ranges, labels, and graph type
///
/// # Returns
/// * A PNG image of the requested graph, or an error message
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

/// Serve the landing page
///
/// Redirects to the login page.
///
/// # Returns
/// * Redirect response to the login page
async fn serve_landing() -> impl IntoResponse {
    // Redirect to login page
    Redirect::to("/login")
}

/// Serve the spreadsheet application page
///
/// Optionally creates a new spreadsheet with the specified dimensions.
/// Returns the HTML for the spreadsheet application.
///
/// # Arguments
/// * `params` - Optional query parameters specifying sheet dimensions
/// * `state` - Application state
///
/// # Returns
/// * HTML content for the spreadsheet UI
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

/// Get spreadsheet data in JSON format
///
/// Returns the current spreadsheet data including all cells with values.
///
/// # Arguments
/// * `params` - Query parameters for pagination
/// * `state` - Application state containing the spreadsheet
///
/// # Returns
/// * JSON representation of the spreadsheet data
async fn get_sheet_data(
    Query(params): Query<SheetDataQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Try to lock the spreadsheet state with error handling
    let sheet_result = state.sheet.lock();
    let sheet = match sheet_result {
        Ok(sheet) => sheet,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Spreadsheet is currently unavailable"
                })),
            )
                .into_response();
        }
    };

    // Get total dimensions
    let total_rows = sheet.rows;
    let total_cols = sheet.cols;

    // Parse pagination parameters with defaults
    let start_row = params.start_row.unwrap_or(1).max(1);
    let start_col = params.start_col.unwrap_or(1).max(1);
    let page_rows = params.rows.unwrap_or(50).min(100).max(1); // Limit to 100 rows
    let page_cols = params.cols.unwrap_or(50).min(100).max(1); // Limit to 100 cols

    // Calculate end bounds respecting sheet dimensions
    let end_row = (start_row + page_rows - 1).min(total_rows);
    let end_col = (start_col + page_cols - 1).min(total_cols);

    let mut cell_data = Vec::new();

    // Only process cells in the requested range
    for r in start_row..=end_row {
        for c in start_col..=end_col {
            // Safely calculate index with bounds checking
            if r > 0 && r <= total_rows && c > 0 && c <= total_cols {
                let index = ((r - 1) * total_cols + (c - 1)) as usize;

                // Make sure index is within bounds of the cells array
                if index < sheet.cells.len() {
                    if let Some(cell) = &sheet.cells[index] {
                        cell_data.push(serde_json::json!({
                            "row": r,
                            "col": c,
                            "name": Spreadsheet::get_cell_name(r, c),
                            "value": cell.value,
                            "formula": formula_to_string(&cell.formula),
                            "error": cell.error,
                        }));
                    }
                }
            }
        }
    }

    Json(serde_json::json!({
        "totalRows": total_rows,
        "totalCols": total_cols,
        "startRow": start_row,
        "startCol": start_col,
        "rows": end_row - start_row + 1,
        "cols": end_col - start_col + 1,
        "cells": cell_data
    }))
    .into_response()
}

/// Get data for a specific cell
///
/// Returns the value, formula and error status of a specific cell.
///
/// # Arguments
/// * `cell_name` - Cell identifier (e.g., "A1", "B2")
/// * `state` - Application state containing the spreadsheet
///
/// # Returns
/// * JSON data for the requested cell or 404 if not found
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

/// Update a cell's value in the spreadsheet
///
/// Parses the input formula/value and updates the specified cell.
///
/// # Arguments
/// * `state` - Application state containing the spreadsheet
/// * `jar` - Cookie jar containing session information
/// * `payload` - Cell update data including the cell name and formula/value
///
/// # Returns
/// * JSON response with update status and the new cell value
async fn update_cell(
    State(state): State<Arc<AppState>>,
    jar: CookieJar, // Added for session check
    Json(payload): Json<CellUpdate>,
) -> impl IntoResponse {
    // Get the current sheet path to check permissions
    let original_path = state.original_path.lock().unwrap().clone();

    // Check permission for editing if not logged in or not owner
    if let Some(path) = &original_path {
        // Extract username/sheet_name from path "database/username/sheet_name.bin.gz"
        let path_parts: Vec<&str> = path.split('/').collect();
        if path_parts.len() >= 3 {
            let username = path_parts[1];
            let sheet_name = path_parts[2].trim_end_matches(".bin.gz");

            // Get current user from session cookie
            let current_user = jar
                .get("session")
                .and_then(|cookie| crate::login::validate_session(cookie.value()));

            let is_owner = current_user.as_deref() == Some(username);

            // If not owner, check if sheet is public
            if !is_owner {
                let public_sheets = state.public_sheets.lock().unwrap();
                let sheet_key = format!("{}/{}", username, sheet_name);

                // If it's not in public sheets, return unauthorized
                if !public_sheets.contains(&sheet_key) {
                    return Json(CellResponse {
                        status: "Error: Not authorized to edit this sheet".to_string(),
                        value: None,
                        version: 0,
                        needs_refresh: false,
                    })
                    .into_response();
                }
            }
        }
    }

    // Check for version conflicts
    let mut current_version = state.version.lock().unwrap();
    let client_version = payload.version.unwrap_or(0);

    // If client has an outdated version, notify them to refresh
    if client_version < *current_version {
        return Json(CellResponse {
            status: "Conflict: sheet has been modified".to_string(),
            value: None,
            version: *current_version,
            needs_refresh: true,
        })
        .into_response();
    }

    // Continue with update logic
    let mut sheet = state.sheet.lock().unwrap();
    let mut status = String::new();
    let mut was_updated = false;

    // Parse the cell name and update the cell
    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(&payload.cell) {
        let (is_valid, _, _, parsed_rhs) = sheet.is_valid_command(&payload.cell, &payload.rhs);

        if is_valid {
            // Store the current value before updating
            let current_value = {
                let index = ((row - 1) * sheet.cols + (col - 1)) as usize;
                sheet
                    .cells
                    .get(index)
                    .and_then(|c| c.as_ref())
                    .map(|c| c.value)
            };

            // Update the cell
            sheet.spreadsheet_set_cell_value(row, col, parsed_rhs, &mut status);
            // println!("Updated cell {}: {}", payload.cell, status);
            // Check if the status indicates an error (e.g., cycle detection)
            if status == "Cycle Detected" {
                // println!("Error: Circular reference detected");
                return Json(CellResponse {
                    status: "Error: Circular reference detected".to_string(),
                    value: None,
                    version: *current_version,
                    needs_refresh: false,
                })
                .into_response();
            }

            // Check if value actually changed
            let new_value = {
                let index = ((row - 1) * sheet.cols + (col - 1)) as usize;
                sheet
                    .cells
                    .get(index)
                    .and_then(|c| c.as_ref())
                    .map(|c| c.value)
            };

            // If value changed
            if current_value != new_value {
                was_updated = true;

                // Increment version
                *current_version += 1;

                // Update last modified time
                *state.last_modified.lock().unwrap() = std::time::SystemTime::now();
            }
        } else {
            return Json(CellResponse {
                status: format!("Error: Invalid formula '{}'", payload.rhs),
                value: None,
                version: *current_version,
                needs_refresh: false,
            })
            .into_response();
        }
    } else {
        return Json(CellResponse {
            status: format!("Error: Invalid cell reference '{}'", payload.cell),
            value: None,
            version: *current_version,
            needs_refresh: false,
        })
        .into_response();
    }

    // Auto-save if the sheet was updated
    if was_updated {
        if let Some(path) = &original_path {
            // Create a clone of the sheet for saving to avoid deadlocks
            let sheet_clone = {
                let sheet_ref = &*sheet;
                bincode::serialize(sheet_ref)
                    .ok()
                    .and_then(|data| bincode::deserialize(&data).ok())
            };

            if let Some(sheet_to_save) = sheet_clone {
                // Spawn a task to save in the background to avoid blocking
                let path_clone = path.clone();
                tokio::spawn(async move {
                    let _ = saving::save_spreadsheet(&sheet_to_save, &path_clone);
                });
            }
        }
    }

    // Prepare response with current cell value and version
    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(&payload.cell) {
        let index = ((row - 1) * sheet.cols + (col - 1)) as usize;
        if let Some(cell) = &sheet.cells.get(index).and_then(|c| c.as_ref()) {
            // Check if the cell has an error flag set
            if cell.error {
                return Json(CellResponse {
                    status: "Error: Invalid calculation result".to_string(),
                    value: Some(cell.value),
                    version: *current_version,
                    needs_refresh: false,
                })
                .into_response();
            }

            Json(CellResponse {
                status: "ok".to_string(),
                value: Some(cell.value),
                version: *current_version,
                needs_refresh: false,
            })
            .into_response()
        } else {
            Json(CellResponse {
                status: "Error: Cell not found".into(),
                value: None,
                version: *current_version,
                needs_refresh: false,
            })
            .into_response()
        }
    } else {
        Json(CellResponse {
            status: format!("Error: Invalid cell reference '{}'", payload.cell),
            value: None,
            version: *current_version,
            needs_refresh: false,
        })
        .into_response()
    }
}

/// Save the current spreadsheet
///
/// Saves the spreadsheet to the provided filename or to the original path.
///
/// # Arguments
/// * `params` - Query parameters containing the filename
/// * `state` - Application state containing the spreadsheet
///
/// # Returns
/// * JSON response indicating success or failure
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

/// Save spreadsheet to a user's directory with a specific name
///
/// Saves the current spreadsheet to the authenticated user's directory
/// with the specified name.
///
/// # Arguments
/// * `state` - Application state containing the spreadsheet
/// * `username` - The authenticated username
/// * `query` - Form data containing the filename
///
/// # Returns
/// * JSON response indicating success or failure
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
    } else if !query.name.ends_with(".bin.gz") {
        format!("{}.bin.gz", query.name)
    } else {
        query.name
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

/// Load a user's spreadsheet file
///
/// Loads a spreadsheet file from a user's directory and serves the spreadsheet UI.
/// Checks for public/private access permissions.
///
/// # Arguments
/// * `username` and `filename` - Path parameters identifying the file
/// * `jar` - Cookie jar containing session information
/// * `state` - Application state
///
/// # Returns
/// * Spreadsheet UI if authorized, or redirects to login
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

/// Change the visibility status of a spreadsheet
///
/// Updates a spreadsheet's status to either "public" or "private".
///
/// # Arguments
/// * `username` and `filename` - Path parameters identifying the file
/// * `current_user` - The authenticated username
/// * `form` - Form data containing the new status
///
/// # Returns
/// * Redirect back to user's sheet list or error response
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
            Ok(data) => serde_json::from_str::<Vec<SheetEntry>>(&data).unwrap_or_default(),
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

/// Export the current spreadsheet as a binary file
///
/// Serializes the spreadsheet and returns it as a downloadable binary file.
///
/// # Arguments
/// * `state` - Application state containing the spreadsheet
///
/// # Returns
/// * Binary file for download or error response
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

/// Load a spreadsheet from an uploaded file
///
/// Processes a multipart form upload and loads the spreadsheet into memory.
///
/// # Arguments
/// * `state` - Application state
/// * `multipart` - Multipart form data containing the uploaded file
///
/// # Returns
/// * JSON response indicating success or failure
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

/// Download the current spreadsheet as CSV
///
/// Converts the spreadsheet to CSV format and returns it as a downloadable file.
///
/// # Arguments
/// * `state` - Application state containing the spreadsheet
///
/// # Returns
/// * CSV file for download or error response
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

/// Download the current spreadsheet as XLSX
///
/// Converts the spreadsheet to XLSX format and returns it as a downloadable file.
///
/// # Arguments
/// * `state` - Application state containing the spreadsheet
///
/// # Returns
/// * XLSX file for download or error response
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

/// Serialize a spreadsheet to a memory buffer
///
/// Compresses and serializes a spreadsheet to a memory buffer.
///
/// # Arguments
/// * `spreadsheet` - The spreadsheet to serialize
/// * `buffer` - The buffer to write to
///
/// # Returns
/// * `std::io::Result<()>` - Success or error
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

/// Deserialize a spreadsheet from a memory buffer
///
/// Decompresses and deserializes a spreadsheet from a memory buffer.
///
/// # Arguments
/// * `buffer` - The buffer containing the serialized spreadsheet
///
/// # Returns
/// * `std::io::Result<Box<Spreadsheet>>` - Deserialized spreadsheet or error
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

/// Convert a formula to a displayable string
///
/// Converts the internal formula representation to a string that can be displayed
/// in the UI or saved to a file.
///
/// # Arguments
/// * `formula` - The formula to convert
///
/// # Returns
/// * A string representation of the formula
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

/// Get information about the current spreadsheet
///
/// Returns metadata about the current spreadsheet, including whether it has been loaded
/// from a file and the original path if applicable.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * JSON response with spreadsheet information
async fn get_sheet_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let original_path = state.original_path.lock().unwrap();

    Json(serde_json::json!({
        "is_loaded": original_path.is_some(),
        "original_path": original_path.clone().unwrap_or_default(),
    }))
}

/// Get current sheet version and last modified timestamp
/// Clients can poll this endpoint to detect when they need to refresh
async fn get_sheet_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let version = *state.version.lock().unwrap();

    // Convert SystemTime to timestamp
    let last_modified = state
        .last_modified
        .lock()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Json(SheetStatusResponse {
        version,
        last_modified,
    })
}

/// Add an endpoint for undo
async fn undo(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut sheet = state.sheet.lock().unwrap();
    // debug before
    // println!(
    //     "DEBUG /undo handler: undo_stack length before undo: {}",
    //     sheet.undo_stack.len()
    // );

    let mut status = String::new();
    if sheet.undo_stack.is_empty() {
        status = String::from("no undo");
    } else {
        // perform undo
        sheet.spreadsheet_undo(&mut status);
        // println!("DEBUG /undo handler: performed undo, status: {}", status);
        // println!(
        //     "DEBUG /undo handler: undo_stack length after undo: {}",
        //     sheet.undo_stack.len()
        // );
    }

    // bump version & timestamp
    {
        let mut version = state.version.lock().unwrap();
        *version += 1;
        *state.last_modified.lock().unwrap() = std::time::SystemTime::now();
    }

    Json(serde_json::json!({
        "status": status,
        "version": *state.version.lock().unwrap()
    }))
}
