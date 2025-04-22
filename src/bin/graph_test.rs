#![cfg(not(tarpaulin_include))]
#[cfg(feature = "web")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate example graphs from the library - now returns file paths
    let graphs = cop::graph::create_example_graphs();

    for (name, file_path) in graphs {
        println!("Created {} graph at {}", name, file_path);
    }

    Ok(())
}
