use wasm_bindgen::prelude::*;

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
