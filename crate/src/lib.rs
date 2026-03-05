use std::f64::NAN;

use wasm_bindgen::prelude::*;
use vsop87::*;

#[wasm_bindgen(start)]
pub fn main() {} // required by wasm

#[wasm_bindgen]
pub fn search2(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    let mut prev_windows: Vec<(f64, f64)> = Vec::from([(start_julian_date, end_julian_date)]);
    let mut curr_windows: Vec<(f64, f64)> = Vec::new();
    const DIVIDE_BY_30: f64 = 1.0_f64 / 30.0_f64; // calculated at compile time

    for i in 0..feature_ids.len() {
        const COARSE_STEP: f64 = 1.0;

        for window in prev_windows.iter() {
            let mut prev_longitude: f64 = geocentric_longitude(window.0, feature_ids[i]);
            let mut prev_longitude_valid: bool = (prev_longitude * DIVIDE_BY_30) as u8 == feature_signs[i];
            let mut curr_window_start: f64 = prev_longitude; // will always get over written if it isnt an actual window start
            let mut curr_step_has_station: bool = false;

            let mut curr_date: f64 = window.0 + COARSE_STEP;
            while curr_date < window.1 {
                let curr_longitude: f64 = geocentric_longitude(curr_date, feature_ids[i]);
                let curr_longitude_valid: bool = (curr_longitude * DIVIDE_BY_30) as u8 == feature_signs[i];
                let next_longitude: f64 = geocentric_longitude(curr_date + COARSE_STEP, feature_ids[i]);

                let left_avg_velocity: f64 = curr_longitude - prev_longitude;
                let right_avg_velocity: f64 = next_longitude - curr_longitude;
                let station_between_now_and_next: bool = f64_same_sign(left_avg_velocity, right_avg_velocity);
                let mut next_step_has_station: bool = false;
                

                if !curr_step_has_station && station_between_now_and_next {
                    let station_date: f64 = bisection_derivative_find_zero(curr_date - COARSE_STEP, curr_date + COARSE_STEP, feature_ids[i]);
                    if station_date > curr_date {
                        next_step_has_station = true;
                    } else {
                        curr_step_has_station = true;
                    }
                }

                if curr_step_has_station {
                     // check for retrograde stations, split either side into monotonic ranges for zero finding
                    let station_date: f64 = bisection_derivative_find_zero(curr_date - COARSE_STEP, curr_date + COARSE_STEP, feature_ids[i]);
                    let station_longitude: f64 = geocentric_longitude(station_date, feature_ids[i]);
                    let station_longitude_valid: bool = (station_longitude * DIVIDE_BY_30) as u8 == feature_signs[i];

                    if prev_longitude_valid && !station_longitude_valid {
                        //bisect add window
                    }
                } else {
                    // same velocity signs means there was no retrograde motion, parse normally
                    if !prev_longitude_valid && curr_longitude_valid {
                        // going from invalid to valid means we passed the start of a window...
                        curr_window_start = bisection_value_find(curr_date - COARSE_STEP, curr_date, (feature_signs[i] as f64)*30.0, feature_ids[i]);
                    } else if prev_longitude_valid && !curr_longitude_valid {
                        // ...and valid to invalid means we just finished a window
                        let window_exit: f64 = bisection_value_find(curr_date - COARSE_STEP, curr_date, ((feature_signs[i]+1) as f64)*30.0, feature_ids[i]);
                        curr_windows.push( (curr_window_start, window_exit) );
                    }
                }


                prev_longitude = curr_longitude;
                prev_longitude_valid = curr_longitude_valid;
                curr_step_has_station = next_step_has_station;
                //curr_longitude = next_longitude;
                curr_date += COARSE_STEP;
            }
        }

        prev_windows = curr_windows;
        curr_windows = Vec::new();
    }

    // TODO: replace original tuple arrays with flat array in the first place
    // OR
    // TODO: remap memory at location ot flat array from tuple array (unsafe)
    let mut flattened_return: Vec<f64> = Vec::with_capacity(prev_windows.len() * 2);
    for (a, b) in &prev_windows {
        flattened_return.push(*a);
        flattened_return.push(*b);
    }
    return flattened_return;
}


///
#[inline(always)]
pub fn bisection_derivative_find_zero(start_julian_date: f64, end_julian_date: f64, feature_id: u8) -> f64{
    let mut left: f64 = start_julian_date;
    let mut right: f64 = end_julian_date;
    let reference_velocity: f64 = instantaneous_velocity(left, feature_id);
    const VELOCITY_TOLERANCE: f64 = 6e-12_f64; // proportional to the square of the error term in instantaneous_velocity
    const ONE_MINUTE: f64 = 1.0_f64 / 1440_f64;

    loop {
        let midpoint: f64 = (left + right) * 0.5;

        let midpoint_velocity: f64 = instantaneous_velocity(midpoint, feature_id);

        // we are explicitly search for zero velocity, so just compare directly
        if midpoint_velocity.abs() <= VELOCITY_TOLERANCE || (right - left) < ONE_MINUTE{
            return midpoint
        } else if f64_same_sign(reference_velocity, midpoint_velocity) {
            right = midpoint;
        } else {
            left = midpoint;
        }
    }
}

/// instantaneous velocity using definition of a derivative
#[inline(always)]
pub fn instantaneous_velocity(julian_date: f64, feature_id: u8) -> f64{
    const DERIVATIVE_STEP: f64 = 6e-6_f64; // equivalent to the cube root of f64::EPSILON, for error stuff
    const DOUBLE_DERIVATIVE_STEP: f64 = 1.2e-5_f64; 

    return (geocentric_longitude(julian_date + DERIVATIVE_STEP, feature_id) - geocentric_longitude(julian_date - DERIVATIVE_STEP, feature_id)) / DOUBLE_DERIVATIVE_STEP
}

///
pub fn bisection_value_find(start_julian_date: f64, end_julian_date: f64, target_value: f64, feature_id: u8) -> f64 {
    let mut left: f64 = start_julian_date;
    let mut right: f64 = end_julian_date;
    const LONGITUDE_TOLERANCE: f64 = 1.0_f64 / 3600.0_f64; // 1 arcsecond accuracy for VSOP87C
    const ONE_MINUTE: f64 = 1.0_f64 / 1440_f64;

    loop {
        let midpoint: f64 = (left + right) * 0.5;
        let midpoint_longitude: f64 = geocentric_longitude(midpoint, feature_id);

        if (midpoint_longitude - target_value).abs() < LONGITUDE_TOLERANCE || (right - left) < ONE_MINUTE {
            return midpoint;
        } else if midpoint_longitude < target_value {
            left = midpoint;
         } else {
            right = midpoint;
         }
    }
}

#[inline]
pub fn geocentric_longitude(julian_date: f64, feature_id: u8) -> f64 {
    let earth: RectangularCoordinates = vsop87c::earth(julian_date);
    return match feature_id {
        // planets
        0 => longitude_from_observer(earth, vsop87c::mercury(julian_date)),
        1 => longitude_from_observer(earth, vsop87c::venus(julian_date)),
        // 2 is earth
        3 => longitude_from_observer(earth, vsop87c::mars(julian_date)),
        4 => longitude_from_observer(earth, vsop87c::jupiter(julian_date)),
        5 => longitude_from_observer(earth, vsop87c::saturn(julian_date)),
        6 => longitude_from_observer(earth, vsop87c::uranus(julian_date)),
        7 => longitude_from_observer(earth, vsop87c::neptune(julian_date)),
        // sun
        10 => (vsop87b::earth(julian_date).longitude().to_degrees() + 180.0).rem_euclid(360.0),
        // catch all error case
        _ => return NAN,
    };
}

/// Returns the ecliptic longitude of a feature around a specified observer feature
/// 
/// * `observer_coords` - RectangularCoordinates of the feature that is the reference frame of the calculation
/// * `feature_coords` - RectangularCoordinates of the feature whose longitude you want to get
#[inline(always)]
pub fn longitude_from_observer(observer_coords: RectangularCoordinates, feature_coords: RectangularCoordinates) -> f64 {
    return (feature_coords.y - observer_coords.y).atan2(feature_coords.x - observer_coords.x).to_degrees().rem_euclid(360.0);
}

/// bit manip to check signs, treats +0.0 and -0.0 as their own sign
/// in this use case its impossible for a and b to both be 0 so we ignore it
#[inline(always)]
pub fn f64_same_sign(a: f64, b: f64) -> bool {
    let a_bits: u64 = a.to_bits();
    let b_bits: u64 = b.to_bits();
    if a_bits << 1 == 0 || b_bits << 1 == 0 { return false; }
    (a_bits ^ b_bits) >> 63 == 0
}










// -----------------------------------------------------

// BELOW IS OLD CODE FOR FIRST SEARCH FUNCTION, IGNORE

// -----------------------------------------------------











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
            0 => longitude_from_observer(earth_coords, vsop87c::mercury(julian_date)),
            1 => longitude_from_observer(earth_coords, vsop87c::venus(julian_date)),
            // 2 is earth
            3 => longitude_from_observer(earth_coords, vsop87c::mars(julian_date)),
            4 => longitude_from_observer(earth_coords, vsop87c::jupiter(julian_date)),
            5 => longitude_from_observer(earth_coords, vsop87c::saturn(julian_date)),
            6 => longitude_from_observer(earth_coords, vsop87c::uranus(julian_date)),
            7 => longitude_from_observer(earth_coords, vsop87c::neptune(julian_date)),
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
