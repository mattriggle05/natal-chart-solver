use natal_chart_solver::search;

fn main() {
    let start_julian_date = 2_453_371.5; // 2005-01-01
    let end_julian_date = 2_453_736.5; // 2006-01-01
    let feature_ids = [10]; // Sun
    let feature_signs = [5]; // Virgo

    let results = search(start_julian_date, end_julian_date, &feature_ids, &feature_signs).expect("hardcoded search inputs must be valid");

    println!("{results:?}");
}
