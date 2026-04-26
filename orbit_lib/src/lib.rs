use wasm_bindgen::prelude::*;

/// Calculate satellite position in Cartesian coordinates
///
/// # Arguments
/// * `time_seconds` - Time in seconds since epoch
/// * `orbital_period` - Orbital period in seconds
/// * `orbital_radius` - Orbital radius in meters
///
/// # Returns
/// Array [x, y, z] in meters
#[wasm_bindgen]
pub fn calculate_satellite_position(
    time_seconds: f64,
    orbital_period: f64,
    orbital_radius: f64,
) -> Vec<f64> {
    let seconds = time_seconds % orbital_period;
    let angle = (seconds / orbital_period) * 2.0 * std::f64::consts::PI;

    let x = orbital_radius * angle.cos();
    let y = orbital_radius * angle.sin();
    let z = 0.0;

    vec![x, y, z]
}

/// Hello world function that returns a greeting message
#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello from WebAssembly! 🚀".to_string()
}

/// Add two numbers
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
