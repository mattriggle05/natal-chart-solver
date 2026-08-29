use std::f64::NAN;

use wasm_bindgen::prelude::*;
use vsop87::*;

const VELOCITY_TOLERANCE: f64 = 6e-12;
const ONE_MINUTE: f64 = 1.0 / 1440.0;

/// Returns flattened Julian-date window pairs using `[start, end)` semantics.
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
        merge_adjacent_or_overlapping_windows(&mut curr_windows);
        prev_windows = curr_windows;
    }

    let mut flattened_return: Vec<f64> = Vec::with_capacity(prev_windows.len() * 2);
    for (a, b) in &prev_windows {
        flattened_return.push(*a);
        flattened_return.push(*b);
    }
    return flattened_return;
}

fn merge_adjacent_or_overlapping_windows(windows: &mut Vec<(f64, f64)>) {
    if windows.len() < 2 { return; }

    let mut write_index = 0;
    for read_index in 1..windows.len() {
        let (start, end) = windows[read_index];
        if start <= windows[write_index].1 {
            windows[write_index].1 = windows[write_index].1.max(end);
        } else {
            write_index += 1;
            windows[write_index] = (start, end);
        }
    }
    windows.truncate(write_index + 1);
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
    if reference_velocity == 0.0 { return left; }
    if instantaneous_velocity(right, feature_id) == 0.0 { return right; }

    loop {
        let midpoint: f64 = (left + right) * 0.5;

        let midpoint_velocity: f64 = instantaneous_velocity(midpoint, feature_id);

        // we are explicitly search for zero velocity, so just compare directly
        if midpoint_velocity == 0.0 || (right - left) < ONE_MINUTE{
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
    if left_error == 0.0 { return left; }
    if angular_difference(geocentric_longitude(right, feature_id), target_value) == 0.0 { return right; }

    loop {
        let midpoint: f64 = (left + right) * 0.5;
        let midpoint_longitude: f64 = geocentric_longitude(midpoint, feature_id);
        let midpoint_error = angular_difference(midpoint_longitude, target_value);

        if midpoint_error == 0.0 || (right - left) < ONE_MINUTE {
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
mod tests;
