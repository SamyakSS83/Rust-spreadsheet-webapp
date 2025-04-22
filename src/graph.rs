#![cfg(not(tarpaulin_include))]
#![cfg(feature = "web")]
use crate::spreadsheet::Spreadsheet;
use plotters::prelude::*;
use std::fs::remove_file;

/// Available graph types supported by the application
///
/// This enum defines the different visualization formats that can be generated
/// from spreadsheet data.
#[derive(Clone, Debug)]
pub enum GraphType {
    /// Line graph - Shows trends over time or continuous data with lines
    /// connecting data points
    Line,

    /// Bar graph - Compares values across different categories with rectangular bars
    Bar,

    /// Scatter plot - Shows the relationship between two variables with points
    Scatter,

    /// Area graph - Similar to line graphs but with the area below the line filled in
    Area,
}

/// Configuration options for graph generation
///
/// This structure contains all the customizable properties for generating
/// different types of graphs.
#[derive(Clone, Debug)]
pub struct GraphOptions {
    /// Title displayed at the top of the graph
    pub title: String,

    /// Label for the X-axis
    pub x_label: String,

    /// Label for the Y-axis
    pub y_label: String,

    /// Width of the graph in pixels
    pub width: u32,

    /// Height of the graph in pixels
    pub height: u32,

    /// Type of graph to generate
    pub graph_type: GraphType,
}

impl Default for GraphOptions {
    /// Creates a default configuration for graph generation
    ///
    /// # Returns
    /// * `GraphOptions` - Default configuration with:
    ///   - Line graph type
    ///   - 800x600 pixel dimensions
    ///   - Generic labels
    fn default() -> Self {
        Self {
            title: "Graph".to_string(),
            x_label: "X Axis".to_string(),
            y_label: "Y Axis".to_string(),
            width: 800,
            height: 600,
            graph_type: GraphType::Line,
        }
    }
}

/// Creates a graph from spreadsheet data
///
/// This is the main entry point for generating graphs from spreadsheet data.
/// It parses cell ranges, extracts data, and delegates to the appropriate graph type generator.
///
/// # Arguments
/// * `spreadsheet` - Reference to the spreadsheet containing the data
/// * `x_range` - Range for X values (e.g., "A1:A10")
/// * `y_range` - Range for Y values (e.g., "B1:B10")
/// * `options` - Graph styling and type options
///
/// # Returns
/// * A Result containing the PNG image data as bytes or an error
///
/// # Examples
/// ```
/// use cop::spreadsheet::Spreadsheet;
/// use cop::graph::{GraphOptions, GraphType, create_graph};
///
/// let spreadsheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
///
/// let options = GraphOptions {
///     title: "Sample Graph".to_string(),
///     x_label: "X Values".to_string(),
///     y_label: "Y Values".to_string(),
///     width: 800,
///     height: 600,
///     graph_type: GraphType::Line,
/// };
///
/// match create_graph(&spreadsheet, "A1:A5", "B1:B5", options) {
///     Ok(png_data) => println!("Graph created successfully: {} bytes", png_data.len()),
///     Err(e) => eprintln!("Failed to create graph: {}", e),
/// }
/// ```
pub fn create_graph(
    spreadsheet: &Spreadsheet,
    x_range: &str,
    y_range: &str,
    options: GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse the ranges to get the actual cells
    let (x_cells, y_cells) = parse_ranges(spreadsheet, x_range, y_range)?;

    // Extract the data from the cells
    let data: Vec<(i32, i32)> = x_cells
        .iter()
        .zip(y_cells.iter())
        .map(|(x, y)| (*x, *y))
        .collect();

    match options.graph_type {
        GraphType::Line => create_line_graph(data, &options),
        GraphType::Bar => create_bar_graph(data, &options),
        GraphType::Scatter => create_scatter_graph(data, &options),
        // GraphType::Pie => create_pie_graph(data, &options),
        GraphType::Area => create_area_graph(data, &options),
    }
}

/// Parses the range strings and returns the cell values
///
/// This function extracts numerical data from spreadsheet ranges for graphing purposes.
/// It supports both column ranges (A1:A10) and row ranges (A1:J1).
///
/// # Arguments
/// * `spreadsheet` - Reference to the spreadsheet to extract data from
/// * `x_range` - Range specification for X values (e.g., "A1:A10")
/// * `y_range` - Range specification for Y values (e.g., "B1:B10")
///
/// # Returns
/// * A Result containing vectors of extracted X and Y values or an error
///
/// # Errors
/// * Returns an error if the range format is invalid
/// * Returns an error if the ranges have different lengths
///
/// # Notes
/// * Empty cells in the range will be treated as having a value of 0
fn parse_ranges(
    spreadsheet: &Spreadsheet,
    x_range: &str,
    y_range: &str,
) -> Result<(Vec<i32>, Vec<i32>), Box<dyn std::error::Error>> {
    // Split the range at the colon
    let x_parts: Vec<&str> = x_range.split(':').collect();
    let y_parts: Vec<&str> = y_range.split(':').collect();

    if x_parts.len() != 2 || y_parts.len() != 2 {
        return Err("Invalid range format. Expected format: A1:A10".into());
    }

    // Parse the cell coordinates
    let (x_start_row, x_start_col) = match spreadsheet.spreadsheet_parse_cell_name(x_parts[0]) {
        Some(coords) => coords,
        None => return Err("Invalid start cell in X range".into()),
    };

    let (x_end_row, x_end_col) = match spreadsheet.spreadsheet_parse_cell_name(x_parts[1]) {
        Some(coords) => coords,
        None => return Err("Invalid end cell in X range".into()),
    };

    let (y_start_row, y_start_col) = match spreadsheet.spreadsheet_parse_cell_name(y_parts[0]) {
        Some(coords) => coords,
        None => return Err("Invalid start cell in Y range".into()),
    };

    let (y_end_row, y_end_col) = match spreadsheet.spreadsheet_parse_cell_name(y_parts[1]) {
        Some(coords) => coords,
        None => return Err("Invalid end cell in Y range".into()),
    };

    // Ensure the ranges have the same length
    let x_len = if x_start_col == x_end_col {
        (x_end_row - x_start_row + 1) as usize
    } else {
        (x_end_col - x_start_col + 1) as usize
    };

    let y_len = if y_start_col == y_end_col {
        (y_end_row - y_start_row + 1) as usize
    } else {
        (y_end_col - y_start_col + 1) as usize
    };

    if x_len != y_len {
        return Err("X and Y ranges must have the same length".into());
    }

    // Extract values
    let mut x_values = Vec::with_capacity(x_len);
    let mut y_values = Vec::with_capacity(y_len);

    // Handle vertical ranges (same column)
    if x_start_col == x_end_col {
        for row in x_start_row..=x_end_row {
            let index = ((row - 1) * spreadsheet.cols + (x_start_col - 1)) as usize;
            if let Some(cell) = &spreadsheet.cells[index] {
                x_values.push(cell.value);
            } else {
                x_values.push(0);
            }
        }
    } else {
        // Handle horizontal ranges (same row)
        for col in x_start_col..=x_end_col {
            let index = ((x_start_row - 1) * spreadsheet.cols + (col - 1)) as usize;
            if let Some(cell) = &spreadsheet.cells[index] {
                x_values.push(cell.value);
            } else {
                x_values.push(0);
            }
        }
    }

    // Do the same for Y values
    if y_start_col == y_end_col {
        for row in y_start_row..=y_end_row {
            let index = ((row - 1) * spreadsheet.cols + (y_start_col - 1)) as usize;
            if let Some(cell) = &spreadsheet.cells[index] {
                y_values.push(cell.value);
            } else {
                y_values.push(0);
            }
        }
    } else {
        for col in y_start_col..=y_end_col {
            let index = ((y_start_row - 1) * spreadsheet.cols + (col - 1)) as usize;
            if let Some(cell) = &spreadsheet.cells[index] {
                y_values.push(cell.value);
            } else {
                y_values.push(0);
            }
        }
    }

    Ok((x_values, y_values))
}

/// Creates a line graph from data points
///
/// Generates a line graph showing the trend between X and Y values with connected lines.
/// Line graphs are ideal for showing trends over continuous data.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
///
/// # Returns
/// * A Result containing the PNG image data as bytes or an error
///
/// # Implementation Notes
/// * Creates a temporary file to store the image before reading it back
/// * Automatically scales axes based on data range
/// * Uses blue color for the line series
fn create_line_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a temporary file-based bitmap solution
    let filename = "temp_graph.png";
    {
        // Create a file-based bitmap backend
        let root =
            BitMapBackend::new(filename, (options.width, options.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
        let min_y = data.iter().map(|(_, y)| y).min().unwrap_or(&0);
        let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

        let x_range = *min_x as f64..*max_x as f64 + 1.0;
        let y_range = *min_y as f64..*max_y as f64 + 1.0;

        let mut chart = ChartBuilder::on(&root)
            .caption(&options.title, ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .x_desc(&options.x_label)
            .y_desc(&options.y_label)
            .draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().map(|&(x, y)| (x as f64, y as f64)),
            &BLUE,
        ))?;

        root.present()?;
    }

    // Read the file directly
    let mut file = std::fs::File::open(filename)?;
    let mut buffer = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut buffer)?;
    remove_file(filename)?;
    Ok(buffer)
}

/// Saves a line graph to a file
///
/// Creates a line graph and saves it directly to the specified file path.
/// Useful for generating examples or saving graphs without returning the image data.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
/// * `path` - File path where the graph should be saved
///
/// # Returns
/// * A Result indicating success or failure
fn save_line_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (options.width, options.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
    let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
    let min_y = data.iter().map(|(_, y)| y).min().unwrap_or(&0);
    let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

    let x_range = *min_x as f64..*max_x as f64 + 1.0;
    let y_range = *min_y as f64..*max_y as f64 + 1.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(&options.title, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .x_desc(&options.x_label)
        .y_desc(&options.y_label)
        .draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().map(|&(x, y)| (x as f64, y as f64)),
        &RED,
    ))?;

    root.present()?;

    Ok(())
}

/// Creates a bar graph from data points
///
/// Generates a bar graph showing values as vertical bars.
/// Bar graphs are ideal for comparing values across different categories.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points, where x is the category position and y is the value
/// * `options` - Graph styling options
///
/// # Returns
/// * A Result containing the PNG image data as bytes or an error
///
/// # Implementation Notes
/// * Creates a temporary file to store the image before reading it back
/// * Uses blue color for bars with solid fill
/// * Each x-value positions a bar with the height of the corresponding y-value
fn create_bar_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a temporary file-based bitmap solution
    let filename = "temp_graph.png";
    {
        // Create a file-based bitmap backend
        let root =
            BitMapBackend::new(filename, (options.width, options.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
        let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

        let x_range = *min_x..*max_x + 1;
        let y_range = 0..*max_y + 1;

        let mut chart = ChartBuilder::on(&root)
            .caption(&options.title, ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .x_desc(&options.x_label)
            .y_desc(&options.y_label)
            .draw()?;

        // Draw wider bars with solid fill and clear borders
        chart.draw_series(
            data.iter()
                .map(|&(x, y)| Rectangle::new([(x - 2, 0), (x + 2, y)], BLUE.filled())),
        )?;

        root.present()?;
    }

    // Read the file directly
    let png_data = std::fs::read(filename)?;

    // Clean up
    remove_file(filename)?;

    Ok(png_data)
}

/// Saves a bar graph to a file
///
/// Creates a bar graph and saves it directly to the specified file path.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
/// * `path` - File path where the graph should be saved
///
/// # Returns
/// * A Result indicating success or failure
///
/// # Implementation Notes
/// * Bars are sized based on the x value - adjacent x values will create adjacent bars
/// * Bar width is set to 0.8 units (from x-0.4 to x+0.4) for visual clarity
fn save_bar_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (options.width, options.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
    let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
    let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

    let x_range = *min_x..*max_x + 1;
    let y_range = 0..*max_y + 1;

    let mut chart = ChartBuilder::on(&root)
        .caption(&options.title, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range, y_range)?;

    chart.draw_series(data.iter().map(|&(x, y)| {
        Rectangle::new(
            [((x as f64 - 0.4) as i32, 0), ((x as f64 + 0.4) as i32, y)],
            BLUE.filled(),
        )
    }))?;

    root.present()?;

    Ok(())
}

/// Creates a scatter plot from data points
///
/// Generates a scatter plot showing individual data points without connecting lines.
/// Scatter plots are ideal for visualizing the relationship between two variables.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
///
/// # Returns
/// * A Result containing the PNG image data as bytes or an error
///
/// # Implementation Notes
/// * Creates a temporary file to store the image before reading it back
/// * Uses green circles with 5-pixel radius for data points
/// * Automatically scales axes based on data range
fn create_scatter_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a temporary file-based bitmap solution
    let filename = "temp_graph.png";
    {
        // Create a file-based bitmap backend
        let root =
            BitMapBackend::new(filename, (options.width, options.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
        let min_y = data.iter().map(|(_, y)| y).min().unwrap_or(&0);
        let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

        let x_range = *min_x as f64..*max_x as f64 + 1.0;
        let y_range = *min_y as f64..*max_y as f64 + 1.0;

        let mut chart = ChartBuilder::on(&root)
            .caption(&options.title, ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .x_desc(&options.x_label)
            .y_desc(&options.y_label)
            .draw()?;

        chart.draw_series(
            data.iter()
                .map(|&(x, y)| Circle::new((x as f64, y as f64), 5, GREEN.filled())),
        )?;

        root.present()?;
    }

    // Read the file directly
    let png_data = std::fs::read(filename)?;

    // Clean up
    remove_file(filename)?;

    Ok(png_data)
}

/// Saves a scatter plot to a file
///
/// Creates a scatter plot and saves it directly to the specified file path.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
/// * `path` - File path where the graph should be saved
///
/// # Returns
/// * A Result indicating success or failure
fn save_scatter_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (options.width, options.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
    let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
    let min_y = data.iter().map(|(_, y)| y).min().unwrap_or(&0);
    let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

    let x_range = *min_x as f64..*max_x as f64 + 1.0;
    let y_range = *min_y as f64..*max_y as f64 + 1.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(&options.title, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .x_desc(&options.x_label)
        .y_desc(&options.y_label)
        .draw()?;

    chart.draw_series(
        data.iter()
            .map(|&(x, y)| Circle::new((x as f64, y as f64), 5, GREEN.filled())),
    )?;

    root.present()?;

    Ok(())
}

/// Creates an area graph from data points
///
/// Generates an area graph showing values with the area under the line filled in.
/// Area graphs are good for emphasizing the magnitude of changes over time.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
///
/// # Returns
/// * A Result containing the PNG image data as bytes or an error
///
/// # Implementation Notes
/// * Creates a temporary file to store the image before reading it back
/// * Uses semi-transparent blue for the area fill
/// * Sorts data points by x-value to ensure proper area filling
/// * Area is filled between the line and y=0
fn create_area_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a temporary file-based bitmap solution
    let filename = "temp_graph.png";
    {
        // Create a file-based bitmap backend
        let root =
            BitMapBackend::new(filename, (options.width, options.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
        let min_y = data.iter().map(|(_, y)| y).min().unwrap_or(&0).min(&0); // Ensure we include 0
        let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

        let x_range = *min_x as f64..*max_x as f64 + 1.0;
        let y_range = *min_y as f64..*max_y as f64 + 1.0;

        let mut chart = ChartBuilder::on(&root)
            .caption(&options.title, ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(x_range.clone(), y_range.clone())?;

        chart
            .configure_mesh()
            .x_desc(&options.x_label)
            .y_desc(&options.y_label)
            .draw()?;

        // Sort data by x to ensure proper area graph
        let mut sorted_data = data.clone();
        sorted_data.sort_by_key(|&(x, _)| x);

        use plotters::series::AreaSeries;
        use plotters::style::RGBAColor;

        // Draw the area graph
        chart.draw_series(AreaSeries::new(
            sorted_data.iter().map(|&(x, y)| (x as f64, y as f64)),
            0.0,
            RGBAColor(30, 144, 255, 0.5), // semi-transparent blue
        ))?;

        root.present()?;
    }

    // Read the file directly
    let mut file = std::fs::File::open(filename)?;
    let mut buffer = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut buffer)?;
    remove_file(filename)?;
    Ok(buffer)
}

/// Saves an area graph to a file
///
/// Creates an area graph and saves it directly to the specified file path.
///
/// # Arguments
/// * `data` - Vector of (x,y) data points
/// * `options` - Graph styling options
/// * `path` - File path where the graph should be saved
///
/// # Returns
/// * A Result indicating success or failure
///
/// # Implementation Notes
/// * Sorts data by x-value to ensure proper area filling
/// * Uses 20% opacity blue fill with blue border
fn save_area_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (options.width, options.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
    let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
    let min_y = data.iter().map(|(_, y)| y).min().unwrap_or(&0).min(&0); // Ensure we include 0
    let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

    let x_range = *min_x as f64..*max_x as f64 + 1.0;
    let y_range = *min_y as f64..*max_y as f64 + 1.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(&options.title, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .x_desc(&options.x_label)
        .y_desc(&options.y_label)
        .draw()?;

    // Sort data by x to ensure proper area graph
    let mut sorted_data = data.clone();
    sorted_data.sort_by_key(|&(x, _)| x);

    chart.draw_series(
        AreaSeries::new(
            sorted_data.iter().map(|&(x, y)| (x as f64, y as f64)),
            0.0,
            BLUE.mix(0.2),
        )
        .border_style(BLUE),
    )?;

    root.present()?;

    Ok(())
}

/// Creates example graphs for demonstration purposes
///
/// Generates a set of example graphs (line, bar, scatter, area) using sample data
/// and saves them to disk in the "graph_output" directory.
///
/// # Returns
/// * A vector of tuples containing graph type name and file path
///
/// # Examples
/// ```
/// use cop::graph::create_example_graphs;
///
/// let examples = create_example_graphs();
/// for (graph_type, path) in examples {
///     println!("Created {} graph at {}", graph_type, path);
/// }
/// ```
///
/// # Implementation Notes
/// * Creates a directory "graph_output" if it doesn't exist
/// * Generates a sample dataset with 7 data points
/// * Creates one graph of each type with standard options
pub fn create_example_graphs() -> Vec<(String, String)> {
    let mut result = Vec::new();

    // Create output directory if it doesn't exist
    let output_dir = "graph_output";
    std::fs::create_dir_all(output_dir).unwrap_or_else(|_| {
        eprintln!("Output directory already exists or couldn't be created");
    });

    // Create sample data
    let data: Vec<(i32, i32)> = vec![
        (1, 10),
        (2, 25),
        (3, 15),
        (4, 30),
        (5, 22),
        (6, 40),
        (7, 35),
    ];

    // Standard options for all examples
    let base_options = GraphOptions {
        title: "Example Graph".to_string(),
        x_label: "X Values".to_string(),
        y_label: "Y Values".to_string(),
        width: 600,
        height: 400,
        graph_type: GraphType::Line,
    };

    // Line graph
    let mut line_options = base_options.clone();
    line_options.title = "Example Line Graph".to_string();
    line_options.graph_type = GraphType::Line;
    let line_path = format!("{}/line_graph.png", output_dir);
    if save_line_graph(data.clone(), &line_options, &line_path).is_ok() {
        result.push(("Line".to_string(), line_path));
    }

    // Bar graph
    let mut bar_options = base_options.clone();
    bar_options.title = "Example Bar Graph".to_string();
    bar_options.graph_type = GraphType::Bar;
    let bar_path = format!("{}/bar_graph.png", output_dir);
    if save_bar_graph(data.clone(), &bar_options, &bar_path).is_ok() {
        result.push(("Bar".to_string(), bar_path));
    }

    // Scatter graph
    let mut scatter_options = base_options.clone();
    scatter_options.title = "Example Scatter Graph".to_string();
    scatter_options.graph_type = GraphType::Scatter;
    let scatter_path = format!("{}/scatter_graph.png", output_dir);
    if save_scatter_graph(data.clone(), &scatter_options, &scatter_path).is_ok() {
        result.push(("Scatter".to_string(), scatter_path));
    }

    // Area graph
    let mut area_options = base_options.clone();
    area_options.title = "Example Area Graph".to_string();
    area_options.graph_type = GraphType::Area;
    let area_path = format!("{}/area_graph.png", output_dir);
    if save_area_graph(data.clone(), &area_options, &area_path).is_ok() {
        result.push(("Area".to_string(), area_path));
    }

    result
}
