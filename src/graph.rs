use crate::spreadsheet::Spreadsheet;
use plotters::prelude::*;

#[derive(Clone)]

pub enum GraphType {
    Line,
    Bar,
    Scatter,
    // Pie,
    Area,
}
#[derive(Clone)]
pub struct GraphOptions {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub width: u32,
    pub height: u32,
    pub graph_type: GraphType,
}
impl Default for GraphOptions {
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
/// # Arguments
/// * `spreadsheet` - Reference to the spreadsheet
/// * `x_range` - Range for X values (e.g., "A1:A10")
/// * `y_range` - Range for Y values (e.g., "B1:B10")
/// * `options` - Graph styling and type options
///
/// # Returns
/// * A Result containing the PNG image data as bytes
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

/// Creates a line graph
fn create_line_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer = vec![0u8; (options.width * options.height * 4) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (options.width, options.height))
            .into_drawing_area();
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
    }

    Ok(buffer)
}

/// Saves a line graph to a file
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

/// Creates a bar graph
fn create_bar_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer = vec![0u8; (options.width * options.height * 4) as usize]; // Preallocate buffer
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (options.width, options.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let min_x = data.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let max_x = data.iter().map(|(x, _)| x).max().unwrap_or(&100);
        let max_y = data.iter().map(|(_, y)| y).max().unwrap_or(&100);

        let x_range = *min_x as i32..*max_x as i32 + 1;
        let y_range = 0..*max_y as i32 + 1;

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

        chart.draw_series(data.iter().map(|&(x, y)| {
            let bar = Rectangle::new(
                [((x as f64 - 0.4) as i32, 0), ((x as f64 + 0.4) as i32, y)],
                BLUE.filled(),
            );
            bar
        }))?;

        root.present()?;
    }

    Ok(buffer)
}

/// Saves a bar graph to a file
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

    let x_range = *min_x as i32..*max_x as i32 + 1;
    let y_range = 0..*max_y as i32 + 1;

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

    chart.draw_series(data.iter().map(|&(x, y)| {
        Rectangle::new(
            [((x as f64 - 0.4) as i32, 0), ((x as f64 + 0.4) as i32, y)],
            BLUE.filled(),
        )
    }))?;

    root.present()?;

    Ok(())
}

/// Creates a scatter plot
fn create_scatter_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Preallocate the buffer with width * height * 4 (RGBA)
    let mut buffer = vec![0u8; (options.width * options.height * 4) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (options.width, options.height))
            .into_drawing_area();
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

    Ok(buffer)
}

/// Saves a scatter plot to a file
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

/// Creates an area graph
fn create_area_graph(
    data: Vec<(i32, i32)>,
    options: &GraphOptions,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Preallocate the buffer with width * height * 4 bytes (RGBA)
    let mut buffer = vec![0u8; (options.width * options.height * 4) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (options.width, options.height))
            .into_drawing_area();
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
                &BLUE.mix(0.2),
            )
            .border_style(&BLUE),
        )?;

        root.present()?;
    }

    Ok(buffer)
}

/// Saves an area graph to a file
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
            &BLUE.mix(0.2),
        )
        .border_style(&BLUE),
    )?;

    root.present()?;

    Ok(())
}

/// Creates example graphs for demonstration purposes
///
/// # Returns
/// * A vector of tuples containing graph type name and file path
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
    if let Ok(_) = save_line_graph(data.clone(), &line_options, &line_path) {
        result.push(("Line".to_string(), line_path));
    }

    // Bar graph
    let mut bar_options = base_options.clone();
    bar_options.title = "Example Bar Graph".to_string();
    bar_options.graph_type = GraphType::Bar;
    let bar_path = format!("{}/bar_graph.png", output_dir);
    if let Ok(_) = save_bar_graph(data.clone(), &bar_options, &bar_path) {
        result.push(("Bar".to_string(), bar_path));
    }

    // Scatter graph
    let mut scatter_options = base_options.clone();
    scatter_options.title = "Example Scatter Graph".to_string();
    scatter_options.graph_type = GraphType::Scatter;
    let scatter_path = format!("{}/scatter_graph.png", output_dir);
    if let Ok(_) = save_scatter_graph(data.clone(), &scatter_options, &scatter_path) {
        result.push(("Scatter".to_string(), scatter_path));
    }

    // Area graph
    let mut area_options = base_options.clone();
    area_options.title = "Example Area Graph".to_string();
    area_options.graph_type = GraphType::Area;
    let area_path = format!("{}/area_graph.png", output_dir);
    if let Ok(_) = save_area_graph(data.clone(), &area_options, &area_path) {
        result.push(("Area".to_string(), area_path));
    }

    result
}
