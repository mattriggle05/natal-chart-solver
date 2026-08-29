use super::*;

fn date_is_valid(julian_date: f64, feature_ids: &[u8], feature_signs: &[u8]) -> bool {
    feature_ids.iter().zip(feature_signs).all(|(&feature_id, &feature_sign)| longitude_is_in_sign(geocentric_longitude(julian_date, feature_id), feature_sign))
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

#[test]
fn angular_difference_wraps_across_zero() {
    assert_eq!(angular_difference(1.0, 359.0), 2.0);
    assert_eq!(angular_difference(359.0, 1.0), -2.0);
    assert_eq!(angular_difference(0.0, 360.0), 0.0);
}

#[test]
fn maps_longitudes_to_half_open_signs() {
    let immediately_before_30 = f64::from_bits(30.0_f64.to_bits() - 1);
    let immediately_before_360 = f64::from_bits(360.0_f64.to_bits() - 1);

    assert!(longitude_is_in_sign(0.0, 0));
    assert!(longitude_is_in_sign(immediately_before_30, 0));
    assert!(!longitude_is_in_sign(30.0, 0));
    assert!(longitude_is_in_sign(30.0, 1));
    assert!(longitude_is_in_sign(immediately_before_360, 11));
    assert!(!longitude_is_in_sign(0.0, 11));
}

#[test]
fn identifies_zero_boundary_in_both_directions() {
    assert_eq!(sign_boundary_between(359.0, 1.0), 0.0);
    assert_eq!(sign_boundary_between(1.0, 359.0), 0.0);
}

#[test]
fn returns_exact_value_boundaries_at_bisection_endpoints() {
    let start = 2_453_371.5;
    let end = start + 1.0;
    let start_longitude = geocentric_longitude(start, 10);
    let end_longitude = geocentric_longitude(end, 10);

    assert_eq!(bisection_value_find(start, end, start_longitude, 10), start);
    assert_eq!(bisection_value_find(start, end, end_longitude, 10), end);
}

#[test]
fn treats_returned_windows_as_half_open() {
    let results = search_refined_windows(2_453_371.5, 2_453_736.5, &[10], &[5]); // Sun in Virgo

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
        assert!(segment_has_interior_station(instantaneous_velocity(bracket_start, feature_id), instantaneous_velocity(bracket_end, feature_id)));
        let station_date = bisection_derivative_find_zero(bracket_start, bracket_end, feature_id);
        assert!((station_date - reference_date).abs() < 0.5, "feature_id={feature_id}, station_date={station_date}, reference_date={reference_date}");
    }
}

#[test]
fn returns_separate_sign_windows_during_mercury_retrograde() {
    const SEARCH_START: f64 = 2_460_511.5; // 2024-07-20
    const SEARCH_END: f64 = 2_460_568.5; // 2024-09-15

    let results = search_refined_windows(SEARCH_START, SEARCH_END, &[0], &[5]); // Mercury in Virgo
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

    let mut random_state = 0x4e41_5441_4c43_4841_u64;

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
        let feature_signs: Vec<u8> = feature_ids.iter().map(|&feature_id| (geocentric_longitude(selected_date, feature_id) / 30.0) as u8).collect();

        let results = search_refined_windows(search_start, search_end, feature_ids, &feature_signs);
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

        assert!(date_is_in_results(selected_date, &results), "the generated source date must be returned; {context}");
        assert_result_boundaries_match_direct_evaluation(search_start, search_end, feature_ids, &feature_signs, &results, &context);

        let mut reference_date = search_start;
        while reference_date <= search_end {
            let expected = date_is_valid(reference_date, feature_ids, &feature_signs);
            let actual = date_is_in_results(reference_date, &results);
            assert_eq!(actual, expected, "optimized result disagrees with direct sign evaluation at julian_date={reference_date}; {context}");
            reference_date += REFERENCE_STEP_DAYS;
        }
    }
}
