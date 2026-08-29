use std::f64::NAN;

use wasm_bindgen::prelude::*;
use vsop87::*;

#[wasm_bindgen(start)]
pub fn main() {} // required by wasm

#[wasm_bindgen]
pub fn search(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Result<Vec<f64>, JsValue> {
    validate_search_inputs(start_julian_date, end_julian_date, feature_ids, feature_signs).map_err(JsValue::from_str)?;

    Ok(search_refined_windows(start_julian_date, end_julian_date, feature_ids, feature_signs))
}

fn validate_search_inputs(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Result<(), &'static str> {
    if !start_julian_date.is_finite() || !end_julian_date.is_finite() {
        return Err("search dates must be finite");
    }
    if start_julian_date >= end_julian_date {
        return Err("search start date must be earlier than end date");
    }
    if feature_ids.is_empty() || feature_signs.is_empty() {
        return Err("search requires at least one feature and sign");
    }
    if feature_ids.len() != feature_signs.len() {
        return Err("feature and sign lists must have equal lengths");
    }
    if !feature_ids
        .iter()
        .all(|feature_id| matches!(feature_id, 0 | 1 | 3 | 4 | 5 | 6 | 7 | 10))
    {
        return Err("search contains an unsupported feature ID");
    }
    if !feature_signs.iter().all(|feature_sign| *feature_sign <= 11) {
        return Err("sign IDs must be between 0 and 11");
    }
    Ok(())
}

fn search_refined_windows(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    let mut prev_windows: Vec<(f64, f64)> = Vec::from([(start_julian_date, end_julian_date)]);

    for i in 0..feature_ids.len() {
        let mut curr_windows: Vec<(f64, f64)> = Vec::new();
        for &(window_start, window_end) in &prev_windows {
            filter_window_for_feature(window_start, window_end, feature_ids[i], feature_signs[i], &mut curr_windows);
        }
        prev_windows = curr_windows;
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

fn filter_window_for_feature(window_start: f64, window_end: f64, feature_id: u8, feature_sign: u8, output: &mut Vec<(f64, f64)>) {
    if window_start >= window_end {
        return;
    }

    let coarse_step = coarse_step_for_feature(feature_id);
    let mut segment_start = window_start;
    let mut start_longitude = geocentric_longitude(segment_start, feature_id);
    let mut open_window_start = longitude_is_in_sign(start_longitude, feature_sign).then_some(window_start);

    while segment_start < window_end {
        let segment_end = (segment_start + coarse_step).min(window_end);
        let end_longitude = geocentric_longitude(segment_end, feature_id);
        let start_velocity = instantaneous_velocity(segment_start, feature_id);
        let end_velocity = instantaneous_velocity(segment_end, feature_id);

        if segment_has_interior_station(start_velocity, end_velocity) {
            let station_date = bisection_derivative_find_zero(segment_start, segment_end, feature_id);
            let station_longitude = geocentric_longitude(station_date, feature_id);
            process_monotonic_segment(segment_start, station_date, start_longitude, station_longitude, feature_id, feature_sign, &mut open_window_start, output);
            process_monotonic_segment(station_date, segment_end, station_longitude, end_longitude, feature_id, feature_sign, &mut open_window_start, output);
        } else {
            process_monotonic_segment(segment_start, segment_end, start_longitude, end_longitude, feature_id, feature_sign, &mut open_window_start, output);
        }

        segment_start = segment_end;
        start_longitude = end_longitude;
    }

    if let Some(start) = open_window_start {
        if start < window_end {
            output.push((start, window_end));
        }
    }
}

#[inline(always)]
fn segment_has_interior_station(start_velocity: f64, end_velocity: f64) -> bool {
    const VELOCITY_TOLERANCE: f64 = 6e-12;
    start_velocity.abs() > VELOCITY_TOLERANCE
        && end_velocity.abs() > VELOCITY_TOLERANCE
        && !f64_same_sign(start_velocity, end_velocity)
}

fn process_monotonic_segment(segment_start: f64, segment_end: f64, start_longitude: f64, end_longitude: f64, feature_id: u8, feature_sign: u8, open_window_start: &mut Option<f64>, output: &mut Vec<(f64, f64)>) {
    if segment_start >= segment_end {
        return;
    }

    let start_is_valid = longitude_is_in_sign(start_longitude, feature_sign);
    let end_is_valid = longitude_is_in_sign(end_longitude, feature_sign);

    match (start_is_valid, end_is_valid) {
        (false, true) => {
            let target = sign_boundary_between(start_longitude, end_longitude);
            *open_window_start = Some(bisection_value_find(segment_start, segment_end, target, feature_id));
        }
        (true, false) => {
            let target = sign_boundary_between(start_longitude, end_longitude);
            let window_end = bisection_value_find(segment_start, segment_end, target, feature_id);
            let window_start = open_window_start.take().unwrap_or(segment_start);
            if window_start < window_end {
                output.push((window_start, window_end));
            }
        }
        (true, true) => {
            if open_window_start.is_none() {
                *open_window_start = Some(segment_start);
            }
        }
        (false, false) => {}
    }
}

#[inline(always)]
fn longitude_is_in_sign(longitude: f64, sign: u8) -> bool {
    (longitude / 30.0) as u8 == sign
}

/// Returns a conservative search step in days for each supported feature.
/// Steps are bounded by sign-crossing speed and minimum retrograde duration.
#[inline(always)]
pub fn coarse_step_for_feature(feature_id: u8) -> f64 {
    match feature_id {
        0 => 3.5,  // Mercury
        1 => 12.0, // Venus
        3 => 18.0, // Mars
        4 => 60.0, // Jupiter
        5 => 67.0, // Saturn
        6 => 75.0, // Uranus
        7 => 78.0, // Neptune
        10 => 14.0, // Sun
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
#[allow(dead_code)] // Retained as a simple reference algorithm for verification and comparison.
fn search_daily_samples(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
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
        assert_eq!(coarse_step_for_feature(0), 3.5);
        assert_eq!(coarse_step_for_feature(1), 12.0);
        assert_eq!(coarse_step_for_feature(3), 18.0);
        assert_eq!(coarse_step_for_feature(4), 60.0);
        assert_eq!(coarse_step_for_feature(5), 67.0);
        assert_eq!(coarse_step_for_feature(6), 75.0);
        assert_eq!(coarse_step_for_feature(7), 78.0);
        assert_eq!(coarse_step_for_feature(10), 14.0);
        assert_eq!(coarse_step_for_feature(2), 1.0);
        assert_eq!(coarse_step_for_feature(255), 1.0);
    }

    #[test]
    fn brackets_only_stations_inside_a_coarse_segment() {
        assert!(segment_has_interior_station(0.5, -0.5));
        assert!(!segment_has_interior_station(0.0, -0.5));
        assert!(!segment_has_interior_station(0.5, 0.0));
        assert!(!segment_has_interior_station(5e-12, -0.5));
        assert!(!segment_has_interior_station(0.5, -5e-12));
        assert!(!segment_has_interior_station(0.5, 0.25));
    }

    #[test]
    fn validates_every_search_input() {
        assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[5]).is_ok());
        assert!(validate_search_inputs(f64::NAN, 2_453_736.5, &[10], &[5]).is_err());
        assert!(validate_search_inputs(2_453_371.5, f64::INFINITY, &[10], &[5]).is_err());
        assert!(validate_search_inputs(2_453_736.5, 2_453_371.5, &[10], &[5]).is_err());
        assert!(validate_search_inputs(2_453_371.5, 2_453_371.5, &[10], &[5]).is_err());
        assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[], &[]).is_err());
        assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10, 0], &[5]).is_err());
        assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[2], &[5]).is_err());
        assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[11], &[5]).is_err());
        assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[12]).is_err());
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
    fn clips_open_window_to_search_boundaries() {
        let search_start = 2_466_674.835_774_712;
        let search_end = 2_468_099.372_874_511;

        let results = search_refined_windows(search_start, search_end, &[6], &[4]); // Uranus in Leo

        assert_eq!(results, vec![search_start, search_end]);
    }

    #[test]
    fn clips_adjacent_partition_results_to_the_same_boundary() {
        let search_start = 2_453_371.5; // 2005-01-01
        let partition = 2_453_620.5;
        let search_end = 2_453_736.5; // 2006-01-01

        let complete = search_refined_windows(search_start, search_end, &[10], &[5]); // Sun in Virgo
        let left = search_refined_windows(search_start, partition, &[10], &[5]);
        let right = search_refined_windows(partition, search_end, &[10], &[5]);

        assert_eq!(complete.len(), 2);
        assert_eq!(left.len(), 2);
        assert_eq!(right.len(), 2);
        assert_eq!(left[1], partition);
        assert_eq!(right[0], partition);
        assert!((left[0] - complete[0]).abs() < 1.0 / 1440.0);
        assert!((right[1] - complete[1]).abs() < 1.0 / 1440.0);
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

            let results = search_refined_windows(search_start, search_end, feature_ids, &feature_signs);
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
