use wasm_bindgen::prelude::*;
use vsop87::*;

#[wasm_bindgen(start)]
pub fn main() {}



#[wasm_bindgen]
pub fn positions_at_jde(jde: f64, planet_ids:  &[u8]) -> Option<Vec<f64>> {
    let mut output: Vec<f64> = Vec::with_capacity(8);
    
    let earth_coords: RectangularCoordinates = vsop87a::earth(jde);

    for id in 0..planet_ids.len() {
        let feature_coords: RectangularCoordinates = match id {
            0 => vsop87a::mercury(jde),
            1 => vsop87a::venus(jde),
            3 => vsop87a::mars(jde),
            4 => vsop87a::jupiter(jde),
            5 => vsop87a::saturn(jde),
            6 => vsop87a::uranus(jde),
            7 => vsop87a::neptune(jde),
            _ => return None,
        };

        output.push(geocentric_longitude(feature_coords, earth_coords));  
    }

    return Some(output)
}

/// Converts heliocentric SphericalCoordinates around the sun to geocentric SphericalCoordinates
/// 
/// Also takes in  
/// 
/// * `feature_coords` - RectangularCoordinates of the feature you want to get the geocentric longitude of
/// * `earth_coords` - RectangularCoordinates of earth to perform the conversion
#[inline(always)]
pub fn geocentric_longitude(feature_coords: RectangularCoordinates, earth_coords: RectangularCoordinates ) -> f64 {
    return (feature_coords.y - earth_coords.y).atan2(feature_coords.x - earth_coords.x).to_degrees().rem_euclid(360.0);
}