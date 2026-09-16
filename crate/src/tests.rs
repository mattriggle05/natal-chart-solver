use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};

fn feature_from_id(feature_id: u8) -> Feature {
    Feature::try_from(feature_id).expect("supported test feature")
}

fn feature_angle(julian_date: f64, feature_id: u8) -> f64 {
    angle_at(julian_date, feature_from_id(feature_id))
}

fn search_sign_windows(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64> {
    let angle_starts: Vec<f64> = feature_signs.iter().map(|&feature_sign| feature_sign as f64 * 30.0).collect();
    let angle_spans = vec![30.0; feature_signs.len()];
    search_refined_windows(start_julian_date, end_julian_date, feature_ids, &angle_starts, &angle_spans)
}

fn date_is_valid(julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> bool {
    feature_ids.iter().zip(feature_signs).all(|(&feature_id, &feature_sign)| angle_is_in_range(feature_angle(julian_date, feature_id), feature_sign as f64 * 30.0, 30.0))
}

fn date_is_in_results(julian_date: f64, results: &[f64]) -> bool {
    results.chunks_exact(2).any(|window| julian_date >= window[0] && julian_date < window[1])
}

fn assert_result_boundaries_match_direct_evaluation(search_start: f64, search_end: f64, feature_ids: &[u8], feature_signs: &[u8], results: &[f64], context: &str) {
    const BOUNDARY_PROBE_DAYS: f64 = 2.0 / 1440.0;

    for (index, window) in results.chunks_exact(2).enumerate() {
        let start = window[0];
        let end = window[1];
        let interior_probe = BOUNDARY_PROBE_DAYS.min((end - start) * 0.5);
        assert!(date_is_valid(start + interior_probe, feature_ids, feature_signs), "window must be valid immediately inside its start; {context}");
        assert!(date_is_valid(end - interior_probe, feature_ids, feature_signs), "window must be valid immediately inside its end; {context}");

        let previous_end = if index == 0 { search_start } else { results[index * 2 - 1] };
        if previous_end < start {
            let exterior_probe = BOUNDARY_PROBE_DAYS.min((start - previous_end) * 0.5);
            assert!(!date_is_valid(start - exterior_probe, feature_ids, feature_signs), "window must be invalid immediately outside its start; {context}");
        }

        let next_start = if index * 2 + 2 == results.len() { search_end } else { results[index * 2 + 2] };
        if end < next_start {
            let exterior_probe = BOUNDARY_PROBE_DAYS.min((next_start - end) * 0.5);
            assert!(!date_is_valid(end + exterior_probe, feature_ids, feature_signs), "window must be invalid immediately outside its end; {context}");
        }
    }
}

fn profile_repeated<F>(label: &str, sample_count: usize, mut run: F) where F: FnMut() {
    run();
    let mut samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        let started = Instant::now();
        run();
        samples.push(started.elapsed().as_secs_f64());
    }

    samples.sort_by(f64::total_cmp);
    let mean = samples.iter().sum::<f64>() / sample_count as f64;
    let variance = samples.iter().map(|sample| (sample - mean).powi(2)).sum::<f64>() / (sample_count - 1) as f64;
    let standard_deviation = variance.sqrt();
    let confidence_interval_95 = 2.262 * standard_deviation / (sample_count as f64).sqrt();
    let median = samples[sample_count / 2];
    eprintln!("{label}: mean={:?}, median={:?}, standard_deviation={:?}, mean_95_percent_ci=±{:?}, samples={sample_count}", Duration::from_secs_f64(mean), Duration::from_secs_f64(median), Duration::from_secs_f64(standard_deviation), Duration::from_secs_f64(confidence_interval_95));
}

#[test]
fn angular_difference_wraps_across_zero() {
    assert_eq!(angular_difference(1.0, 359.0), 2.0);
    assert_eq!(angular_difference(359.0, 1.0), -2.0);
    assert_eq!(angular_difference(0.0, 360.0), 0.0);
}

#[test]
fn maps_angles_to_half_open_constraints() {
    let immediately_before_30 = f64::from_bits(30.0_f64.to_bits() - 1);
    let immediately_before_360 = f64::from_bits(360.0_f64.to_bits() - 1);

    assert!(angle_is_in_range(0.0, 0.0, 30.0));
    assert!(angle_is_in_range(immediately_before_30, 0.0, 30.0));
    assert!(!angle_is_in_range(30.0, 0.0, 30.0));
    assert!(angle_is_in_range(immediately_before_360, 330.0, 30.0));
    assert!(!angle_is_in_range(0.0, 330.0, 30.0));
}

#[test]
fn matches_reference_sun_longitudes_across_supported_centuries() {
    // NASA JPL Horizons DE441 apparent geocentric ecliptic-of-date longitude, quantity 31.
    // The tolerance covers the expected difference between VSOP87's geometric coordinates
    // and Horizons' light-time, aberration, and gravitational-deflection corrections.
    const MAX_DIFFERENCE_DEGREES: f64 = 0.02;
    let cases = [
        (2_415_021.0, 280.663_243_8), // 1900-01-01 12:00 TT
        (2_451_545.0, 280.368_151_9), // 2000-01-01 12:00 TT
        (2_488_070.0, 281.112_835_3), // 2100-01-01 12:00 TT
    ];

    for (julian_date, reference_longitude) in cases {
        let longitude = feature_angle(julian_date, 10);
        let difference = angular_difference(longitude, reference_longitude).abs();
        assert!(difference < MAX_DIFFERENCE_DEGREES, "julian_date={julian_date}, longitude={longitude}, reference_longitude={reference_longitude}, difference={difference}");
    }
}

#[test]
fn matches_reference_moon_longitudes_across_supported_centuries() {
    // NASA JPL Horizons DE441 apparent geocentric ecliptic-of-date longitude, quantity 31.
    // The tolerance covers nutation, light-time, and model differences from the mean geometric longitude.
    const MAX_DIFFERENCE_DEGREES: f64 = 0.02;
    let cases = [
        (2_415_021.0, 279.616_644_4), // 1900-01-01 12:00 TT
        (2_451_545.0, 223.314_855_7), // 2000-01-01 12:00 TT
        (2_488_070.0, 164.412_513_6), // 2100-01-01 12:00 TT
    ];

    for (julian_date, reference_longitude) in cases {
        let longitude = feature_angle(julian_date, Feature::MoonLongitude as u8);
        let difference = angular_difference(longitude, reference_longitude).abs();
        assert!(difference < MAX_DIFFERENCE_DEGREES, "julian_date={julian_date}, longitude={longitude}, reference_longitude={reference_longitude}, difference={difference}");
    }
}

#[test]
fn matches_reference_moon_phase_angles_across_supported_centuries() {
    // Differences between the NASA JPL Horizons Moon and Sun quantity 31 longitudes.
    const MAX_DIFFERENCE_DEGREES: f64 = 0.03;
    let cases = [
        (2_415_021.0, 358.953_400_6),
        (2_451_545.0, 302.946_703_8),
        (2_488_070.0, 243.299_678_3),
    ];

    for (julian_date, reference_phase_angle) in cases {
        let phase_angle = angle_at(julian_date, Feature::MoonPhaseAngle);
        let difference = angular_difference(phase_angle, reference_phase_angle).abs();
        assert!(difference < MAX_DIFFERENCE_DEGREES, "julian_date={julian_date}, phase_angle={phase_angle}, reference_phase_angle={reference_phase_angle}, difference={difference}");
    }
}

#[test]
fn matches_reference_2024_moon_phase_times() {
    // NASA GSFC phase times in UTC. The 15-minute tolerance includes UTC-to-TT conversion
    // and the documented accuracy of the lunar and solar position models.
    const MAX_DIFFERENCE_DAYS: f64 = 15.0 / 1440.0;
    let cases = [
        (0.0, 2_460_409.264_583_333_4),   // New Moon, 2024-04-08 18:21 UTC
        (90.0, 2_460_416.300_694_444),   // First Quarter, 2024-04-15 19:13 UTC
        (180.0, 2_460_424.492_361_111),  // Full Moon, 2024-04-23 23:49 UTC
        (270.0, 2_460_431.977_083_333_3), // Last Quarter, 2024-05-01 11:27 UTC
    ];

    for (target_phase_angle, reference_date) in cases {
        let phase_date = bisection_value_find(reference_date - 1.0, reference_date + 1.0, target_phase_angle, Feature::MoonPhaseAngle);
        assert!((phase_date - reference_date).abs() < MAX_DIFFERENCE_DAYS, "target_phase_angle={target_phase_angle}, phase_date={phase_date}, reference_date={reference_date}");
    }
}

#[test]
fn moon_features_are_monotonic_with_safe_steps_across_supported_centuries() {
    const START_JD: f64 = 2_415_020.5; // 1900-01-01
    const END_JD: f64 = 2_488_069.5; // 2100-01-01

    for feature in [Feature::MoonLongitude, Feature::MoonPhaseAngle] {
        let mut previous_date = START_JD;
        let mut previous_angle = angle_at(previous_date, feature);
        while previous_date < END_JD {
            let current_date = (previous_date + 1.0).min(END_JD);
            let current_angle = angle_at(current_date, feature);
            assert!(angular_difference(current_angle, previous_angle) > 0.0, "feature={feature:?}, previous_date={previous_date}, current_date={current_date}, previous_angle={previous_angle}, current_angle={current_angle}");
            previous_date = current_date;
            previous_angle = current_angle;
        }

        let step = coarse_step_for_feature(feature);
        let mut segment_start = START_JD;
        let mut start_angle = angle_at(segment_start, feature);
        while segment_start < END_JD {
            let segment_end = (segment_start + step).min(END_JD);
            let end_angle = angle_at(segment_end, feature);
            let displacement = angular_difference(end_angle, start_angle);
            assert!(displacement > 0.0 && displacement < 180.0, "feature={feature:?}, segment_start={segment_start}, segment_end={segment_end}, displacement={displacement}");
            segment_start = segment_end;
            start_angle = end_angle;
        }
    }
}

#[test]
fn returns_exact_value_boundaries_at_bisection_endpoints() {
    let start = 2_453_371.5;
    let end = start + 1.0;
    let start_longitude = feature_angle(start, 10);
    let end_longitude = feature_angle(end, 10);

    assert_eq!(bisection_value_find(start, end, start_longitude, Feature::SunLongitude), start);
    assert_eq!(bisection_value_find(start, end, end_longitude, Feature::SunLongitude), end);
}

#[test]
fn finds_narrow_and_wrapped_angular_constraints() {
    const SEARCH_START: f64 = 2_451_544.5; // 2000-01-01
    const SEARCH_END: f64 = 2_451_910.5; // 2001-01-01

    for (angle_start, angle_span) in [(15.0, 1.0), (350.0, 20.0)] {
        let results = search_refined_windows(SEARCH_START, SEARCH_END, &[Feature::SunLongitude as u8], &[angle_start], &[angle_span]);
        assert_eq!(results.len(), 2, "angle_start={angle_start}, angle_span={angle_span}, results={results:?}");
        assert!(angle_is_in_range(angle_at((results[0] + results[1]) * 0.5, Feature::SunLongitude), angle_start, angle_span), "angle_start={angle_start}, angle_span={angle_span}, results={results:?}");
        assert!(!angle_is_in_range(angle_at(results[0] - 2.0 / 1440.0, Feature::SunLongitude), angle_start, angle_span), "angle_start={angle_start}, angle_span={angle_span}, results={results:?}");
        assert!(!angle_is_in_range(angle_at(results[1] + 2.0 / 1440.0, Feature::SunLongitude), angle_start, angle_span), "angle_start={angle_start}, angle_span={angle_span}, results={results:?}");
    }
}

#[test]
fn accepts_a_full_circle_constraint() {
    const SEARCH_START: f64 = 2_451_544.5;
    const SEARCH_END: f64 = 2_451_910.5;

    assert_eq!(search_refined_windows(SEARCH_START, SEARCH_END, &[Feature::SunLongitude as u8], &[0.0], &[360.0]), vec![SEARCH_START, SEARCH_END]);
}

#[test]
fn finds_narrow_angular_constraints_for_every_feature() {
    const SELECTED_DATE: f64 = 2_451_727.5; // 2000-07-02

    for feature_id in [0, 1, 3, 4, 5, 6, 7, 10, 11, 16] {
        let feature = feature_from_id(feature_id);
        let selected_angle = angle_at(SELECTED_DATE, feature);
        let angle_start = (selected_angle - 0.5).rem_euclid(360.0);
        let angle_span = 1.0;
        let search_radius = coarse_step_for_feature(feature) * 2.0;
        let results = search_refined_windows(SELECTED_DATE - search_radius, SELECTED_DATE + search_radius, &[feature_id], &[angle_start], &[angle_span]);

        assert!(date_is_in_results(SELECTED_DATE, &results), "feature={feature:?}, angle_start={angle_start}, angle_span={angle_span}, results={results:?}");
        for window in results.chunks_exact(2) {
            let midpoint = (window[0] + window[1]) * 0.5;
            assert!(angle_is_in_range(angle_at(midpoint, feature), angle_start, angle_span), "feature={feature:?}, angle_start={angle_start}, angle_span={angle_span}, results={results:?}");
        }
    }
}

#[test]
fn treats_returned_windows_as_half_open() {
    let results = search_sign_windows(2_453_371.5, 2_453_736.5, &[10], &[5]); // Sun in Virgo

    assert_eq!(results.len(), 2);
    assert!(date_is_in_results(results[0], &results));
    assert!(!date_is_in_results(results[1], &results));
}

#[test]
fn merges_only_adjacent_or_overlapping_windows() {
    let mut windows = vec![(1.0, 2.0), (2.0, 3.0), (2.5, 4.0), (5.0, 6.0)];

    merge_adjacent_or_overlapping_windows(&mut windows);

    assert_eq!(windows, vec![(1.0, 4.0), (5.0, 6.0)]);
}

#[test]
fn uses_safe_coarse_step_for_each_supported_feature() {
    assert_eq!(coarse_step_for_feature(Feature::MercuryLongitude), 3.5);
    assert_eq!(coarse_step_for_feature(Feature::VenusLongitude), 12.0);
    assert_eq!(coarse_step_for_feature(Feature::MarsLongitude), 18.0);
    assert_eq!(coarse_step_for_feature(Feature::JupiterLongitude), 60.0);
    assert_eq!(coarse_step_for_feature(Feature::SaturnLongitude), 67.0);
    assert_eq!(coarse_step_for_feature(Feature::UranusLongitude), 75.0);
    assert_eq!(coarse_step_for_feature(Feature::NeptuneLongitude), 78.0);
    assert_eq!(coarse_step_for_feature(Feature::SunLongitude), 28.0);
    assert_eq!(coarse_step_for_feature(Feature::MoonLongitude), 10.0);
    assert_eq!(coarse_step_for_feature(Feature::MoonPhaseAngle), 10.0);
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
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[150.0], &[30.0]).is_ok());
    assert!(validate_search_inputs(f64::NAN, 2_453_736.5, &[10], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, f64::INFINITY, &[10], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_736.5, 2_453_371.5, &[10], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_371.5, &[10], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[], &[], &[]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10, 0], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[150.0, 0.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[2], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[11], &[150.0], &[30.0]).is_ok());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[16], &[150.0], &[30.0]).is_ok());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[12], &[150.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[f64::NAN], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[-1.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[360.0], &[30.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[150.0], &[f64::NAN]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[150.0], &[0.0]).is_err());
    assert!(validate_search_inputs(2_453_371.5, 2_453_736.5, &[10], &[150.0], &[360.1]).is_err());
}

#[test]
fn enforces_search_date_range_without_clipping() {
    assert_eq!(search_date_range(), vec![SEARCH_START_JD_TT, SEARCH_END_JD_TT]);
    assert!(validate_search_inputs(SEARCH_START_JD_TT, SEARCH_END_JD_TT, &[10], &[0.0], &[360.0]).is_ok());
    let before_start = f64::from_bits(SEARCH_START_JD_TT.to_bits() - 1);
    let after_end = f64::from_bits(SEARCH_END_JD_TT.to_bits() + 1);
    assert!(validate_search_inputs(before_start, SEARCH_END_JD_TT, &[10], &[0.0], &[360.0]).is_err());
    assert!(validate_search_inputs(SEARCH_START_JD_TT, after_end, &[10], &[0.0], &[360.0]).is_err());
    assert!(validate_search_inputs(SEARCH_END_JD_TT, after_end, &[10], &[0.0], &[360.0]).is_err());
    assert!(validate_search_inputs(SEARCH_END_JD_TT - 1.0, SEARCH_END_JD_TT, &[10], &[0.0], &[30.0]).is_ok());
}

#[test]
fn refines_prograde_and_retrograde_zero_crossings() {
    let mut previous_date = 2_451_544.5; // 2000-01-01
    let mut previous_longitude = feature_angle(previous_date, 0); // Mercury
    let mut prograde_crossing = None;
    let mut retrograde_crossing = None;

    for day in 1..=(365 * 50) {
        let current_date = 2_451_544.5 + day as f64;
        let current_longitude = feature_angle(current_date, 0);
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
        let crossing = bisection_value_find(start, end, 0.0, Feature::MercuryLongitude);
        let longitude = feature_angle(crossing, 0);
        assert!(angular_difference(longitude, 0.0).abs() < 0.001);
    }
}

#[test]
fn matches_reference_retrograde_stations() {
    // Reference times are rounded from NASA JPL Horizons DE441 geometric geocentric vectors.
    // A half-day tolerance allows for the reference-frame difference between J2000 and of-date ecliptics.
    let cases = [
        (0, 2_460_527.5, 2_460_528.5, 2_460_527.70), // Mercury stations retrograde, 2024-08-05
        (0, 2_460_550.5, 2_460_551.5, 2_460_551.37), // Mercury stations direct, 2024-08-28
        (1, 2_459_567.0, 2_459_569.0, 2_459_567.94), // Venus stations retrograde, 2021-12-19
        (1, 2_459_608.0, 2_459_610.0, 2_459_608.87), // Venus stations direct, 2022-01-29
        (3, 2_459_882.0, 2_459_884.0, 2_459_883.06), // Mars stations retrograde, 2022-10-30
        (3, 2_459_956.0, 2_459_958.0, 2_459_957.37), // Mars stations direct, 2023-01-12
    ];

    for (feature_id, bracket_start, bracket_end, reference_date) in cases {
        let feature = feature_from_id(feature_id);
        assert!(segment_has_interior_station(instantaneous_velocity(bracket_start, feature), instantaneous_velocity(bracket_end, feature)));
        let station_date = bisection_derivative_find_zero(bracket_start, bracket_end, feature);
        assert!((station_date - reference_date).abs() < 0.5, "feature_id={feature_id}, station_date={station_date}, reference_date={reference_date}");
    }
}

#[test]
fn returns_separate_sign_windows_during_mercury_retrograde() {
    const SEARCH_START: f64 = 2_460_511.5; // 2024-07-20
    const SEARCH_END: f64 = 2_460_568.5; // 2024-09-15

    let results = search_sign_windows(SEARCH_START, SEARCH_END, &[0], &[5]); // Mercury in Virgo
    let context = format!("2024 Mercury retrograde results={results:?}");

    assert_eq!(results.len(), 4, "expected separate Virgo entries around the 2024 Mercury retrograde; results={results:?}");
    assert!((2_460_516.5..2_460_519.5).contains(&results[0]), "unexpected first Virgo entry; results={results:?}");
    assert!((2_460_536.5..2_460_539.5).contains(&results[1]), "unexpected retrograde Virgo exit; results={results:?}");
    assert!((2_460_561.5..2_460_565.5).contains(&results[2]), "unexpected second Virgo entry; results={results:?}");
    assert_eq!(results[3], SEARCH_END);
    assert_result_boundaries_match_direct_evaluation(SEARCH_START, SEARCH_END, &[0], &[5], &results, &context);
}

#[test]
fn clips_open_window_to_search_boundaries() {
    let search_start = 2_466_674.835_774_712;
    let search_end = 2_468_099.372_874_511;

    let results = search_sign_windows(search_start, search_end, &[6], &[4]); // Uranus in Leo

    assert_eq!(results, vec![search_start, search_end]);
}

#[test]
fn clips_adjacent_partition_results_to_the_same_boundary() {
    let search_start = 2_453_371.5; // 2005-01-01
    let partition = 2_453_620.5;
    let search_end = 2_453_736.5; // 2006-01-01

    let complete = search_sign_windows(search_start, search_end, &[10], &[5]); // Sun in Virgo
    let left = search_sign_windows(search_start, partition, &[10], &[5]);
    let right = search_sign_windows(partition, search_end, &[10], &[5]);

    assert_eq!(complete.len(), 2);
    assert_eq!(left.len(), 2);
    assert_eq!(right.len(), 2);
    assert_eq!(left[1], partition);
    assert_eq!(right[0], partition);
    assert!((left[0] - complete[0]).abs() < 1.0 / 1440.0);
    assert!((right[1] - complete[1]).abs() < 1.0 / 1440.0);
}

#[test]
fn searches_sun_before_other_features_regardless_of_input_order() {
    let search_start = 2_451_544.5; // 2000-01-01
    let search_end = 2_451_910.5; // 2001-01-01
    let selected_date = (search_start + search_end) * 0.5;
    let mercury_sign = (feature_angle(selected_date, 0) / 30.0) as u8;
    let sun_sign = (feature_angle(selected_date, 10) / 30.0) as u8;

    let sun_first = search_sign_windows(search_start, search_end, &[10, 0], &[sun_sign, mercury_sign]);
    let sun_last = search_sign_windows(search_start, search_end, &[0, 10], &[mercury_sign, sun_sign]);

    assert_eq!(sun_last, sun_first);
}

#[test]
fn sun_fast_path_matches_daily_evaluation_for_every_sign() {
    let search_start = 2_451_544.5; // 2000-01-01
    let search_end = 2_451_910.5; // 2001-01-01

    for sign in 0..12 {
        let results = search_sign_windows(search_start, search_end, &[10], &[sign]);
        let mut date = search_start;
        while date < search_end {
            let expected = angle_is_in_range(feature_angle(date, 10), sign as f64 * 30.0, 30.0);
            let actual = date_is_in_results(date, &results);
            assert_eq!(actual, expected, "Sun sign={sign}, julian_date={date}, results={results:?}");
            date += 1.0;
        }
    }
}

#[test]
#[ignore = "manual release profiling"]
fn profile_search_execution() {
    assert!(!cfg!(debug_assertions), "run this profiler with cargo test --release profile_search_execution -- --ignored --nocapture --test-threads=1");

    const PROFILE_START: f64 = 2_451_544.5; // 2000-01-01
    const PROFILE_END: f64 = 2_455_197.5; // 2010-01-01
    const PROFILE_MIDPOINT: f64 = (PROFILE_START + PROFILE_END) * 0.5;
    const FEATURE_IDS: [u8; 10] = [10, 11, 16, 0, 1, 3, 4, 5, 6, 7];

    let first_evaluation_start = Instant::now();
    black_box(feature_angle(PROFILE_MIDPOINT, 0));
    let first_evaluation_time = first_evaluation_start.elapsed();

    black_box(search_sign_windows(PROFILE_START, PROFILE_START + 365.0, &[10], &[0]));

    let validation_feature_ids = [10];
    let validation_angle_starts = [0.0];
    let validation_angle_spans = [30.0];
    let validation_start = Instant::now();
    for _ in 0..100_000 {
        black_box(validate_search_inputs(black_box(PROFILE_START), black_box(PROFILE_END), black_box(&validation_feature_ids), black_box(&validation_angle_starts), black_box(&validation_angle_spans)).unwrap());
    }
    let validation_time = validation_start.elapsed();

    eprintln!("profile range: 2000-01-01 through 2010-01-01");
    eprintln!("first Mercury longitude evaluation: {first_evaluation_time:?}");
    eprintln!("100000 input validations: {validation_time:?}");

    for feature_id in FEATURE_IDS {
        let evaluations_started = Instant::now();
        for offset in 0..100 {
            black_box(feature_angle(PROFILE_MIDPOINT + offset as f64, feature_id));
        }
        eprintln!("100 warmed feature {feature_id} longitude evaluations: {:?}", evaluations_started.elapsed());
    }

    for feature_id in FEATURE_IDS {
        let feature_sign = (feature_angle(PROFILE_MIDPOINT, feature_id) / 30.0) as u8;
        let started = Instant::now();
        let results = black_box(search_sign_windows(PROFILE_START, PROFILE_END, &[feature_id], &[feature_sign]));
        eprintln!("single feature {feature_id}: {:?}, {} day coarse step, {} windows", started.elapsed(), coarse_step_for_feature(feature_from_id(feature_id)), results.len() / 2);
    }

    let feature_signs: Vec<u8> = FEATURE_IDS.iter().map(|&feature_id| (feature_angle(PROFILE_MIDPOINT, feature_id) / 30.0) as u8).collect();
    let total_search_start = Instant::now();
    let total_results = black_box(search_sign_windows(PROFILE_START, PROFILE_END, &FEATURE_IDS, &feature_signs));
    let total_search_time = total_search_start.elapsed();

    let mut windows = vec![(PROFILE_START, PROFILE_END)];
    let staged_search_start = Instant::now();
    for (&feature_id, &feature_sign) in FEATURE_IDS.iter().zip(&feature_signs) {
        let input_window_count = windows.len();
        let stage_start = Instant::now();
        let mut filtered_windows = Vec::new();
        for &(window_start, window_end) in &windows {
            filter_window_for_constraint(window_start, window_end, feature_from_id(feature_id), feature_sign as f64 * 30.0, 30.0, &mut filtered_windows);
        }
        merge_adjacent_or_overlapping_windows(&mut filtered_windows);
        let stage_time = stage_start.elapsed();
        eprintln!("staged feature {feature_id}: {stage_time:?}, {input_window_count} input windows, {} output windows", filtered_windows.len());
        windows = filtered_windows;
    }

    eprintln!("complete ten feature search: {total_search_time:?}, {} windows", total_results.len() / 2);
    eprintln!("sum of separately timed search stages: {:?}, {} windows", staged_search_start.elapsed(), windows.len());

    let sun_last_feature_ids = [11, 16, 0, 1, 3, 4, 5, 6, 7, 10];
    let sun_last_feature_signs: Vec<u8> = sun_last_feature_ids.iter().map(|&feature_id| (feature_angle(PROFILE_MIDPOINT, feature_id) / 30.0) as u8).collect();
    profile_repeated("repeated Sun only search", 10, || { black_box(search_sign_windows(PROFILE_START, PROFILE_END, &[10], &[feature_signs[0]])); });
    profile_repeated("repeated ten feature Sun first search", 10, || { black_box(search_sign_windows(PROFILE_START, PROFILE_END, &FEATURE_IDS, &feature_signs)); });
    profile_repeated("repeated ten feature Sun last search", 10, || { black_box(search_sign_windows(PROFILE_START, PROFILE_END, &sun_last_feature_ids, &sun_last_feature_signs)); });
}

#[test]
fn randomized_search_matches_direct_evaluation() {
    const CASE_COUNT: usize = 100;
    const MIN_SEARCH_RADIUS_DAYS: f64 = 180.0;
    const MAX_SEARCH_RADIUS_DAYS: f64 = 1_800.0;
    const REFERENCE_STEP_DAYS: f64 = 1.0;
    const REFERENCE_START_JD: f64 = 2_415_020.5; // 1900-01-01
    const REFERENCE_END_JD: f64 = 2_488_069.5; // 2100-01-01
    const SUPPORTED_FEATURES: [u8; 10] = [0, 1, 3, 4, 5, 6, 7, 10, 11, 16];

    struct ResolutionSample {
        width_days: f64,
        feature_count: usize,
        feature_ids: Vec<u8>,
    }

    fn next_random(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }

    fn median(values: &[f64]) -> f64 {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        if sorted.len() % 2 == 0 { (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) * 0.5 } else { sorted[sorted.len() / 2] }
    }

    fn geometric_mean(values: &[f64]) -> f64 {
        (values.iter().map(|value| value.ln()).sum::<f64>() / values.len() as f64).exp()
    }

    fn feature_name(feature_id: u8) -> &'static str {
        match feature_id {
            0 => "Mercury",
            1 => "Venus",
            3 => "Mars",
            4 => "Jupiter",
            5 => "Saturn",
            6 => "Uranus",
            7 => "Neptune",
            10 => "Sun",
            11 => "Moon",
            16 => "Moon phase",
            _ => "Unknown",
        }
    }

    let mut random_state = 0x4e41_5441_4c43_4841_u64;
    let warm_up_start = Instant::now();
    black_box(search_sign_windows(2_451_544.5, 2_451_909.5, &[10], &[0]));
    let warm_up_time = warm_up_start.elapsed();
    let mut total_search_time = Duration::ZERO;
    let mut total_boundary_check_time = Duration::ZERO;
    let mut total_reference_time = Duration::ZERO;
    let mut resolution_samples = Vec::with_capacity(CASE_COUNT);

    for case_index in 0..CASE_COUNT {
        let random_fraction = next_random(&mut random_state) as f64 / u64::MAX as f64;
        let selected_date = REFERENCE_START_JD + MAX_SEARCH_RADIUS_DAYS + random_fraction * (REFERENCE_END_JD - REFERENCE_START_JD - 2.0 * MAX_SEARCH_RADIUS_DAYS);
        let radius_fraction = next_random(&mut random_state) as f64 / u64::MAX as f64;
        let search_radius = MIN_SEARCH_RADIUS_DAYS + radius_fraction * (MAX_SEARCH_RADIUS_DAYS - MIN_SEARCH_RADIUS_DAYS);
        let search_start = selected_date - search_radius;
        let search_end = selected_date + search_radius;

        let mut shuffled_features = SUPPORTED_FEATURES;
        for index in (1..shuffled_features.len()).rev() {
            let swap_index = next_random(&mut random_state) as usize % (index + 1);
            shuffled_features.swap(index, swap_index);
        }

        let feature_count = next_random(&mut random_state) as usize % SUPPORTED_FEATURES.len() + 1;
        let feature_ids = &shuffled_features[..feature_count];
        let feature_signs: Vec<u8> = feature_ids.iter().map(|&feature_id| (feature_angle(selected_date, feature_id) / 30.0) as u8).collect();

        let search_timer = Instant::now();
        let results = search_sign_windows(search_start, search_end, feature_ids, &feature_signs);
        total_search_time += search_timer.elapsed();
        let context = format!("case={case_index}, selected_date={selected_date}, search=[{search_start}, {search_end}], feature_ids={feature_ids:?}, feature_signs={feature_signs:?}, results={results:?}");

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

        let matching_window = results.chunks_exact(2).find(|window| selected_date >= window[0] && selected_date < window[1]).unwrap_or_else(|| panic!("the generated source date must be returned; {context}"));
        resolution_samples.push(ResolutionSample { width_days: matching_window[1] - matching_window[0], feature_count, feature_ids: feature_ids.to_vec() });
        let boundary_check_timer = Instant::now();
        assert_result_boundaries_match_direct_evaluation(search_start, search_end, feature_ids, &feature_signs, &results, &context);
        total_boundary_check_time += boundary_check_timer.elapsed();

        let reference_timer = Instant::now();
        let mut reference_date = search_start;
        while reference_date <= search_end {
            let expected = date_is_valid(reference_date, feature_ids, &feature_signs);
            let actual = date_is_in_results(reference_date, &results);
            assert_eq!(actual, expected, "optimized result disagrees with direct sign evaluation at julian_date={reference_date}; {context}");
            reference_date += REFERENCE_STEP_DAYS;
        }
        total_reference_time += reference_timer.elapsed();
    }

    eprintln!("randomized verifier timing: warm_up={warm_up_time:?}, searches={total_search_time:?}, boundary_checks={total_boundary_check_time:?}, daily_reference={total_reference_time:?}");
    eprintln!("randomized verifier resolution by feature count:");
    for feature_count in 1..=SUPPORTED_FEATURES.len() {
        let widths: Vec<f64> = resolution_samples.iter().filter(|sample| sample.feature_count == feature_count).map(|sample| sample.width_days).collect();
        eprintln!("features={feature_count}, cases={}, mean_days={:.6}, median_days={:.6}, min_days={:.6}, max_days={:.6}", widths.len(), mean(&widths), median(&widths), widths.iter().copied().reduce(f64::min).unwrap(), widths.iter().copied().reduce(f64::max).unwrap());
    }

    eprintln!("randomized verifier resolution by included feature:");
    for feature_id in SUPPORTED_FEATURES {
        let widths_when_included: Vec<f64> = resolution_samples.iter().filter(|sample| sample.feature_ids.contains(&feature_id)).map(|sample| sample.width_days).collect();
        let mut weighted_log_ratio = 0.0;
        let mut total_weight = 0.0;
        // Compare queries with and without this feature only within equal-sized groups. Geometric means limit distortion from the very long outer-planet windows.
        for feature_count in 1..SUPPORTED_FEATURES.len() {
            let included: Vec<f64> = resolution_samples.iter().filter(|sample| sample.feature_count == feature_count && sample.feature_ids.contains(&feature_id)).map(|sample| sample.width_days).collect();
            let excluded: Vec<f64> = resolution_samples.iter().filter(|sample| sample.feature_count == feature_count && !sample.feature_ids.contains(&feature_id)).map(|sample| sample.width_days).collect();
            if !included.is_empty() && !excluded.is_empty() {
                let weight = included.len().min(excluded.len()) as f64;
                weighted_log_ratio += (geometric_mean(&included) / geometric_mean(&excluded)).ln() * weight;
                total_weight += weight;
            }
        }
        let same_count_width_ratio = (weighted_log_ratio / total_weight).exp();
        eprintln!("feature={}({feature_id}), included_cases={}, mean_days={:.6}, median_days={:.6}, same_count_width_ratio={same_count_width_ratio:.6}", feature_name(feature_id), widths_when_included.len(), mean(&widths_when_included), median(&widths_when_included));
    }
}

#[test]
fn preserves_subminute_windows_with_second_level_refinement() {
    let date = 2451545.0;
    let feature = Feature::SunLongitude;
    let start = angle_at(date, feature);
    let end = angle_at(date + 20.0 / 86400.0, feature);
    let windows = search_refined_windows(date - 1.0, date + 1.0, &[feature as u8], &[start], &[angular_difference(end, start)]);
    assert_eq!(windows.len(), 2);
    assert!((windows[0] - date).abs() * 86400.0 < 0.5);
    assert!((windows[1] - date).abs() * 86400.0 > 19.5);
    assert!((windows[1] - date).abs() * 86400.0 < 20.5);
}
