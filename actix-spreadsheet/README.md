# Actix Spreadsheet Application

This project is a web-based spreadsheet application built using Actix Web in Rust. It allows users to create, load, save, and manipulate spreadsheets through a web interface.

## Features

- Create new spreadsheets with specified dimensions.
- Load existing spreadsheets from the server.
- Save spreadsheets to the server.
- Manipulate spreadsheet data, including formulas and cell values.
- User session management for personalized experiences.

## Project Structure

```
actix-spreadsheet
├── src
│   ├── main.rs              # Application entry point
│   ├── app.rs               # Application state and setup
│   ├── routes
│   │   ├── mod.rs           # Route module exports
│   │   ├── api.rs           # API endpoints for spreadsheet operations
│   │   └── pages.rs         # Page handlers 
│   ├── models
│   │   ├── mod.rs           # Models module exports
│   │   └── session.rs       # Session management
│   ├── spreadsheet          # Core spreadsheet logic
│   │   ├── mod.rs
│   │   ├── spreadsheet.rs   # Core spreadsheet functionality
│   │   └── cell.rs          # Cell definition and operations
│   └── saving.rs            # Save/load functionality
├── static                   # Static assets
│   ├── css
│   │   └── style.css        # CSS styles for the application
│   ├── js
│   │   ├── main.js          # Main JavaScript logic
│   │   └── spreadsheet.js    # Client-side spreadsheet UI
│   └── favicon.ico          # Favicon for the web application
├── templates                # HTML templates
│   ├── base.html            # Base HTML template
│   ├── index.html           # Main landing page
│   └── spreadsheet.html      # Spreadsheet interface template
├── Cargo.toml               # Rust project configuration
└── README.md                # Project documentation
```

## Getting Started

### Prerequisites

- Rust and Cargo installed on your machine. You can install them from [rustup.rs](https://rustup.rs/).

### Installation

1. Clone the repository:
   ```
   git clone <repository-url>
   cd actix-spreadsheet
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the application:
   ```
   cargo run
   ```

4. Open your web browser and navigate to `http://localhost:8080` to access the application.

## Usage

- Use the landing page to create a new spreadsheet or load an existing one.
- Interact with the spreadsheet interface to enter data and formulas.
- Save your work to persist changes.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for details.