/*!
# Spreadsheet Application

A browser-based spreadsheet application with Excel-like functionality, built in Rust.

## Overview

This project is a migration of an existing spreadsheet application from C to Rust,
enhancing its functionality and integrating a browser-based GUI. The application provides
an intuitive interface with navigation, support for complex formulas, graph plotting,
and enhanced editing features such as copy and paste, formula management, and more.

## Architecture

The application follows a client-server architecture:

### Frontend Layer
- **Technologies**: HTML, CSS, WebAssembly
- **Key Components**:
  - Cell Grid Renderer - Displays the active 10×10 grid
  - Formula Bar Component - For formula entry and editing
  - Navigation - Keyboard/mouse/touchpad navigation
  - Event Handler - Processes user actions and communicates with backend

### Backend Layer
- **Technologies**: Rust, actix-web
- **Core Components**:
  - Cell Storage Engine - Maintains spreadsheet data structure (up to 999 rows × 18,278 columns)
  - Formula Evaluator - Processes formulas and functions
  - Dependency Graph - Tracks cell relationships for efficient recalculation
  - Recalculation Engine - Updates dependent cells when values change
  - Error Handler - Manages error conditions and circular references
  - Command Processor - Interprets user commands and delegates actions

### Data Persistence Layer
- File Storage with Gzip compression and bincode serialization
- Custom binary (.bin.gz) and CSV export/import
- Version tracking for undo/redo operations

## Key Features

- Formula support: Arithmetic operations, functions (SUM, MIN, MAX, AVG, STDEV)
- Copy/paste functionality
- Cell dependency tracking and circular reference detection
- Undo capability
- File operations (save, load, export)
- Graph plotting (line, bar, scatter, area)
- User authentication and session management
- Public/private sheet sharing

## Modules

- **cell**: Cell struct and functions (initialization, dependency management)
- **spreadsheet**: Core module for spreadsheet functionality (formula evaluation, error handling)
- **login**: User authentication and session management
- **mailer**: Password reset email functionality
- **saving**: Spreadsheet persistence with compression
- **downloader**: Export functionality (CSV, XLSX)
- **graph**: Graph generation from spreadsheet data
- **app**: Routing and middleware

## Design Highlights

- Object-oriented approach with encapsulated functionality
- Efficient formula parsing with regex
- Optimized dependency management (vector/ordered set hybrid)
- Topological sorting for efficient recalculation
- Memory-efficient data structures

## Usage

The application provides a web interface accessed through a browser, with:

- Cell editing and formula entry
- Navigation controls
- File operations menu
- Graph generation capabilities
- User authentication

## REST API Endpoints

- `/load/{filename}` - Retrieves spreadsheet data
- `/save/{filename}` - Persists current state
- `/edit/{cell}` - Modifies cell content
- `/copy/{range}`, `/paste/{cell}` - Clipboard operations
- `/formula/{string}` - Processes formula input
- `/release/{range}` - Reverts cell modifications
*/

// Re-export all modules so they appear in the documentation
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
pub use graph::*;
pub use login::*;
pub use mailer::*;
pub use saving::*;
pub use spreadsheet::*;
