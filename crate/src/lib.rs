use wasm_bindgen::prelude::*;
use vsop87::*;

const VELOCITY_TOLERANCE: f64 = 6e-12;
const ONE_MINUTE: f64 = 1.0 / 1440.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Feature {
    MercuryLongitude = 0,
    VenusLongitude = 1,
    MarsLongitude = 3,
    JupiterLongitude = 4,
    SaturnLongitude = 5,
    UranusLongitude = 6,
    NeptuneLongitude = 7,
    SunLongitude = 10,
    MoonLongitude = 11,
    MoonPhaseAngle = 16,
}

impl TryFrom<u8> for Feature {
    type Error = ();

    fn try_from(feature_id: u8) -> Result<Self, Self::Error> {
        match feature_id {
            0 => Ok(Self::MercuryLongitude),
            1 => Ok(Self::VenusLongitude),
            3 => Ok(Self::MarsLongitude),
            4 => Ok(Self::JupiterLongitude),
            5 => Ok(Self::SaturnLongitude),
            6 => Ok(Self::UranusLongitude),
            7 => Ok(Self::NeptuneLongitude),
            10 => Ok(Self::SunLongitude),
            11 => Ok(Self::MoonLongitude),
            16 => Ok(Self::MoonPhaseAngle),
            _ => Err(()),
        }
    }
}

/// Returns flattened Julian-date window pairs using `[start, end)` semantics.
#[wasm_bindgen]
pub fn search(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], angle_starts: &[f64], angle_spans: &[f64]) -> Result<Vec<f64>, JsValue> {
    validate_search_inputs(start_julian_date, end_julian_date, feature_ids, angle_starts, angle_spans).map_err(JsValue::from_str)?;

    Ok(search_refined_windows(start_julian_date, end_julian_date, feature_ids, angle_starts, angle_spans))
}

fn validate_search_inputs(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], angle_starts: &[f64], angle_spans: &[f64]) -> Result<(), &'static str> {
    if !start_julian_date.is_finite() || !end_julian_date.is_finite() {
        return Err("search dates must be finite");
    }
    if start_julian_date >= end_julian_date {
        return Err("search start date must be earlier than end date");
    }
    if feature_ids.is_empty() || angle_starts.is_empty() || angle_spans.is_empty() {
        return Err("search requires at least one angular constraint");
    }
    if feature_ids.len() != angle_starts.len() || feature_ids.len() != angle_spans.len() {
        return Err("feature, angle start, and angle span lists must have equal lengths");
    }
    if !feature_ids.iter().all(|&feature_id| Feature::try_from(feature_id).is_ok()) {
        return Err("search contains an unsupported feature ID");
    }
    if !angle_starts.iter().all(|&angle_start| angle_start.is_finite() && (0.0..360.0).contains(&angle_start)) {
        return Err("angle starts must be finite and between 0 inclusive and 360 exclusive");
    }
    if !angle_spans.iter().all(|&angle_span| angle_span.is_finite() && angle_span > 0.0 && angle_span <= 360.0) {
        return Err("angle spans must be finite and greater than 0 and no greater than 360");
    }
    Ok(())
}

fn search_refined_windows(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], angle_starts: &[f64], angle_spans: &[f64]) -> Vec<f64> {
    let mut prev_windows: Vec<(f64, f64)> = Vec::from([(start_julian_date, end_julian_date)]);

    let constraint_order = (0..feature_ids.len()).filter(|&index| feature_ids[index] == Feature::SunLongitude as u8).chain((0..feature_ids.len()).filter(|&index| feature_ids[index] != Feature::SunLongitude as u8));

    for index in constraint_order {
        let feature = Feature::try_from(feature_ids[index]).expect("validated feature ID");
        let mut curr_windows: Vec<(f64, f64)> = Vec::new();
        for &(window_start, window_end) in &prev_windows {
            filter_window_for_constraint(window_start, window_end, feature, angle_starts[index], angle_spans[index], &mut curr_windows);
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

fn filter_window_for_constraint(window_start: f64, window_end: f64, feature: Feature, angle_start: f64, angle_span: f64, output: &mut Vec<(f64, f64)>) {
    if window_start >= window_end {
        return;
    }

    match feature {
        Feature::SunLongitude => filter_sun_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output),
        Feature::MoonLongitude => filter_moon_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output),
        Feature::MoonPhaseAngle => filter_moon_phase_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output),
        _ => filter_retrograde_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output),
    }
}

#[inline(always)]
fn filter_sun_window_for_constraint(window_start: f64, window_end: f64, feature: Feature, angle_start: f64, angle_span: f64, output: &mut Vec<(f64, f64)>) {
    filter_monotonic_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output);
}

#[inline(always)]
fn filter_moon_window_for_constraint(window_start: f64, window_end: f64, feature: Feature, angle_start: f64, angle_span: f64, output: &mut Vec<(f64, f64)>) {
    filter_monotonic_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output);
}

#[inline(always)]
fn filter_moon_phase_window_for_constraint(window_start: f64, window_end: f64, feature: Feature, angle_start: f64, angle_span: f64, output: &mut Vec<(f64, f64)>) {
    filter_monotonic_window_for_constraint(window_start, window_end, feature, angle_start, angle_span, output);
}

fn filter_retrograde_window_for_constraint(window_start: f64, window_end: f64, feature: Feature, angle_start: f64, angle_span: f64, output: &mut Vec<(f64, f64)>) {
    let coarse_step = coarse_step_for_feature(feature);
    let mut segment_start = window_start;
    let mut start_angle = angle_at(segment_start, feature);
    let mut start_velocity = instantaneous_velocity(segment_start, feature);
    let mut open_window_start = angle_is_in_range(start_angle, angle_start, angle_span).then_some(window_start);

    while segment_start < window_end {
        let segment_end = (segment_start + coarse_step).min(window_end);
        let end_angle = angle_at(segment_end, feature);
        let end_velocity = instantaneous_velocity(segment_end, feature);

        if segment_has_interior_station(start_velocity, end_velocity) {
            let station_date = bisection_derivative_find_zero(segment_start, segment_end, feature);
            let station_angle = angle_at(station_date, feature);
            process_monotonic_segment(segment_start, station_date, start_angle, station_angle, feature, angle_start, angle_span, &mut open_window_start, output);
            process_monotonic_segment(station_date, segment_end, station_angle, end_angle, feature, angle_start, angle_span, &mut open_window_start, output);
        } else {
            process_monotonic_segment(segment_start, segment_end, start_angle, end_angle, feature, angle_start, angle_span, &mut open_window_start, output);
        }

        segment_start = segment_end;
        start_angle = end_angle;
        start_velocity = end_velocity;
    }

    if let Some(start) = open_window_start {
        if start < window_end {
            output.push((start, window_end));
        }
    }
}

fn filter_monotonic_window_for_constraint(window_start: f64, window_end: f64, feature: Feature, angle_start: f64, angle_span: f64, output: &mut Vec<(f64, f64)>) {
    let coarse_step = coarse_step_for_feature(feature);
    let mut segment_start = window_start;
    let mut start_angle = angle_at(segment_start, feature);
    let mut open_window_start = angle_is_in_range(start_angle, angle_start, angle_span).then_some(window_start);

    while segment_start < window_end {
        let segment_end = (segment_start + coarse_step).min(window_end);
        let end_angle = angle_at(segment_end, feature);
        process_monotonic_segment(segment_start, segment_end, start_angle, end_angle, feature, angle_start, angle_span, &mut open_window_start, output);
        segment_start = segment_end;
        start_angle = end_angle;
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

fn process_monotonic_segment(segment_start: f64, segment_end: f64, start_angle: f64, end_angle: f64, feature: Feature, angle_start: f64, angle_span: f64, open_window_start: &mut Option<f64>, output: &mut Vec<(f64, f64)>) {
    if segment_start >= segment_end || angle_span == 360.0 {
        return;
    }

    let displacement = angular_difference(end_angle, start_angle);
    if displacement == 0.0 {
        return;
    }

    let angle_immediately_after_start = (start_angle + displacement.signum() * 1e-10).rem_euclid(360.0);
    let should_be_open = angle_is_in_range(angle_immediately_after_start, angle_start, angle_span);
    if should_be_open && open_window_start.is_none() {
        *open_window_start = Some(segment_start);
    } else if !should_be_open {
        if let Some(window_start) = open_window_start.take() {
            if window_start < segment_start {
                output.push((window_start, segment_start));
            }
        }
    }

    let end_boundary = (angle_start + angle_span).rem_euclid(360.0);
    let mut crossings = [
        constraint_boundary_crossing(start_angle, displacement, angle_start),
        constraint_boundary_crossing(start_angle, displacement, end_boundary),
    ];
    if crossings[0].0 > crossings[1].0 {
        crossings.swap(0, 1);
    }

    for (distance, target_angle) in crossings {
        if distance > 0.0 && distance <= displacement.abs() {
            let crossing_date = bisection_value_find_with_angles(segment_start, segment_end, start_angle, end_angle, target_angle, feature);
            if let Some(window_start) = open_window_start.take() {
                if window_start < crossing_date {
                    output.push((window_start, crossing_date));
                }
            } else {
                *open_window_start = Some(crossing_date);
            }
        }
    }
}

#[inline(always)]
fn angle_is_in_range(angle: f64, angle_start: f64, angle_span: f64) -> bool {
    (angle - angle_start).rem_euclid(360.0) < angle_span
}

#[inline(always)]
fn constraint_boundary_crossing(start_angle: f64, displacement: f64, boundary: f64) -> (f64, f64) {
    let distance = if displacement > 0.0 { (boundary - start_angle).rem_euclid(360.0) } else { (start_angle - boundary).rem_euclid(360.0) };
    (distance, boundary)
}

/// Returns a conservative search step in days for each supported feature.
/// Steps are bounded by angular speed and minimum retrograde duration.
#[inline(always)]
pub fn coarse_step_for_feature(feature: Feature) -> f64 {
    match feature {
        Feature::MercuryLongitude => 3.5,
        Feature::VenusLongitude => 12.0,
        Feature::MarsLongitude => 18.0,
        Feature::JupiterLongitude => 60.0,
        Feature::SaturnLongitude => 67.0,
        Feature::UranusLongitude => 75.0,
        Feature::NeptuneLongitude => 78.0,
        Feature::SunLongitude => 28.0,
        Feature::MoonLongitude => 10.0,
        Feature::MoonPhaseAngle => 10.0,
    }
}


///
#[inline(always)]
pub fn bisection_derivative_find_zero(start_julian_date: f64, end_julian_date: f64, feature: Feature) -> f64{
    let mut left: f64 = start_julian_date;
    let mut right: f64 = end_julian_date;
    let reference_velocity: f64 = instantaneous_velocity(left, feature);
    if reference_velocity == 0.0 { return left; }
    if instantaneous_velocity(right, feature) == 0.0 { return right; }

    loop {
        let midpoint: f64 = (left + right) * 0.5;

        let midpoint_velocity: f64 = instantaneous_velocity(midpoint, feature);

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
pub fn instantaneous_velocity(julian_date: f64, feature: Feature) -> f64{
    const DERIVATIVE_STEP: f64 = 6e-6_f64; // equivalent to the cube root of f64::EPSILON, for error stuff
    const DOUBLE_DERIVATIVE_STEP: f64 = 1.2e-5_f64; 

    let before = angle_at(julian_date - DERIVATIVE_STEP, feature);
    let after = angle_at(julian_date + DERIVATIVE_STEP, feature);
    return angular_difference(after, before) / DOUBLE_DERIVATIVE_STEP
}

///
pub fn bisection_value_find(start_julian_date: f64, end_julian_date: f64, target_value: f64, feature: Feature) -> f64 {
    let start_angle = angle_at(start_julian_date, feature);
    let end_angle = angle_at(end_julian_date, feature);
    bisection_value_find_with_angles(start_julian_date, end_julian_date, start_angle, end_angle, target_value, feature)
}

fn bisection_value_find_with_angles(start_julian_date: f64, end_julian_date: f64, start_angle: f64, end_angle: f64, target_value: f64, feature: Feature) -> f64 {
    let mut left: f64 = start_julian_date;
    let mut right: f64 = end_julian_date;
    let mut left_error = angular_difference(start_angle, target_value);
    if left_error == 0.0 { return left; }
    if angular_difference(end_angle, target_value) == 0.0 { return right; }

    loop {
        let midpoint: f64 = (left + right) * 0.5;
        let midpoint_angle: f64 = angle_at(midpoint, feature);
        let midpoint_error = angular_difference(midpoint_angle, target_value);

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

#[inline]
pub fn angle_at(julian_date: f64, feature: Feature) -> f64 {
    match feature {
        Feature::SunLongitude => return sun_longitude(julian_date),
        Feature::MoonLongitude => return moon_longitude(julian_date),
        Feature::MoonPhaseAngle => return (moon_longitude(julian_date) - sun_longitude(julian_date)).rem_euclid(360.0),
        _ => {}
    }

    let earth: RectangularCoordinates = vsop87c::earth(julian_date);
    return match feature {
        Feature::MercuryLongitude => longitude_from_observer(earth, vsop87c::mercury(julian_date)),
        Feature::VenusLongitude => longitude_from_observer(earth, vsop87c::venus(julian_date)),
        Feature::MarsLongitude => longitude_from_observer(earth, vsop87c::mars(julian_date)),
        Feature::JupiterLongitude => longitude_from_observer(earth, vsop87c::jupiter(julian_date)),
        Feature::SaturnLongitude => longitude_from_observer(earth, vsop87c::saturn(julian_date)),
        Feature::UranusLongitude => longitude_from_observer(earth, vsop87c::uranus(julian_date)),
        Feature::NeptuneLongitude => longitude_from_observer(earth, vsop87c::neptune(julian_date)),
        Feature::SunLongitude | Feature::MoonLongitude | Feature::MoonPhaseAngle => unreachable!(),
    };
}

#[inline(always)]
fn sun_longitude(julian_date: f64) -> f64 {
    (vsop87d::earth(julian_date).longitude().to_degrees() + 180.0).rem_euclid(360.0)
}

#[inline(always)]
fn moon_longitude(julian_date: f64) -> f64 {
    astro::lunar::geocent_ecl_pos(julian_date).0.long.to_degrees().rem_euclid(360.0)
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
