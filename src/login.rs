#![cfg(not(tarpaulin_include))]

#[cfg(feature = "web")]
use crate::mailer::{Mailer, generate_reset_code};
#[cfg(feature = "web")]
use crate::saving;
#[cfg(feature = "web")]
use crate::spreadsheet::Spreadsheet;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
#[cfg(feature = "web")]
use axum::extract::FromRef;
#[cfg(feature = "web")]
use axum::{
    Form,
    Json,
    extract::{Path as AxumPath, Query, State}, // Rename to avoid conflict
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
use std::path::{Path, PathBuf}; // Keep this import
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
#[cfg(feature = "web")]
use urlencoding;
use uuid::Uuid;

// User data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    // #[cfg(feature = "web")]
    pub reset_code: Option<String>,
    // #[cfg(feature = "web")]
    pub reset_code_expires: Option<SystemTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    #[serde(default)]
    pub email: String,
    pub password: String,
}

#[cfg(feature = "web")]
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[cfg(feature = "web")]
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetConfirm {
    pub email: String,
    pub reset_code: String,
    pub new_password: String,
}

#[cfg(feature = "web")]
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordChangeRequest {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserFile {
    pub name: String,
    pub path: String,
    pub created: SystemTime,
    pub modified: SystemTime,
}

#[cfg(feature = "web")]
#[derive(Debug, Serialize, Deserialize)]
pub struct SheetEntry {
    pub name: String,
    pub status: String, // "public" or "private"
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
pub fn register_user(username: &str, email: &str, password: &str) -> Result<(), String> {
    if username.is_empty() || password.is_empty() || email.is_empty() {
        return Err("Username, email and password cannot be empty".to_string());
    }

    // Check if username already exists
    let mut users = get_users()?;
    if users.contains_key(username) {
        return Err("Username already exists".to_string());
    }

    // Check if email is already in use
    if users.values().any(|user| user.email == email) {
        return Err("Email address is already registered".to_string());
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
        email: email.to_string(),
        password_hash,
        reset_code: None,
        reset_code_expires: None,
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
    // We don't need email for login
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
    match register_user(
        &credentials.username,
        &credentials.email,
        &credentials.password,
    ) {
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
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    // eprintln!("DEBUG: require_auth called. Request URI: {:?}", request.uri());

    // First, if a valid session exists, allow the request.
    if let Some(session_cookie) = jar.get("session") {
        // eprintln!("DEBUG: Found session cookie: {:?}", session_cookie);
        if let Some(username) = validate_session(session_cookie.value()) {
            // eprintln!("DEBUG: Valid session for user: {}", username);
            request.extensions_mut().insert(username);
            return next.run(request).await;
        } else {
            // eprintln!("DEBUG: Session cookie invalid or expired.");
        }
    } else {
        // eprintln!("DEBUG: No session cookie found.");
    }

    // No valid session; if the call is for an API endpoint, check if the sheet is public.
    let uri = request.uri().path();
    // eprintln!("DEBUG: Processing URI: {}", uri);
    if uri.starts_with("/api/") {
        let parts: Vec<&str> = uri.split('/').filter(|s| !s.is_empty()).collect();
        let (owner, sheet_name) = if parts.len() >= 3 {
            (
                parts[1].to_string(),
                parts[2].trim_end_matches(".bin.gz").to_string(),
            )
        } else {
            (String::new(), String::new())
        };
        // eprintln!("DEBUG: Parsed owner: '{}', sheet_name: '{}'", owner, sheet_name);

        // NEW: If there's an authenticated user matching the owner, allow access.
        if let Some(auth_user) = request.extensions().get::<String>() {
            // eprintln!("DEBUG: Found authenticated user in extensions: {}", auth_user);
            if *auth_user == owner {
                // eprintln!("DEBUG: Authenticated user matches owner. Allowing access.");
                return next.run(request).await;
            }
        } else {
            // eprintln!("DEBUG: No authenticated user in extensions.");
        }

        if !owner.is_empty() && !sheet_name.is_empty() {
            let list_path = format!("database/{}/list.json", owner);
            // eprintln!("DEBUG: Checking public status from list at path: {}", list_path);
            if let Ok(data) = std::fs::read_to_string(&list_path) {
                // eprintln!("DEBUG: Read list.json: {}", data);
                if let Ok(entries) = serde_json::from_str::<Vec<crate::login::SheetEntry>>(&data) {
                    // eprintln!("DEBUG: Parsed {} entries", entries.len());
                    let is_public = entries.iter().any(|entry| {
                        let condition = entry.name == sheet_name && entry.status == "public";
                        // eprintln!("DEBUG: Checking entry: {:?} -> {}", entry, condition);
                        condition
                    });
                    // eprintln!("DEBUG: is_public: {}", is_public);
                    if is_public {
                        return next.run(request).await;
                    }
                } else {
                    // eprintln!("DEBUG: Failed to parse list.json");
                }
            } else {
                // eprintln!("DEBUG: Failed to read list.json from path: {}", list_path);
            }
        }
    }
    // eprintln!("DEBUG: Access denied. Redirecting to /login");
    // Failing the above, redirect to login.
    Redirect::to("/login").into_response()
}

#[cfg(feature = "web")]
pub async fn list_files(
    jar: CookieJar,
    AxumPath(username): AxumPath<String>,
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

                // 3) Get the template and inject the data
                let mut template = include_str!("./static/list.html").to_string();

                // Insert the sheets data as JavaScript
                let sheets_json =
                    serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());

                template = template.replace(
                    "</head>",
                    &format!(
                        "    <script>const SHEETS_DATA = {};</script>\n</head>",
                        sheets_json
                    ),
                );

                return Ok(Html(template));
            }
        }
    }
    Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
}

#[cfg(feature = "web")]
#[derive(Debug, Deserialize)]
pub struct CreateSheetForm {
    pub name: String,
    pub rows: u16,
    pub cols: u16,
    pub status: String,
}

#[cfg(feature = "web")]
pub async fn serve_create_sheet_form(
    jar: CookieJar,
    AxumPath(username): AxumPath<String>,
) -> Result<Redirect, (StatusCode, &'static str)> {
    // Redirect back to the list page - the form is now in the modal
    Ok(Redirect::to(&format!("/{}", username)))
}

#[cfg(feature = "web")]
pub async fn handle_create_sheet(
    jar: CookieJar,
    AxumPath(username): AxumPath<String>,
    Form(form): Form<CreateSheetForm>,
) -> Redirect {
    // 1) Create the directory if it doesn't exist
    let user_dir = PathBuf::from(DATABASE_DIR).join(&username);
    let _ = create_dir_all(&user_dir);

    // 2) Create and save the spreadsheet
    let filename = format!("{}.bin.gz", form.name);
    let path = user_dir.join(&filename);
    let sheet = Spreadsheet::spreadsheet_create(form.rows as i16, form.cols as i16)
        .expect("Failed to create spreadsheet");
    saving::save_spreadsheet(&sheet, path.to_str().unwrap()).expect("Failed to save spreadsheet");

    // 3) Update list.json
    let list_path = user_dir.join("list.json");
    let mut entries: Vec<SheetEntry> = if list_path.exists() {
        let data = fs::read_to_string(&list_path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };
    entries.push(SheetEntry {
        name: form.name,
        status: form.status,
    });
    fs::write(&list_path, serde_json::to_string_pretty(&entries).unwrap())
        .expect("Failed to write list.json");

    Redirect::to(&format!("/{}", username))
}

#[cfg(feature = "web")]
pub async fn handle_delete_sheet(
    jar: CookieJar,
    AxumPath((username, sheet_name)): AxumPath<(String, String)>,
) -> Redirect {
    // 1) Delete the spreadsheet file
    let user_dir = PathBuf::from(DATABASE_DIR).join(&username);
    let file_path = user_dir.join(format!("{}.bin.gz", sheet_name));
    let _ = fs::remove_file(&file_path);

    // 2) Update list.json
    let list_path = user_dir.join("list.json");
    if list_path.exists() {
        if let Ok(data) = fs::read_to_string(&list_path) {
            if let Ok(mut entries) = serde_json::from_str::<Vec<SheetEntry>>(&data) {
                entries.retain(|entry| entry.name != sheet_name);
                let _ = fs::write(&list_path, serde_json::to_string_pretty(&entries).unwrap());
            }
        }
    }

    Redirect::to(&format!("/{}", username))
}

// #[cfg(feature = "web")]
// pub async fn handle_forgot_password(
//     Form(reset_req): Form<PasswordResetRequest>,
// ) -> impl IntoResponse {
//     let mut users = match get_users() {
//         Ok(users) => users,
//         Err(_) => {
//             return (StatusCode::INTERNAL_SERVER_ERROR, "Server error").into_response();
//         }
//     };

//     // Find user by email
//     let user = users.values_mut().find(|u| u.email == reset_req.email);

//     if let Some(user) = user {
//         let reset_code = generate_reset_code();
//         let expires = SystemTime::now() + Duration::from_secs(3600); // 1 hour

//         // Update user with reset code
//         user.reset_code = Some(reset_code.clone());
//         user.reset_code_expires = Some(expires);

//         // Save updated users
//         if save_users(&users).is_err() {
//             return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save reset code").into_response();
//         }

//         // Send email
//         match Mailer::new() {
//             Ok(mailer) => {
//                 if let Err(_) = mailer.send_password_reset(&reset_req.email, &reset_code) {
//                     return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send email").into_response();
//                 }
//             }
//             Err(_) => {
//                 return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to initialize mailer").into_response();
//             }
//         }

//         (StatusCode::OK, "Password reset email sent").into_response()
//     } else {
//         (StatusCode::NOT_FOUND, "Email not found").into_response()
//     }
// }

#[cfg(feature = "web")]
pub async fn handle_forgot_password(
    Form(reset_req): Form<PasswordResetRequest>,
) -> impl IntoResponse {
    let mut users = match get_users() {
        Ok(users) => users,
        Err(_) => {
            return Redirect::to("/forgot-password?error=Server+error").into_response();
        }
    };

    // Find user by email
    let user = users.values_mut().find(|u| u.email == reset_req.email);

    if let Some(user) = user {
        let reset_code = generate_reset_code();
        let expires = SystemTime::now() + Duration::from_secs(3600); // 1 hour

        // Update user with reset code
        user.reset_code = Some(reset_code.clone());
        user.reset_code_expires = Some(expires);

        // Save updated users
        if save_users(&users).is_err() {
            return Redirect::to("/forgot-password?error=Failed+to+generate+reset+code")
                .into_response();
        }

        // Send email
        match Mailer::new() {
            Ok(mailer) => {
                if let Err(_) = mailer.send_password_reset(&reset_req.email, &reset_code) {
                    return Redirect::to("/forgot-password?error=Failed+to+send+email")
                        .into_response();
                }
            }
            Err(_) => {
                return Redirect::to("/forgot-password?error=Failed+to+initialize+mailer")
                    .into_response();
            }
        }

        // Redirect to reset form with success message
        Redirect::to(&format!(
            "/reset-password?email_sent=true&email={}",
            urlencoding::encode(&reset_req.email)
        ))
        .into_response()
    } else {
        Redirect::to("/forgot-password?error=Email+not+found").into_response()
    }
}

#[cfg(feature = "web")]
pub async fn handle_reset_password(
    Form(reset_confirm): Form<PasswordResetConfirm>,
) -> impl IntoResponse {
    let mut users = match get_users() {
        Ok(users) => users,
        Err(_) => {
            return Redirect::to("/reset-password?error=Server+error").into_response();
        }
    };

    // Find user by email
    let user = users.values_mut().find(|u| u.email == reset_confirm.email);

    if let Some(user) = user {
        // Verify reset code
        if let Some(stored_code) = &user.reset_code {
            if let Some(expires) = user.reset_code_expires {
                if SystemTime::now() > expires {
                    return Redirect::to("/reset-password?error=Reset+code+expired")
                        .into_response();
                }

                if stored_code != &reset_confirm.reset_code {
                    return Redirect::to("/reset-password?error=Invalid+reset+code")
                        .into_response();
                }

                // Update password
                match hash_password(&reset_confirm.new_password) {
                    Ok(hash) => {
                        user.password_hash = hash;
                        user.reset_code = None;
                        user.reset_code_expires = None;

                        if save_users(&users).is_err() {
                            return Redirect::to(
                                "/reset-password?error=Failed+to+save+new+password",
                            )
                            .into_response();
                        }

                        Redirect::to("/login?success=Password+reset+successful").into_response()
                    }
                    Err(_) => Redirect::to("/reset-password?error=Failed+to+hash+password")
                        .into_response(),
                }
            } else {
                Redirect::to("/reset-password?error=Reset+code+expired").into_response()
            }
        } else {
            Redirect::to("/reset-password?error=No+reset+code+found").into_response()
        }
    } else {
        Redirect::to("/reset-password?error=Email+not+found").into_response()
    }
}

// #[cfg(feature = "web")]
// pub async fn handle_reset_password(
//     Form(reset_confirm): Form<PasswordResetConfirm>,
// ) -> impl IntoResponse {
//     let mut users = match get_users() {
//         Ok(users) => users,
//         Err(_) => {
//             return (StatusCode::INTERNAL_SERVER_ERROR, "Server error").into_response();
//         }
//     };

//     // Find user by email
//     let user = users.values_mut().find(|u| u.email == reset_confirm.email);

//     if let Some(user) = user {
//         // Verify reset code
//         if let Some(stored_code) = &user.reset_code {
//             if let Some(expires) = user.reset_code_expires {
//                 if SystemTime::now() > expires {
//                     return (StatusCode::BAD_REQUEST, "Reset code expired").into_response();
//                 }

//                 if stored_code != &reset_confirm.reset_code {
//                     return (StatusCode::BAD_REQUEST, "Invalid reset code").into_response();
//                 }

//                 // Update password
//                 match hash_password(&reset_confirm.new_password) {
//                     Ok(hash) => {
//                         user.password_hash = hash;
//                         user.reset_code = None;
//                         user.reset_code_expires = None;

//                         if save_users(&users).is_err() {
//                             return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save new password").into_response();
//                         }

//                         (StatusCode::OK, "Password reset successful").into_response()
//                     }
//                     Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
//                 }
//             } else {
//                 (StatusCode::BAD_REQUEST, "Reset code expired").into_response()
//             }
//         } else {
//             (StatusCode::BAD_REQUEST, "No reset code found").into_response()
//         }
//     } else {
//         (StatusCode::NOT_FOUND, "Email not found").into_response()
//     }
// }

#[cfg(feature = "web")]
pub async fn handle_change_password(
    jar: CookieJar,
    Form(change_req): Form<PasswordChangeRequest>,
) -> impl IntoResponse {
    // Verify current user is authenticated
    if let Some(cookie) = jar.get("session") {
        if let Some(current_user) = validate_session(cookie.value()) {
            if current_user != change_req.username {
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }

            // Verify old password and update to new password
            let mut users = match get_users() {
                Ok(users) => users,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Server error").into_response();
                }
            };

            if let Some(user) = users.get_mut(&change_req.username) {
                // Verify old password
                match verify_password(&change_req.old_password, &user.password_hash) {
                    Ok(true) => {
                        // Verify new passwords match
                        if change_req.new_password != change_req.confirm_password {
                            return (StatusCode::BAD_REQUEST, "New passwords don't match")
                                .into_response();
                        }

                        // Update password
                        match hash_password(&change_req.new_password) {
                            Ok(hash) => {
                                user.password_hash = hash;
                                if save_users(&users).is_err() {
                                    return (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        "Failed to save new password",
                                    )
                                        .into_response();
                                }
                                (StatusCode::OK, "Password changed successfully").into_response()
                            }
                            Err(_) => {
                                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password")
                                    .into_response()
                            }
                        }
                    }
                    Ok(false) => (StatusCode::BAD_REQUEST, "Invalid old password").into_response(),
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Password verification failed",
                    )
                        .into_response(),
                }
            } else {
                (StatusCode::NOT_FOUND, "User not found").into_response()
            }
        } else {
            (StatusCode::UNAUTHORIZED, "Invalid session").into_response()
        }
    } else {
        (StatusCode::UNAUTHORIZED, "No session found").into_response()
    }
}

#[cfg(feature = "web")]
pub async fn serve_forgot_password_page() -> Html<&'static str> {
    Html(include_str!("./static/password.html"))
}

#[cfg(feature = "web")]
pub async fn serve_reset_password_page() -> Html<&'static str> {
    Html(include_str!("./static/password.html"))
}

#[cfg(feature = "web")]
pub async fn serve_change_password_page() -> Html<&'static str> {
    Html(include_str!("./static/password.html"))
}
