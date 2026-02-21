use std::ops::Add;

use wasm_bindgen::prelude::*;
use vsop87::*;

#[wasm_bindgen(start)]
pub fn main() {}



/// A function to search through a range of dates and return subsets of those dates where a certain planetary alignment occurred.
/// 
/// Searches the range [start_date, end_date) for the alignment of the nth feature_id with the nth feature_sign.
/// 
/// * `start_julian_date` - a Julian Date for the beginning date for the range of the search, inclusive
/// * `end_julian_date` - a Julian Date the end date for rhe range of the search, exclusive
/// * `feature_ids` - features to calculate for, ids as defined in /src/types/features.ts
/// * `feature_signs` - the zodiac sign of each feature, must be the same length as feature_ids 
#[wasm_bindgen]
pub fn search(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Option<Vec<f64>> {
    
    let mut curr_date: f64 = start_julian_date;

    while curr_date < end_julian_date {
        let earth_coords: RectangularCoordinates = vsop87c::earth(curr_date);

        for i in 0..feature_ids.len() {
            let longitude: f64 = match feature_ids[i] {
                // inner planets
                0 => geocentric_longitude(vsop87c::mercury(curr_date), vsop87c::earth(curr_date)),
                1 => geocentric_longitude(vsop87c::venus(curr_date), vsop87c::earth(curr_date)),
                2 => geocentric_longitude(vsop87c::earth(curr_date), vsop87c::earth(curr_date)),
                3 => geocentric_longitude(vsop87c::mars(curr_date), vsop87c::earth(curr_date)),
                4 => geocentric_longitude(vsop87c::jupiter(curr_date), vsop87c::earth(curr_date)),
                5 => geocentric_longitude(vsop87c::saturn(curr_date), vsop87c::earth(curr_date)),
                6 => geocentric_longitude(vsop87c::uranus(curr_date), vsop87c::earth(curr_date)),
                7 => geocentric_longitude(vsop87c::neptune(curr_date), vsop87c::earth(curr_date)),
                // sun
                10 => vsop87d::earth(curr_date).longitude().to_degrees().add(180.0).rem_euclid(360.0),
                // catch all error case
                _ => return None,
            };

            const DIVIDE_BY_30: f64 = 1.0_f64 / 30.0_f64;
            if ( feature_signs[i] != (longitude * DIVIDE_BY_30) as u8) { break; }

            

        }

        curr_date += 1.0;
    }

    
    
    
    
    
    return Some(Vec::new());
}





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


/// * `feature_coords` - RectangularCoordinates of the feature you want to get the geocentric longitude of
/// * `earth_coords` - RectangularCoordinates of earth to perform the conversion
#[inline]
pub fn geocentric_longitude(feature_coords: RectangularCoordinates, earth_coords: RectangularCoordinates ) -> f64 {
    return (feature_coords.y - earth_coords.y).atan2(feature_coords.x - earth_coords.x).to_degrees().rem_euclid(360.0);
}