use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
#[cfg(feature = "web")]
use axum::extract::FromRef;
#[cfg(feature = "web")]
use axum::{
    Form, Json,
    extract::{Query, State, Path},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
#[cfg(feature = "web")]
use axum_extra::extract::cookie::{Cookie, CookieJar};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
#[cfg(feature = "web")]
use std::{fs, fs::create_dir_all, path::PathBuf};
#[cfg(feature = "web")]
use crate::saving;
#[cfg(feature = "web")]
use crate::spreadsheet::Spreadsheet;

// User data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserFile {
    pub name: String,
    pub path: String,
    pub created: SystemTime,
    pub modified: SystemTime,
}

// Session management
#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub expires_at: SystemTime,
}

lazy_static! {
    static ref SESSIONS: RwLock<HashMap<String, Session>> = RwLock::new(HashMap::new());
}

// Constants
const USERS_FILE: &str = "database/users.json";
const DATABASE_DIR: &str = "database";
const SESSION_DURATION: u64 = 24 * 60 * 60; // 24 hours in seconds

// Initialization function to ensure database structure exists
pub fn init_database() -> std::io::Result<()> {
    // Create database directory if it doesn't exist
    if !std::path::Path::new(DATABASE_DIR).exists() {
        create_dir_all(DATABASE_DIR)?;
    }

    // Create Admin folder
    let admin_dir = std::path::Path::new(DATABASE_DIR).join("Admin");
    if !admin_dir.exists() {
        create_dir_all(&admin_dir)?;
    }

    // Create users.json if it doesn't exist
    let users_path = std::path::Path::new(USERS_FILE);
    if !users_path.exists() {
        let mut file = File::create(users_path)?;
        file.write_all(b"{}")?;
    }

    Ok(())
}

// User management functions
pub fn get_users() -> Result<HashMap<String, User>, String> {
    let mut file = match File::open(USERS_FILE) {
        Ok(file) => file,
        Err(_) => return Err("Failed to open users file".to_string()),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err("Failed to read users file".to_string());
    }

    match serde_json::from_str(&contents) {
        Ok(users) => Ok(users),
        Err(_) => Err("Failed to parse users data".to_string()),
    }
}

pub fn save_users(users: &HashMap<String, User>) -> Result<(), String> {
    let json = match serde_json::to_string_pretty(users) {
        Ok(json) => json,
        Err(_) => return Err("Failed to serialize users data".to_string()),
    };

    let mut file = match File::create(USERS_FILE) {
        Ok(file) => file,
        Err(_) => return Err("Failed to create users file".to_string()),
    };

    if file.write_all(json.as_bytes()).is_err() {
        return Err("Failed to write users data".to_string());
    }

    Ok(())
}

// Register a new user
pub fn register_user(username: &str, password: &str) -> Result<(), String> {
    if username.is_empty() || password.is_empty() {
        return Err("Username and password cannot be empty".to_string());
    }

    // Check if username already exists
    let mut users = get_users()?;
    if users.contains_key(username) {
        return Err("Username already exists".to_string());
    }

    // Hash the password
    let password_hash = hash_password(password)?;

    // Create user directory
    let user_dir = std::path::Path::new(DATABASE_DIR).join(username);
    if create_dir_all(&user_dir).is_err() {
        return Err("Failed to create user directory".to_string());
    }

    // Add user to users.json
    let user = User {
        username: username.to_string(),
        password_hash,
    };

    users.insert(username.to_string(), user);
    save_users(&users)?;

    Ok(())
}

// Verify user credentials
pub fn verify_user(username: &str, password: &str) -> Result<bool, String> {
    let users = get_users()?;

    if let Some(user) = users.get(username) {
        verify_password(password, &user.password_hash)
    } else {
        Ok(false)
    }
}

// Password hashing functions
fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err("Password hashing failed".to_string()),
    }
}

fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(hash) => hash,
        Err(_) => return Err("Invalid password hash format".to_string()),
    };

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false), // Password didn't match
    }
}

// Session management
pub fn create_session(username: &str) -> String {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = SystemTime::now() + Duration::from_secs(SESSION_DURATION);

    let session = Session {
        user_id: username.to_string(),
        expires_at,
    };

    let mut sessions = SESSIONS.write().unwrap();
    sessions.insert(session_id.clone(), session);

    session_id
}

pub fn validate_session(session_id: &str) -> Option<String> {
    let sessions = SESSIONS.read().unwrap();

    if let Some(session) = sessions.get(session_id) {
        if session.expires_at > SystemTime::now() {
            return Some(session.user_id.clone());
        }
    }

    None
}

// File management
pub fn get_user_files(username: &str) -> Vec<UserFile> {
    let mut files = Vec::new();
    let user_dir = Path::new(DATABASE_DIR).join(username);

    if let Ok(entries) = fs::read_dir(user_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
                if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                    let metadata = match fs::metadata(&path) {
                        Ok(meta) => meta,
                        Err(_) => continue, // Skip this file if we can't get metadata
                    };
                    let created = metadata.created().unwrap_or(SystemTime::now());
                    let modified = metadata.modified().unwrap_or(SystemTime::now());

                    files.push(UserFile {
                        name: filename.to_string(),
                        path: path.to_string_lossy().to_string(),
                        created,
                        modified,
                    });
                }
            }
        }
    }

    files
}

// Axum handler functions - only compiled when "web" feature is enabled
#[cfg(feature = "web")]
pub async fn serve_login_page() -> Html<&'static str> {
    Html(include_str!("./static/login.html"))
}

#[cfg(feature = "web")]
pub async fn serve_signup_page() -> Html<&'static str> {
    Html(include_str!("./static/signup.html"))
}

#[cfg(feature = "web")]
#[axum::debug_handler]
pub async fn handle_login(jar: CookieJar, Form(credentials): Form<UserCredentials>) -> Response {
    match verify_user(&credentials.username, &credentials.password) {
        Ok(true) => {
            let session_id = create_session(&credentials.username);
            let cookie = Cookie::new("session", session_id);
            (
                jar.add(cookie),
                Redirect::to(&format!("/{}", credentials.username)),
            )
                .into_response()
        }
        Ok(false) => (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error").into_response(),
    }
}

#[cfg(feature = "web")]
pub async fn handle_signup(
    Form(credentials): Form<UserCredentials>,
) -> Result<Redirect, (StatusCode, String)> {
    match register_user(&credentials.username, &credentials.password) {
        Ok(_) => Ok(Redirect::to("/login?registered=true")),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[cfg(feature = "web")]
pub async fn handle_logout(jar: CookieJar) -> (CookieJar, Redirect) {
    // Remove session cookie
    let cookie = Cookie::new("session", "");

    (jar.add(cookie), Redirect::to("/login"))
}

// Middleware to check if user is authenticated
#[cfg(feature = "web")]
pub async fn require_auth(
    jar: CookieJar,
    mut request: axum::extract::Request, // Remove generic parameter B
    next: axum::middleware::Next,        // Remove generic parameter B
) -> Response {
    // Check for session cookie
    if let Some(session_cookie) = jar.get("session") {
        let session_id = session_cookie.value();

        // Validate the session
        if let Some(username) = validate_session(session_id) {
            // Add username to request extensions
            request.extensions_mut().insert(username);

            // Continue with the request
            return next.run(request).await;
        }
    }

    // No valid session found, redirect to login
    Redirect::to("/login").into_response()
}

#[cfg(feature = "web")]
pub async fn list_files(
    jar: CookieJar,
    Path(username): Path<String>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    // 1) auth check
    if let Some(cookie) = jar.get("session") {
        if let Some(current) = validate_session(cookie.value()) {
            if current == username {
                // 2) load list.json
                let user_dir = PathBuf::from(DATABASE_DIR).join(&username);
                let list_path = user_dir.join("list.json");
                let entries: Vec<SheetEntry> = if list_path.exists() {
                    let data = fs::read_to_string(&list_path).unwrap_or_default();
                    serde_json::from_str(&data).unwrap_or_default()
                } else {
                    Vec::new()
                };

                // 3) render HTML
                let mut html = format!(
                    "<!DOCTYPE html>
                    <html><head><title>{0}’s Sheets</title></head><body>
                    <h1>{0}’s Spreadsheets</h1>
                    <a href=\"/{0}/create\">📄 Create New Spreadsheet</a>
                    <div class=\"file-list\">",
                    username
                );

                for e in entries {
                    html.push_str(&format!(
                        "<div>
                            <a href=\"/{u}/{n}\">{n}</a> — {s}
                            <form style=\"display:inline\" method=\"post\" action=\"/{u}/{n}/delete\">
                              <button>🗑️ Delete</button>
                            </form>
                          </div>",
                        u = username,
                        n = e.name,
                        s = e.status,
                    ));
                }

                html.push_str("</div></body></html>");
                return Ok(Html(html));
            }
        }
    }
    Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
}

#[cfg(feature = "web")]
#[derive(Debug, Serialize, Deserialize)]
struct SheetEntry {
    name: String,
    status: String, // "public" or "private"
}

#[cfg(feature = "web")]
#[derive(Debug, Deserialize)]
pub struct CreateSheetForm {
    pub name: String,
    pub rows: u16,
    pub cols: u16,
    pub status: String,
}
