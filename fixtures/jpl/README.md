# JPL position regression fixtures

The ordinary Rust test suite checks apparent longitudes for nine bodies at 12 fixed epochs: 108 body/epoch pairs. It also checks 12 Moon phase angles derived from the independent JPL Sun and Moon longitudes, and Neptune's position at its independently determined 2025 Aries ingress. The fixture loader rejects missing, duplicate, unknown, or reordered body/epoch coverage and invalid coordinates.

## Reference conventions and provenance

These are offline Horizons observer-table quantity 31 fixtures: Earth center `500@399`, target body centers, TT input/output, and airless apparent ecliptic-of-date coordinates. `manifest.json` records every request and the SHA-256 of each original JSON response. The responses retain the actual ephemeris source and coordinate descriptions; do not infer the source from the package being tested.

Dates include 1900-01-01 and 2100-01-01 TT, intermediate epochs, and several 2024 dates. The upper endpoint is an allowed ephemeris evaluation/exclusive search endpoint. These are representative position samples, not exhaustive coverage or a near-station/window accuracy certification.

Verify the existing references without network access:

```sh
python3 scripts/fetch_jpl.py --verify
cargo test --manifest-path crate/Cargo.toml --release jpl -- --nocapture
```

The Python check verifies response hashes, request conventions, requested/returned epochs, and exact reproduction of the retained responses by `positions.csv`. It does not generate references from production calculations. Run `python3 scripts/fetch_jpl.py` only when intentionally refreshing the source data; it requires network access and verifies the refreshed fixtures afterward. Review data and tolerance changes separately rather than automatically increasing limits when a test fails.

## Sampled regression budgets

Errors are shortest circular longitude differences in arcseconds. The measured maxima below use the reviewed apparent-position implementation (`0ad2c60`). All checks call the public Rust `angle_at` dispatch, including the derived phase feature.

| Feature | Maximum sampled error | Regression limit |
|---|---:|---:|
| Mercury | 0.103513 | 0.20 |
| Venus | 0.097074 | 0.20 |
| Mars | 0.092300 | 0.20 |
| Jupiter | 0.480681 | 0.75 |
| Saturn | 0.401042 | 0.75 |
| Uranus | 1.091874 | 1.50 |
| Neptune | 2.539385 | 3.00 |
| Sun | 0.097565 | 0.20 |
| Moon | 5.320064 | 10.00 |
| Moon phase | 5.275545 | 10.20 |

The planetary/Sun limits are rounded, per-body characterization budgets above the diagnosed discrepancies, replacing the old broad 72-arcsecond Sun/Moon and 108-arcsecond phase checks. They provide explicit regression thresholds, not independent theoretical accuracy bounds. The limits allow the known analytical ephemeris and Earth-orientation convention differences; they are not justified by floating-point noise. The Moon's 10-arcsecond budget follows the existing lunar model's documented longitude target and accommodates the measured residual. Phase uses the sum of the Moon and Sun budgets (triangle inequality). Neither is a newly certified whole-domain guarantee.

A test passes only if every sampled error is within its specified limit. The maxima are printed for diagnosis but printing alone is not acceptance. Passing these angular checks does not establish one-minute crossing accuracy: small angular errors can imply large timing errors during slow motion.

## Neptune model regression and focused frame check

`neptune_ingress.json` records independent JPL root queries; original responses are retained as `neptune_ingress_*.json`. Regenerate that investigation with `python3 scripts/probe_neptune.py` if needed. JPL's approximate root is JD(TT) 2460764.999763406, limited by the output angle precision rather than every displayed decimal digit.

At that epoch, the reviewed implementation is +1.436231 arcseconds past the Aries boundary. The new position regression uses Neptune's same 3-arcsecond budget; it does not require the model to retain its current error or claim a one-minute bound. The earlier direct crossing experiment measured about 919.75 seconds (15.33 minutes) early. The manual ingress timing probe remains ignored in ordinary tests; complete search-window acceptance is task 3.

The separate correction-pipeline test uses independent JPL vectors and a test-only observed IERS nutation adjustment to explain the much smaller frame discrepancy. Production loads neither JPL vectors nor EOP data. See [ephemeris_diagnosis.md](../../../docs/ephemeris_diagnosis.md) for the model diagnosis and the resolved Earth-orientation difference.
