use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {

}

// random calculation for testing
#[wasm_bindgen]
pub fn mars_longitude(julian_date: f64) -> f64 {
    let coords = vsop87::vsop87b::neptune(julian_date);
    coords.l.to_degrees().rem_euclid(360.0)
}