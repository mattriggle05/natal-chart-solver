# Browser ephemeris benchmark results

## Current decision

**Keep the production backend in Rust.** On 2026-09-15, Matt decided to finish testing, correctness work, and a working product in Rust for ease of understanding and iteration. A C/WASM worker or translation to C is deferred as a future optimization.

Astronomy Engine C/WASM was the fastest overall candidate in this experiment: about 56× baseline mixed throughput and 1.5× Astronomy Engine JavaScript, with a representative search about 16× faster than production and 2.8× faster than JavaScript. These compare different astronomical implementations, not an inherent C-versus-Rust language advantage. Retain the measurements for a later decision; they do not justify changing production now.

**Production search code is unchanged.** Next, establish the current Rust backend's time/accuracy contract and expand planetary JPL and complete-window verification near boundaries and stations. The sampled model disagreements below are not independent absolute error bounds, and the Sun/Moon search does not establish planetary or retrograde window agreement. Small longitude differences near stations can shift, create, or remove matching windows. One-minute numerical refinement does not certify one-minute astronomical accuracy.

## Reproducibility and scope

Recorded throughput run: `2026-09-14T02:33:04.673Z`. Finalist search run: `2026-09-14T02:37:42.271Z`.
Browser: `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/152.0.0.0 Safari/537.36`; reported logical processors: 10. Host macOS 26.6, arm64. Power mode, thermal state, and competing OS activity were not controlled.

Rust 1.94.0, wasm-pack 0.14.0, Emscripten 4.0.15, Astronomy Engine 2.1.19 at `865d3da7d8112bbc7911238052c6af4aaf877181`; exact Rust dependencies are locked in the lab. See [README.md](README.md) for builds, coordinate conventions, and limitations.

Nine randomized trials per engine/body use the identical deterministic 2,000-date corpus. Calibration selects a fixed repeat count per path to target 60 ms; repetitions execute entirely inside WASM or the JavaScript worker. Rates account for those repeats. All measured batches lasted at least 58.1 ms. p95 is the nearest-rank maximum of nine trials.

Exploratory runs showed substantial changes in absolute wall-clock timings. This report uses the final calibrated record and compares engines within each experiment. Do not extrapolate these absolute times into device-wide latency guarantees or compare a mixed longitude batch directly with the separately measured search. Mixed-path engine ordering persisted, but close per-body differences are not a reason to add complexity.

Raw records: [throughput](results/throughput.json), [searches](results/search.json), [asset sizes and hashes](results/asset-sizes.json).

## Warmed longitude throughput

| Feature | Baseline eval/s | astro eval/s | AE JavaScript eval/s | AE C/WASM eval/s |
|---|---:|---:|---:|---:|
| Mercury | 1,848 | 2,242 | 160,064 | 181,504 |
| Venus | 2,691 | 4,081 | 132,565 | 156,076 |
| Mars | 1,459 | 2,203 | 103,699 | 96,919 |
| Jupiter | 1,825 | 2,740 | 103,599 | 109,430 |
| Saturn | 1,449 | 2,128 | 119,284 | 116,054 |
| Uranus | 1,492 | 3,114 | 91,519 | 126,716 |
| Neptune | 2,564 | 4,063 | 131,868 | 161,399 |
| Sun | 6,948 | 7,424 | 154,895 | 193,012 |
| Moon | 79,523 | 80,999 | 35,907 | 196,477 |
| Moon phase | 6,400 | 6,299 | 30,288 | 97,087 |
| Mixed | 2,269 | 3,220 | 82,645 | 127,389 |

Median and p95 batch durations, repeat counts, and checksums are retained in the raw record. Because repeat counts differ, compare evaluations/sec rather than unnormalized batch milliseconds. The mixed path weights all ten features equally; it is a proxy, not a trace of production filter costs.

## Initialization and transferred assets

| Engine | Fresh worker ready ms | Module init ms | Raw bytes | Gzip bytes | Brotli bytes |
|---|---:|---:|---:|---:|---:|
| Current VSOP87 Rust/WASM | 6.7 | 3.8 | 1,565,216 | 1,103,830 | 755,050 |
| astro Rust/WASM | 18.4 | 8.7 | 1,616,051 | 862,443 | 533,703 |
| Astronomy Engine JavaScript | 11.5 | 7.0 | 414,730 | 108,940 | 86,275 |
| Astronomy Engine C/WASM | 5.6 | 2.7 | 62,892 | 35,293 | 32,457 |

Initialization is one fresh worker/module observation per engine with the browser cache in its current state; it is not a fresh-browser/network-cold distribution. Gzip uses level 9, Brotli quality 11. Sizes include each engine’s adapter and search code where implemented, excluding the common harness. C is dead-code-eliminated; JavaScript is upstream unminified ESM. Production bundle sizes will differ.

## Accuracy over 1900–2100

Geometric mean ecliptic-of-date longitude at identical TT dates, circular differences against baseline over 1,001 dates including endpoints. This is model disagreement, not independent absolute error.

| Feature | astro max ° | AE max ° | AE RMS ° |
|---|---:|---:|---:|
| Mercury | 0.00000126 | 0.00314881 | 0.00072597 |
| Venus | 0.00000279 | 0.00565406 | 0.00078592 |
| Mars | 0.00000621 | 0.00297711 | 0.00058677 |
| Jupiter | 0.00000264 | 0.00290362 | 0.00093462 |
| Saturn | 0.00000462 | 0.00348685 | 0.00116039 |
| Uranus | 0.00000644 | 0.00343539 | 0.00109095 |
| Neptune | 0.00000241 | 0.00500040 | 0.00280850 |
| Sun | 0.00000000 | 0.00064118 | 0.00018865 |
| Moon | 0.00000000 | 0.00307739 | 0.00075188 |
| Moon phase | 0.00000000 | 0.00285421 | 0.00077005 |

The JavaScript and C accuracy summaries agree to floating-point precision. The maximum Astronomy Engine disagreement is 0.00565406° (20.35 arcseconds), below the candidate one-arcminute target in this sample. astro stays within 0.00000645° (0.024 arcseconds).

Existing JPL DE441 apparent quantity-31 fixtures are checked separately. Maximum absolute errors:

| Engine | Sun ° | Moon ° | Phase ° |
|---|---:|---:|---:|
| Current VSOP87 Rust/WASM | 0.00969177 | 0.00443791 | 0.00583644 |
| astro Rust/WASM | 0.00969177 | 0.00443791 | 0.00583644 |
| Astronomy Engine JavaScript | 0.00952235 | 0.00405158 | 0.00561787 |
| Astronomy Engine C/WASM | 0.00952235 | 0.00405158 | 0.00561787 |

All pass the existing 0.02° Sun/Moon and 0.03° phase tolerances. Errors include the intentional geometric/apparent distinction. No new authoritative planetary fixtures were added in this experiment; stations and narrow planetary angular constraints remain an acceptance gate.

## Complete finalist search

JD TT 2415021–2488070, Sun [150°,180°), then Moon [0°,30°). One warmup, nine randomized trials. The baseline calls the unchanged production search. Finalists implement its relevant monotonic coarse-step and one-minute bisection behavior in isolation; this does not validate a retrograde implementation.

| Engine | Median ms | p95 ms | Windows | Max boundary shift vs baseline, minutes |
|---|---:|---:|---:|---:|
| Current VSOP87 Rust/WASM | 197.6 | 198.5 | 244 | 0.0000 |
| Astronomy Engine JavaScript | 35.8 | 41.2 | 244 | 0.8789 |
| Astronomy Engine C/WASM | 12.6 | 12.7 | 244 | 0.8789 |

Both finalists return exactly identical windows to each other. Every engine returns 244 sorted, nonoverlapping windows and repeats its own results exactly. Shifts include numerical root-refinement differences as well as model differences. Only this query has this measured boundary agreement.

## Verification and next task

- All four release adapters built successfully using the documented build script.
- Browser runs completed; the saved-record audit checks completeness, finite/repeatable checksums, visibility, fixture tolerances, and valid matching finalist windows.
- Production `crate/`, `src/`, and package manifests remain unchanged. The existing expensive production test suite was not rerun; the experiment directly exercised its search as reference.
- Next: complete Rust correctness work, planetary truth fixtures, complete-window verification, and TT/UTC and supported-range semantics, then finish the working Rust product. Revisit C/WASM only as a future optimization.
- Optional Swiss Ephemeris and XALEN were not required to distinguish the four requested candidates.
