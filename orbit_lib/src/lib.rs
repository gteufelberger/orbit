//! Orbit propagation and onboard component simulation

pub mod components;
pub mod propagator;
pub mod satellite;
pub mod simulation;
pub mod sun;

use wasm_bindgen::prelude::*;

/// Simulates every satellite over one time window.
#[wasm_bindgen]
pub fn simulate(config: JsValue) -> Result<JsValue, JsError> {
    let config: simulation::SimulationConfig = serde_wasm_bindgen::from_value(config)
        .map_err(|error| JsError::new(&format!("invalid simulation config: {error}")))?;

    let result = simulation::run(&config).map_err(|error| JsError::new(&error.to_string()))?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|error| JsError::new(&format!("could not serialise result: {error}")))
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
