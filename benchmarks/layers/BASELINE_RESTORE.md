# Local baseline restoration — October 11, 2026

The local production search now uses the geometric longitude paths from f5f65a8 and one-minute root brackets. Current date validation and range bounds remain. The apparent-position implementation remains compiled only for tests/diagnostics. Existing apparent-accuracy documentation describes the prior implementation and is not a valid claim for this local candidate.

Release WASM, TypeScript compilation, and Vite production build passed. The local server is listening on 127.0.0.1:5173. The worker measures search duration; the frontend displays computation and startup/formatting time. Neither includes React paint. No deployment or commit performed.

Direct Node WASM observations (milliseconds):

| Query | Process 1, run 1 | Process 1, run 2 | Process 2, run 1 | Process 2, run 2 |
|---|---:|---:|---:|---:|
| Sun Aries, 200 years | 13358.913 | 367.358 | 371.355 | 365.517 |
| Sun Virgo + Moon Aries, 200 years | 394.461 | 212.660 | 391.568 | 389.366 |
| Mercury Aries, 10 years | 548.375 | 827.514 | 1089.591 | 1092.257 |

Counts are 200, 244, and 11. The first 13.36-second observation is unexplained and was not reproduced in the second process; it is not discarded or attributed conclusively to startup. Host activity was uncontrolled. Browser verification was rejected by automatic approval review because of an unresolved prior usage-limit block. These are direct Node timings, not fresh frontend/browser timings.

Full Rust release suite: 25 passed, 5 failed, 3 ignored. Failures: apparent_positions_match_jpl_samples, apparent_moon_phase_matches_independent_jpl_positions, neptune_position_at_jpl_aries_ingress_stays_within_model_budget, complete_search_windows_match_fixed_jpl_references, preserves_subminute_windows_with_second_level_refinement. The first four conflict with the reverted coordinate/accuracy contract; the fifth exposes reduced precision for subminute intervals. The complete-window test stops at a 610.034-second Sun endpoint discrepancy, so no claim is made that every independent case retains its expected count. Tests were not weakened or disabled. This is a local performance candidate, not a fully passing release.

Build notices: wasm-bindgen prebuilt lookup failed but fallback succeeded; optional package metadata is absent; a newer wasm-pack is available. These did not prevent the build.
