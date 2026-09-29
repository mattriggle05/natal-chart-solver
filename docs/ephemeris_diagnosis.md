# Neptune boundary discrepancy diagnosis

Investigation date: 2026-09-15. This is an isolated diagnosis, not a completed accuracy certification or a new production engine selection.

## Finding

Both `astro 2.0.0` and `vsop87 2.1.0` reproduce the Neptune discrepancy. The dominant error is present in their underlying planetary positions before light-time, aberration, nutation, or root finding. Switching between these packages does not resolve it.

At the JPL 2025 Aries crossing, JD(TT) approximately 2460764.999763406, geometric geocentric J2000 longitude differs from JPL by:

| Position implementation | Error, arcseconds |
|---|---:|
| VSOP87E rectangular barycentric | +1.418404 |
| VSOP87A rectangular heliocentric | +1.418347 |
| astro VSOP87D spherical, transformed to J2000 | +1.417460 |

These comparisons include small differences between VSOP dynamical and ICRF frames; they are not claimed to be exactly identical reference axes. The substitution tests below isolate the much larger Neptune contribution.

The astro source uses VSOP87D planetary coefficients. An independent Python evaluation of all 681 longitude terms in the original Neptune VSOP87D coefficient file agrees with both crates' VSOP87D longitudes within 0.000001 arcsecond at 13 dates. This argues against a coefficient transcription or longitude summation bug in either package. VSOP87's historical calibration predates the current JPL solution.

## Controlled alternatives

The following experiments retain the same full apparent-position pipeline and change its coordinate provider. Negative time differences mean earlier than JPL. Roots are refined to 0.01 second; that is numerical precision, not a claim about physical accuracy.

| Provider change | Crossing difference |
|---|---:|
| VSOP87E, current implementation | −919.743 s |
| VSOP87A instead | −919.704 s |
| astro VSOP87D instead | −919.130 s |
| Substitute only JPL Earth positions | −914.074 s |
| Substitute only JPL Neptune positions | −17.386 s |
| Substitute JPL Earth, Neptune, and Sun positions | −11.717 s |
| JPL providers, Neptune system barycenter rather than center | −15.507 s |

The JPL substitutions are diagnostic: the unchanged pipeline still applies the VSOP-to-FK5 correction to those substituted coordinates. That correction is not appropriate as a general conversion of ICRF input. Disabling it for the all-JPL case leaves −70.114 seconds. Thus the 12-second residual is **not** evidence that a finished JPL-backed implementation has already achieved the target. The remaining frame/precession/nutation conventions must be aligned independently. Horizons uses EOP-corrected IAU76/80 transformations, whereas the current pipeline uses astro's analytical precession/nutation routines. Their complete equivalence has not been established.

Nevertheless, replacing Neptune removes about 902 seconds under otherwise identical processing, while replacing Earth removes only about 6 seconds. The main 15-minute discrepancy is in the Neptune ephemeris, not the boundary solver. Changing center to barycenter accounts for seconds, not 15 minutes.

Removing the FK5 correction from the VSOP87E case makes its error worse (−978.140 seconds). Using `astro::planet::geocent_apprnt_ecl_coords` directly also does not solve the issue: it corrects only light time and differs from JPL's full apparent longitude by 21.403 arcseconds at this epoch.

## Why a constant adjustment is unsuitable

The geometric Neptune discrepancy changes across the sampled range: about −0.105 arcsecond in 1900, +0.723 in 2000, +1.418 at this 2025 crossing, +2.171 in 2050, and +2.514 around 2075. Neither a fixed longitude offset nor a fixed 15-minute timestamp adjustment is justified. Near stations, a similar angular discrepancy can create a much larger time discrepancy.

## Recommended next implementation experiment

Keep Rust, but evaluate a modern JPL-derived position representation over the bounded 1900–2100 domain. The isolated substitutions demonstrate that better Neptune data addresses the dominant error. A production solution could evaluate ephemeris segments locally in Rust; it need not call Horizons during user searches. Data size, interpolation error, frame conventions, and complete window accuracy still need validation. No modern analytic replacement or production interpolation format has yet been benchmarked by this diagnosis.

## Reproduction and evidence

- `python3 scripts/diagnose_ephemeris.py`: downloads geometric JPL vectors and saves raw responses and request parameters under `fixtures/jpl/diagnosis/`.
- `python3 scripts/check_vsop_coefficients.py /path/to/VSOP87D.nep`: independently evaluates original coefficients; the generated source metadata records their URL and SHA-256.
- `cargo test --manifest-path crate/Cargo.toml --release packages_match_original_neptune_coefficients`
- `cargo test --manifest-path crate/Cargo.toml --release diagnose_neptune_models -- --ignored --nocapture`
- Diagnostic output: `fixtures/jpl/diagnosis/results.txt`.

JPL vector tables use TDB; the observer crossing uses TT. The millisecond-scale TT/TDB difference is neglected in this diagnostic and cannot account for the hundreds of seconds observed. A production implementation should handle or explicitly budget it. Dense JPL vector samples use cubic Hermite interpolation at 0.01-day spacing. Separate midpoint samples verify interpolation within 1e-10 AU for the tested Earth, Sun, Neptune-center, and Neptune-barycenter intervals.

References: [Horizons API](https://ssd-api.jpl.nasa.gov/doc/horizons.html), [Horizons frames and apparent-coordinate definitions](https://ssd.jpl.nasa.gov/horizons/manual.html), [original VSOP87 distribution documentation mirror](https://raw.githubusercontent.com/ctdk/vsop87/master/vsop87.txt).

## Follow-up: apparent pipeline review (2026-09-29)

The previously unexplained frame residual is accounted for at the diagnostic epoch. IERS/USNO `finals.all` gives Bulletin A dPsi corrections of −109.679 and −110.421 milliarcseconds at 2025-03-30 and 2025-03-31 UTC. Interpolating to the TT reference epoch converted to UTC supplies approximately −0.110049 arcsecond. Applying this independently sourced correction to the unadjusted JPL-position calculation's +0.109485 arcsecond residual leaves approximately −0.000564 arcsecond (about 0.36 second of crossing time). Source rows are retained in `fixtures/jpl/diagnosis/iers_nutation.txt`.

A regression test now exercises the actual production correction function with independent JPL vectors and that test-only EOP adjustment; its tolerance is 0.002 arcsecond. This resolves the discrepancy at this epoch without adding EOP downloads to production. Horizons' observed Earth-orientation adjustment and the application's analytical nutation remain explicitly different conventions. The older 12-second result obtained by applying the VSOP frame correction to JPL vectors should not be used as evidence of correct frame handling.

Production now rotates native VSOP vectors into FK5 at J2000 before combining them with the precessed lunar offset. Previously, the Moon's mean-of-date coordinates were combined with unconverted VSOP vectors and then received the VSOP correction at output. The common input frame avoids that inconsistency. Solar deflection now uses a solar-disk-based limiter at conjunction. Print-only diagnostic surveys are ignored manual tests, not routine acceptance tests. The retained `diagnostic_apparent` helper deliberately reproduces the earlier experiment and is test-only; it is not the production correction path.

The original model diagnosis is unchanged: both packages share the Neptune discrepancy. No fixed offset, new ephemeris dependency, or production data table was introduced.

Step 1 verification: full native release suite 29 passed, 0 failed, 4 ignored manual diagnostics/profiler; five TypeScript time tests passed; WASM, TypeScript, and Vite production builds passed. WASM output is 2,077.71 kB raw / 1,410.00 kB gzip. Full position acceptance fixtures and complete reference-window tests remain tasks 2 and 3. No commit was made during this review.
