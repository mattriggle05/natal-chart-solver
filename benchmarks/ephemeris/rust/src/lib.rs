use wasm_bindgen::prelude::*;
const BODIES: [u8; 10] = [0, 1, 3, 4, 5, 6, 7, 10, 11, 16];

#[cfg(feature = "baseline")]
fn angle(jd: f64, body: u8) -> f64 {
    natal_chart_solver::angle_at(jd, natal_chart_solver::Feature::try_from(body).unwrap())
}

#[cfg(not(feature = "baseline"))]
fn angle(jd: f64, body: u8) -> f64 {
    use astro::planet::{heliocent_coords, Planet};
    if body == 11 { return astro::lunar::geocent_ecl_pos(jd).0.long.to_degrees().rem_euclid(360.0); }
    if body == 16 { return (angle(jd, 11) - angle(jd, 10)).rem_euclid(360.0); }
    let (el, eb, er) = heliocent_coords(&Planet::Earth, jd);
    if body == 10 { return (el.to_degrees() + 180.0).rem_euclid(360.0); }
    let planet = match body { 0 => Planet::Mercury, 1 => Planet::Venus, 3 => Planet::Mars, 4 => Planet::Jupiter, 5 => Planet::Saturn, 6 => Planet::Uranus, 7 => Planet::Neptune, _ => panic!("unsupported body") };
    let (pl, pb, pr) = heliocent_coords(&planet, jd);
    (pr * pb.cos() * pl.sin() - er * eb.cos() * el.sin()).atan2(pr * pb.cos() * pl.cos() - er * eb.cos() * el.cos()).to_degrees().rem_euclid(360.0)
}

#[wasm_bindgen]
pub fn run(body: i32, count: u32, repeats: u32) -> f64 {
    let mut checksum = 0.0;
    for _ in 0..repeats {
        for i in 0..count {
            let jd = 2415021.0 + ((i as u64 * 104729) % 1000003) as f64 / 1000002.0 * 73049.0;
            checksum += angle(jd, if body < 0 { BODIES[i as usize % BODIES.len()] } else { body as u8 });
        }
    }
    checksum
}

// Accuracy sampling is a separate, untimed batch, never part of throughput trials.
#[wasm_bindgen]
pub fn sample(dates: &[f64], body: u8) -> Vec<f64> { dates.iter().map(|&jd| angle(jd, body)).collect() }

#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn representative_search() -> Vec<f64> {
    natal_chart_solver::search(2415021.0, 2488069.5, &[10,11], &[150.0,0.0], &[30.0,30.0]).unwrap()
}
