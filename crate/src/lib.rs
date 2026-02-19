use wasm_bindgen::prelude::*;
use vsop87::*;

#[wasm_bindgen(start)]
pub fn main() {}

#[wasm_bindgen]
pub fn heliocentric_longitudes_at_jde(jde: f64, planet_ids:  &[u8]) -> Option<Vec<f64>> {
    let mut output: Vec<f64> = Vec::with_capacity(8);
    
    for i in 0..planet_ids.len() {
        let feature_coords: SphericalCoordinates = match planet_ids[i] {
            0 => vsop87d::mercury(jde),
            1 => vsop87d::venus(jde),
            2 => vsop87d::earth(jde),
            3 => vsop87d::mars(jde),
            4 => vsop87d::jupiter(jde),
            5 => vsop87d::saturn(jde),
            6 => vsop87d::uranus(jde),
            7 => vsop87d::neptune(jde),
            _ => return None,
        };

        output.push(feature_coords.longitude().to_degrees().rem_euclid(360.0));  
    }

    return Some(output)
}

#[wasm_bindgen]
pub fn geocentric_longitudes_at_jde(jde: f64, planet_ids:  &[u8]) -> Option<Vec<f64>> {
    let mut output: Vec<f64> = Vec::with_capacity(8);
    
    let earth_coords: RectangularCoordinates = vsop87c::earth(jde);

    for i in 0..planet_ids.len() {
        let feature_coords: RectangularCoordinates = match planet_ids[i] {
            0 => vsop87c::mercury(jde),
            1 => vsop87c::venus(jde),
            // 2 is earth
            3 => vsop87c::mars(jde),
            4 => vsop87c::jupiter(jde),
            5 => vsop87c::saturn(jde),
            6 => vsop87c::uranus(jde),
            7 => vsop87c::neptune(jde),
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
#[inline]
pub fn geocentric_longitude(feature_coords: RectangularCoordinates, earth_coords: RectangularCoordinates ) -> f64 {
    return (feature_coords.y - earth_coords.y).atan2(feature_coords.x - earth_coords.x).to_degrees().rem_euclid(360.0);
}