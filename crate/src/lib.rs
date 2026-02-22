use wasm_bindgen::prelude::*;
use vsop87::*;


#[wasm_bindgen(start)]
pub fn main() {}




#[wasm_bindgen]
pub fn search2(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    let mut prev_windows: Vec<(f64, f64)> = Vec::from([(start_julian_date, end_julian_date)]);
    let mut curr_windows: Vec<(f64, f64)> = Vec::new();
    const COARSE_STEP: f64 = 1.0;

    for id in feature_ids.iter() {
        for window in prev_windows.iter() {
            let mut curr_date: f64 = window.0;
            while curr_date < end_julian_date {
                let earth_coords: RectangularCoordinates = vsop87c::earth(curr_date);

                let longitude: f64 = match *id {
                    //find_windows_planets(&mut curr_windows, start_julian_date, end_julian_date, &vsop87c::jupiter)
                    // planets
                    0 => observer_longitude(earth_coords, vsop87c::mercury(curr_date)),
                    1 => observer_longitude(earth_coords, vsop87c::venus(curr_date)),
                    // 2 is earth
                    3 => observer_longitude(earth_coords, vsop87c::mars(curr_date)),
                    4 => observer_longitude(earth_coords, vsop87c::jupiter(curr_date)),
                    5 => observer_longitude(earth_coords, vsop87c::saturn(curr_date)),
                    6 => observer_longitude(earth_coords, vsop87c::uranus(curr_date)),
                    7 => observer_longitude(earth_coords, vsop87c::neptune(curr_date)),
                    // sun
                    10 => (vsop87b::earth(curr_date).longitude().to_degrees() + 180.0).rem_euclid(360.0),

                    // catch all error case
                    _ => return Vec::from([-1.0]),
                };

                curr_date += COARSE_STEP;
            }
        }
        
        prev_windows = curr_windows;
        curr_windows = Vec::new();
    }


 
    return Vec::new();
}


// pub fn find_windows_planets(output_vec: &mut Vec<(f64, f64)>, start_julian_date: f64, end_julian_date: f64, longitude_func: fn(&f64) -> ) {
//     let mut curr_date: f64 = start_julian_date;
//     const step: f64 = 1.0;
//     let mut prev_longitude: f64 = 0.0;

//     while curr_date < end_julian_date {

//     }

// }


// pub fn step_size_by_planet(feature_id: u8) -> f64 {
//     return 1.0
// }

/// A function to search through a range of dates and return subsets of those dates where a certain planetary alignment occurred. 
/// 
/// Searches the range [start_date, end_date) for the alignment of the nth feature_id with the nth feature_sign.
/// 
/// * `start_julian_date` - a Julian Date for the beginning date for the range of the search, inclusive
/// * `end_julian_date` - a Julian Date the end date for rhe range of the search, exclusive
/// * `feature_ids` - features to calculate for, with the best filter first, ids as defined in /src/types/features.ts
/// * `feature_signs` - the zodiac sign of each feature, must be the same length as feature_ids, ids as defined in /src/types/signs.ts
#[wasm_bindgen]
pub fn search(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    let mut curr_date: f64 = start_julian_date;
    const COARSE_STEP: f64 = 1.0; // on day

    while curr_date < end_julian_date {
        let valid: bool = is_valid_date(curr_date, &feature_ids, &feature_signs);

        if valid {
            output.push(curr_date)
        }

        curr_date += COARSE_STEP;
    }

    return output;
}


/// Returns true if there exists a valid alignment of the nth feature_id with the nth feature_sign at an exact Julian Date, otherwise false.
/// 
/// * `julian_date` - the exact Julian Date to perform the check at
/// * `feature_ids` - features to calculate for, with the best filter first, ids as defined in /src/types/features.ts
/// * `feature_signs` - the zodiac sign of each feature, must be the same length as feature_ids, ids as defined in /src/types/signs.ts
pub fn is_valid_date(julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> bool {
    let earth_coords: RectangularCoordinates = vsop87c::earth(julian_date);

    for i in 0..feature_ids.len() {
        let longitude: f64 = match feature_ids[i] {
            // planets
            0 => observer_longitude(earth_coords, vsop87c::mercury(julian_date)),
            1 => observer_longitude(earth_coords, vsop87c::venus(julian_date)),
            // 2 is earth
            3 => observer_longitude(earth_coords, vsop87c::mars(julian_date)),
            4 => observer_longitude(earth_coords, vsop87c::jupiter(julian_date)),
            5 => observer_longitude(earth_coords, vsop87c::saturn(julian_date)),
            6 => observer_longitude(earth_coords, vsop87c::uranus(julian_date)),
            7 => observer_longitude(earth_coords, vsop87c::neptune(julian_date)),
            // sun
            10 => (vsop87b::earth(julian_date).longitude().to_degrees() + 180.0).rem_euclid(360.0),
            // catch all error case
            _ => return false,
        };

        const DIVIDE_BY_30: f64 = 1.0_f64 / 30.0_f64;
        if feature_signs[i] != (longitude * DIVIDE_BY_30) as u8 { return false }
    }

    return true
}


/// Returns the ecliptic longitude of a feature around a specified observer feature
/// 
/// * `observer_coords` - RectangularCoordinates of the feature that is the reference frame of the calculation
/// * `feature_coords` - RectangularCoordinates of the feature whose longitude you want to get
#[inline]
pub fn observer_longitude(observer_coords: RectangularCoordinates, feature_coords: RectangularCoordinates) -> f64 {
    return (feature_coords.y - observer_coords.y).atan2(feature_coords.x - observer_coords.x).to_degrees().rem_euclid(360.0);
}


/// Returns a list of the solar system's planet's ecliptic longitudes at a given date, used to model the system simply in UI
/// 
/// * `jde` - the exact Julian Date for which to get the positions
#[wasm_bindgen]
pub fn system_model_at_date(julian_date: f64) -> Vec<f64> {
    return Vec::from([
        vsop87d::mercury(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::venus(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::earth(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::mars(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::jupiter(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::saturn(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::uranus(julian_date).longitude().to_degrees().rem_euclid(360.0),
        vsop87d::neptune(julian_date).longitude().to_degrees().rem_euclid(360.0)
    ]);
}