use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {

}

// random calculation for testing
#[wasm_bindgen]
pub fn neptune_longitude(julian_date: f64) -> f64 {
    let output = vsop87::vsop87b::neptune(julian_date);
    output.longitude().to_degrees()
}