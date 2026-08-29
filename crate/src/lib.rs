use std::f64::NAN;

use wasm_bindgen::prelude::*;
use vsop87::*;

#[wasm_bindgen(start)]
pub fn main() {} // required by wasm

#[wasm_bindgen]
pub fn search2(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    search_dates(start_julian_date, end_julian_date, feature_ids, feature_signs)
}

pub fn search_dates(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    let mut prev_windows: Vec<(f64, f64)> = Vec::from([(start_julian_date, end_julian_date)]);
    let mut curr_windows: Vec<(f64, f64)> = Vec::new();
    const DIVIDE_BY_30: f64 = 1.0_f64 / 30.0_f64; // calculated at compile time

    for i in 0..feature_ids.len() {
        let coarse_step = coarse_step_for_feature(feature_ids[i]);

        for window in prev_windows.iter() {
            let mut prev_longitude: f64 = geocentric_longitude(window.0, feature_ids[i]);
            let mut prev_longitude_valid: bool = (prev_longitude * DIVIDE_BY_30) as u8 == feature_signs[i];
            let mut curr_window_start: f64 = window.0; // will always get over written if it isnt an actual window start
            let mut curr_step_has_station: bool = false;

            let mut curr_date: f64 = window.0 + coarse_step;
            while curr_date < window.1 {
                let curr_longitude: f64 = geocentric_longitude(curr_date, feature_ids[i]);
                let curr_longitude_valid: bool = (curr_longitude * DIVIDE_BY_30) as u8 == feature_signs[i];
                let next_longitude: f64 = geocentric_longitude(curr_date + coarse_step, feature_ids[i]);

                let left_avg_velocity: f64 = angular_difference(curr_longitude, prev_longitude);
                let right_avg_velocity: f64 = angular_difference(next_longitude, curr_longitude);
                let station_between_now_and_next: bool = !f64_same_sign(left_avg_velocity, right_avg_velocity);
                let mut next_step_has_station: bool = false;
                

                if !curr_step_has_station && station_between_now_and_next {
                    let station_date: f64 = bisection_derivative_find_zero(curr_date - coarse_step, curr_date + coarse_step, feature_ids[i]);
                    if station_date > curr_date {
                        next_step_has_station = true;
                    } else {
                        curr_step_has_station = true;
                    }
                }

                if curr_step_has_station {
                     // check for retrograde stations, split either side into monotonic ranges for zero finding
                    let station_date: f64 = bisection_derivative_find_zero(curr_date - coarse_step, curr_date + coarse_step, feature_ids[i]);
                    let station_longitude: f64 = geocentric_longitude(station_date, feature_ids[i]);
                    let station_longitude_valid: bool = (station_longitude * DIVIDE_BY_30) as u8 == feature_signs[i];


                    // this is messy an bad, but it should be the correct functionality, so were gonna use it to test.
                    if !prev_longitude_valid && station_longitude_valid {
                        let target = sign_boundary_between(prev_longitude, station_longitude);
                        curr_window_start = bisection_value_find(curr_date - coarse_step, station_date, target, feature_ids[i]);
                    }

                    if prev_longitude_valid && !station_longitude_valid {
                        let target = sign_boundary_between(prev_longitude, station_longitude);
                        let window_exit: f64 = bisection_value_find(curr_date - coarse_step, station_date, target, feature_ids[i]);
                        curr_windows.push( (curr_window_start, window_exit) );
                    }

                    if !station_longitude_valid && curr_longitude_valid {
                        let target = sign_boundary_between(station_longitude, curr_longitude);
                        curr_window_start = bisection_value_find(station_date, curr_date, target, feature_ids[i]);
                    }

                    if station_longitude_valid && !curr_longitude_valid {
                        let target = sign_boundary_between(station_longitude, curr_longitude);
                        let window_exit: f64 = bisection_value_find(station_date, curr_date, target, feature_ids[i]);
                        curr_windows.push( (curr_window_start, window_exit) );
                    }

                } else {
                    // same velocity signs means there was no retrograde motion, parse normally
                    if !prev_longitude_valid && curr_longitude_valid {
                        // going from invalid to valid means we passed the start of a window...
                        let target = sign_boundary_between(prev_longitude, curr_longitude);
                        curr_window_start = bisection_value_find(curr_date - coarse_step, curr_date, target, feature_ids[i]);
                    } else if prev_longitude_valid && !curr_longitude_valid {
                        // ...and valid to invalid means we just finished a window
                        let target = sign_boundary_between(prev_longitude, curr_longitude);
                        let window_exit: f64 = bisection_value_find(curr_date - coarse_step, curr_date, target, feature_ids[i]);
                        curr_windows.push( (curr_window_start, window_exit) );
                    }
                }


                prev_longitude = curr_longitude;
                prev_longitude_valid = curr_longitude_valid;
                curr_step_has_station = next_step_has_station;
                //curr_longitude = next_longitude;
                curr_date += coarse_step;
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

/// Returns a conservative search step in days for each supported feature.
/// Steps are bounded by sign-crossing speed and minimum retrograde duration.
#[inline(always)]
pub fn coarse_step_for_feature(feature_id: u8) -> f64 {
    match feature_id {
        0 => 7.3,   // Mercury
        1 => 24.0,  // Venus
        3 => 43.0,  // Mars
        4 => 120.0, // Jupiter
        5 => 135.0, // Saturn
        6 => 150.0, // Uranus
        7 => 156.0, // Neptune
        10 => 14.7, // Sun
        _ => 1.0,   // Preserve conservative behavior until input validation rejects unsupported IDs.
    }
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
            left = midpoint;   // zero is in right half, advance left
        } else {
            right = midpoint;  // zero is in left half, retreat right
        }
    }
}

/// instantaneous velocity using definition of a derivative
#[inline(always)]
pub fn instantaneous_velocity(julian_date: f64, feature_id: u8) -> f64{
    const DERIVATIVE_STEP: f64 = 6e-6_f64; // equivalent to the cube root of f64::EPSILON, for error stuff
    const DOUBLE_DERIVATIVE_STEP: f64 = 1.2e-5_f64; 

    let before = geocentric_longitude(julian_date - DERIVATIVE_STEP, feature_id);
    let after = geocentric_longitude(julian_date + DERIVATIVE_STEP, feature_id);
    return angular_difference(after, before) / DOUBLE_DERIVATIVE_STEP
}

///
pub fn bisection_value_find(start_julian_date: f64, end_julian_date: f64, target_value: f64, feature_id: u8) -> f64 {
    let mut left: f64 = start_julian_date;
    let mut right: f64 = end_julian_date;
    let mut left_error = angular_difference(geocentric_longitude(left, feature_id), target_value);
    const LONGITUDE_TOLERANCE: f64 = 1.0_f64 / 3600.0_f64; // 1 arcsecond accuracy for VSOP87C
    const ONE_MINUTE: f64 = 1.0_f64 / 1440_f64;

    loop {
        let midpoint: f64 = (left + right) * 0.5;
        let midpoint_longitude: f64 = geocentric_longitude(midpoint, feature_id);
        let midpoint_error = angular_difference(midpoint_longitude, target_value);

        if midpoint_error.abs() < LONGITUDE_TOLERANCE || (right - left) < ONE_MINUTE {
            return midpoint;
        } else if f64_same_sign(left_error, midpoint_error) {
            left = midpoint;
            left_error = midpoint_error;
        } else {
            right = midpoint;
        }
    }
}

/// Returns the shortest signed angular displacement from `from` to `to` in
/// the half-open interval [-180°, 180°).
#[inline(always)]
pub fn angular_difference(to: f64, from: f64) -> f64 {
    (to - from + 180.0).rem_euclid(360.0) - 180.0
}

/// Returns the zodiac boundary crossed while moving from `start_longitude`
/// to `end_longitude`. Callers must provide a step containing one boundary at most.
#[inline(always)]
pub fn sign_boundary_between(start_longitude: f64, end_longitude: f64) -> f64 {
    if angular_difference(end_longitude, start_longitude) > 0.0 {
        ((start_longitude / 30.0).floor() + 1.0).rem_euclid(12.0) * 30.0
    } else {
        (start_longitude / 30.0).floor() * 30.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angular_difference_wraps_across_zero() {
        assert_eq!(angular_difference(1.0, 359.0), 2.0);
        assert_eq!(angular_difference(359.0, 1.0), -2.0);
        assert_eq!(angular_difference(0.0, 360.0), 0.0);
    }

    #[test]
    fn identifies_zero_boundary_in_both_directions() {
        assert_eq!(sign_boundary_between(359.0, 1.0), 0.0);
        assert_eq!(sign_boundary_between(1.0, 359.0), 0.0);
    }

    #[test]
    fn uses_safe_coarse_step_for_each_supported_feature() {
        assert_eq!(coarse_step_for_feature(0), 7.3);
        assert_eq!(coarse_step_for_feature(1), 24.0);
        assert_eq!(coarse_step_for_feature(3), 43.0);
        assert_eq!(coarse_step_for_feature(4), 120.0);
        assert_eq!(coarse_step_for_feature(5), 135.0);
        assert_eq!(coarse_step_for_feature(6), 150.0);
        assert_eq!(coarse_step_for_feature(7), 156.0);
        assert_eq!(coarse_step_for_feature(10), 14.7);
        assert_eq!(coarse_step_for_feature(2), 1.0);
        assert_eq!(coarse_step_for_feature(255), 1.0);
    }

    #[test]
    fn refines_prograde_and_retrograde_zero_crossings() {
        let mut previous_date = 2_451_544.5; // 2000-01-01
        let mut previous_longitude = geocentric_longitude(previous_date, 0); // Mercury
        let mut prograde_crossing = None;
        let mut retrograde_crossing = None;

        for day in 1..=(365 * 50) {
            let current_date = 2_451_544.5 + day as f64;
            let current_longitude = geocentric_longitude(current_date, 0);
            let displacement = angular_difference(current_longitude, previous_longitude);

            if previous_longitude > 330.0 && current_longitude < 30.0 && displacement > 0.0 {
                prograde_crossing = Some((previous_date, current_date));
            }

            if previous_longitude < 30.0 && current_longitude > 330.0 && displacement < 0.0 {
                retrograde_crossing = Some((previous_date, current_date));
            }

            if prograde_crossing.is_some() && retrograde_crossing.is_some() {
                break;
            }

            previous_date = current_date;
            previous_longitude = current_longitude;
        }

        for crossing_range in [prograde_crossing, retrograde_crossing] {
            let (start, end) = crossing_range.expect("expected Mercury to cross 0° in both directions");
            let crossing = bisection_value_find(start, end, 0.0, 0);
            let longitude = geocentric_longitude(crossing, 0);
            assert!(angular_difference(longitude, 0.0).abs() < 0.001);
        }
    }

    #[test]
    fn randomized_search_matches_direct_evaluation() {
        const CASE_COUNT: usize = 100;
        const MIN_SEARCH_RADIUS_DAYS: f64 = 180.0;
        const MAX_SEARCH_RADIUS_DAYS: f64 = 1_800.0;
        const REFERENCE_STEP_DAYS: f64 = 1.0;
        const REFERENCE_START_JD: f64 = 2_415_020.5; // 1900-01-01
        const REFERENCE_END_JD: f64 = 2_488_069.5; // 2100-01-01
        const SUPPORTED_FEATURES: [u8; 8] = [0, 1, 3, 4, 5, 6, 7, 10];

        fn next_random(state: &mut u64) -> u64 {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            *state
        }

        fn date_is_valid(julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> bool {
            feature_ids.iter().zip(feature_signs).all(|(&feature_id, &feature_sign)| {
                let longitude = geocentric_longitude(julian_date, feature_id);
                (longitude / 30.0) as u8 == feature_sign
            })
        }

        fn date_is_in_results(julian_date: f64, results: &[f64]) -> bool {
            results
                .chunks_exact(2)
                .any(|window| julian_date >= window[0] && julian_date <= window[1])
        }

        let mut random_state = 0x4e41_5441_4c43_4841_u64;

        for case_index in 0..CASE_COUNT {
            let random_fraction = next_random(&mut random_state) as f64 / u64::MAX as f64;
            let selected_date = REFERENCE_START_JD
                + MAX_SEARCH_RADIUS_DAYS
                + random_fraction
                    * (REFERENCE_END_JD - REFERENCE_START_JD - 2.0 * MAX_SEARCH_RADIUS_DAYS);
            let radius_fraction = next_random(&mut random_state) as f64 / u64::MAX as f64;
            let search_radius = MIN_SEARCH_RADIUS_DAYS
                + radius_fraction * (MAX_SEARCH_RADIUS_DAYS - MIN_SEARCH_RADIUS_DAYS);
            let search_start = selected_date - search_radius;
            let search_end = selected_date + search_radius;

            let mut shuffled_features = SUPPORTED_FEATURES;
            for index in (1..shuffled_features.len()).rev() {
                let swap_index = next_random(&mut random_state) as usize % (index + 1);
                shuffled_features.swap(index, swap_index);
            }

            let feature_count = next_random(&mut random_state) as usize % SUPPORTED_FEATURES.len() + 1;
            let feature_ids = &shuffled_features[..feature_count];
            let feature_signs: Vec<u8> = feature_ids
                .iter()
                .map(|&feature_id| (geocentric_longitude(selected_date, feature_id) / 30.0) as u8)
                .collect();

            let results = search_dates(search_start, search_end, feature_ids, &feature_signs);
            let context = format!(
                "case={case_index}, selected_date={selected_date}, search=[{search_start}, {search_end}], feature_ids={feature_ids:?}, feature_signs={feature_signs:?}, results={results:?}"
            );

            assert_eq!(results.len() % 2, 0, "result array must contain pairs; {context}");

            let mut previous_end = None;
            for window in results.chunks_exact(2) {
                let start = window[0];
                let end = window[1];
                assert!(start < end, "window must have positive length; {context}");
                assert!(start >= search_start && end <= search_end, "window must stay inside the requested range; {context}");
                if let Some(previous_end) = previous_end {
                    assert!(start >= previous_end, "windows must be sorted and nonoverlapping; {context}");
                }
                previous_end = Some(end);
            }

            assert!(
                date_is_in_results(selected_date, &results),
                "the generated source date must be returned; {context}"
            );

            let mut reference_date = search_start;
            while reference_date <= search_end {
                let expected = date_is_valid(reference_date, feature_ids, &feature_signs);
                let actual = date_is_in_results(reference_date, &results);
                assert_eq!(
                    actual, expected,
                    "optimized result disagrees with direct sign evaluation at julian_date={reference_date}; {context}"
                );
                reference_date += REFERENCE_STEP_DAYS;
            }
        }
    }
}
