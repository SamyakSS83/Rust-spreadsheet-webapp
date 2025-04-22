/*!
# Spreadsheet Application

A browser-based spreadsheet application with Excel-like functionality, fully implemented in Rust.

This application is a complete rewrite of a legacy spreadsheet tool from C to Rust, enhanced with modern features and a browser GUI using WebAssembly. It features a modular, maintainable, and performant design with support for advanced operations like dependency-tracked formulas, graph plotting, and file sharing.

---

## 📦 Project Summary

- **Frontend**: HTML, CSS
- **Backend**: Rust (via `actix-web`)
- **Persistence**: Gzip + Bincode for `.bin.gz`; CSV/XLSX support
- **Graph Support**: Line, bar, area, scatter via `plotters` and `image`
- **Authentication**: Sessions, password reset, cookie-based auth
- **Key Features**: Copy/paste, undo/redo, formula evaluation, dependency graph, public/private sheet sharing

---

## 🏗️ Architecture

### Frontend Layer
- Written in HTML/CSS
- Renders a grid window of the custom size
- Formula bar component for inline editing
- Handles mouse, keyboard, and touchpad interactions
- Sends commands to backend via WebAssembly bindings

### Backend Layer
- Built using `actix-web`, `auxm` and `wasm-bindgen`
- Contains spreadsheet logic, cell storage, command processing
- Manages dependencies, error handling, and formula evaluation
- Ensures recalculations respect topological order
- Implements cell cycle detection and error propagation

### Data Persistence Layer
- Custom binary format with `.bin.gz` extension
- Serialization via `bincode` + `serde`, compression with `gzip`
- File versioning supports undo/redo state tracking
- CSV and XLSX export via `downloader` module

---

## 🔧 Modules

### `cell` Module
- Core `Cell` structure storing value, formula, dependents
- Methods for dependency management (insert, remove, check)
- Location tracking via row/column coordinates

### `spreadsheet` Module
- Main logic; manages grid, evaluation, updates
- Formula evaluation engine with function support
- Dependency tracking with cycle detection
- Topological sorting for correct update order
- Command processing (set cell, copy/paste of **values only**, undo/redo)
- Error handling and propagation

### `login` Module
- User registration, session validation, password reset
- Cookie-based authentication system
- User data persistence via JSON
- File access permission management

### `mailer` Module
- Sends password reset links via email
- Template-based email generation
- SMTP connection management

### `saving` Module
- Save/load logic with gzip+bincode
- File version management
- State tracking for undo/redo operations
- Compressed binary format for efficient storage

### `downloader` Module
- Data export in CSV/XLSX formats
- Format conversion utilities
- Download request handling

### `graph` Module
- Graph plotting via `plotters`
- Support for line, bar, area, scatter charts
- Customizable titles, labels, and dimensions
- Image data generation for browser display and saving via `image`

---

## 🌐 Webserver

### Public Endpoints
- **Authentication**: `/login`, `/signup`, `/logout`, `/reset-password`, `/forgot-password`, `/change-password`
- **Public Access**: `/:username/:sheet_name` for read-only sheet access
- **Data Operations**: `/api/update_cell`, `/api/save`, `/api/load`, `/api/graph`, `/api/export`
- **Downloads**: `/api/download/csv`, `/api/download/xlsx`
- **API Access**: `/api/sheet`, `/api/cell/:cell_name`, `/api/sheet_info` for read-only data
- **Static Content**: Static assets from `/static`

### Protected Endpoints
- **Sheet Management**: `/sheet` for editing UI
- **File Operations**: `/:username` (file listing), `/:username/create` (create new sheet),
  `/:username/:sheet_name/status` (update access), `/:username/:sheet_name/delete` (deletion)

### Middleware Logic
- Auth checks via `require_auth` middleware for protected endpoints
- Session map maintained for user state with expiration time
- Public/private file access logic via metadata in `list.json`
- Redirection to login page for unauthorized access attempts

---

## 🔍 Features

### Formula Support
- Basic operations: `+`, `-`, `*`, `/`
- Functions: `SUM`, `AVG`, `MAX`, `MIN`, `STDEV`, `SLEEP`, `COPY`, `UNDO`, `REDO`
- Formula processing with regex-based parsing
- One-time parsing optimization for performance

### Dependency Management
- Dependency graph with automated updating
- Cycle detection algorithm to prevent circular references
- Topological sort for ordered recalculation
- Error propagation through dependent cells

### User Interface
- Copy/paste support with range validation
- Undo through versioned states
- Navigation via keyboard/mouse/touchpad
- Formula bar for direct formula editing

### Error Handling
- Division by zero detection and propagation
- Syntax error identification in formulas
- Cycle detection with clear error messages
- Cascading errors through dependent cells

### Data Visualization
- Graph plotting with customizable options
- Support for multiple chart types
- Title and axis labeling
- Dimension configuration

### Sharing & Access Control
- Public/private spreadsheet access
- Spreadsheet sharing by URL
- User-based access control
- Metadata tracking of file ownership
- Real time updates for public sheets
- First access precedence updation conflict resolution

---

## 🔒 Security Model

- Input sanitization for formulas and commands
- Public/private access enforcement via metadata
- Session-based access control with expiration
- Authentication middleware for protected endpoints
- Cookie validation for session management

---

## 🧠 Key Design Decisions

### Data Structures
- `Spreadsheet`: Container for cells, viewport management, undo stack
- `Cell`: Value, formula, dependents, location storage
- `Formula` enum: Extensible function handling with variant types
- Hybrid approach for dependency tracking:
  - `Vec` for initial, small dependency lists
  - `OrderedSet` (AVL tree) for larger dependency collections
- Tuple indices `(row, col)` instead of strings for memory efficiency

### Optimizations
- `regex` for one-time formula parsing (no repeated parsing)
- `lazy_static!` for precompiled regexes
- Topological sort for dependency-ordered recalculation
- `u16` for index compression to save memory
- Stack-based recursion elimination
- Sparse cell storage to reduce memory footprint

### Communication Flow
1. User actions trigger WebAssembly functions
2. WebAssembly communicates with Rust backend
3. Backend processes commands and updates state
4. Dependency resolution triggers recalculation
5. Updated state returns to frontend for rendering
6. Changes persist to storage in compressed format

---

## 📉 Performance Optimizations

- Reduced memory footprint with sparse cell storage
- Use of `u16` for row/column indices instead of larger types
- Optimized dependency graph updates with topological ordering
- No redundant formula parsing (one-time evaluation)
- Precompiled regex + tuple indices for faster dependency lookups
- Profiling with `flamegraph` used to identify and improve bottlenecks
- Optimized memory allocation by reusing data structures

---

## ⚠️ Covered Edge Cases

### Circular Dependencies
- Direct cycles (e.g., A1 = B1, B1 = A1)
- Indirect cycles through multiple cells
- Self-referential cells (e.g., A1 = A1)
- Non-obvious cycles (e.g., A1 = 0*B1, B1 = A1)

### Formula Errors
- Invalid formula syntax (e.g., 1++1, unrecognized functions)
- Missing arguments (e.g., MAX(), SLEEP())
- Division by zero and error cascading
- Out-of-bounds cell references

### Sleep Functions
- Normal sleep operation
- Cascaded SLEEP cells
- Negative sleep values
- Error propagation through sleep functions

### General Edge Cases
- Empty/invalid commands
- Large scrolling behavior
- Dependency graph correctness
- Error propagation through complex dependencies

---

## 🧪 Primary Data Structures

- **Spreadsheet**: Core container storing cells, viewport, and undo stack
- **Cell**: Stores formula, value, dependents, and location
- **Formula (enum)**: Represents different formula types with variant data
- **Vector**: Used for temporary storage and initial dependency lists
- **OrderedSet (AVL Tree)**: For efficient storage of large dependency collections
- **Directed Acyclic Graph**: Models dependencies between cells
- **Linked List**: Used for topological sort ordering
- **Stack**: Implements undo/redo logic and eliminates recursion

---

## ➕ Future Enhancements

- Drag and drop support in GUI
- VLOOKUP/HLOOKUP and more Excel functions
- Multi-cell selection in GUI
- Floating-point formula support (e.g., `f32` vs `i32`)
- Further performance optimizations for large spreadsheets

---

## 🔗 GitHub Repository

[https://github.com/SamyakSS83/cop](https://github.com/SamyakSS83/cop)
*/

// Re-export all modules so they appear in the documentation
#[cfg(feature = "web")]
pub mod app;
pub mod cell;
pub mod downloader;
pub mod graph;
pub mod login;
pub mod mailer;
pub mod saving;
pub mod spreadsheet;

/// Re-export everything from these modules to make it easier to use
pub use cell::*;
pub use downloader::*;
pub use login::*;
pub use saving::*;
pub use spreadsheet::*;
