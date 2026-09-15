# Browser ephemeris laboratory

Current decision (2026-09-15): finish correctness testing and a working product in Rust first. C/WASM is deferred as a future optimization; this laboratory preserves the experiment for later use.

This directory is isolated from the production frontend and search implementation. No candidate is wired into production. The baseline depends on the production Rust crate and calls its public `angle_at`; the other Rust build has no production dependency.

## Build and run

Requirements: Rust 1.94.0 with wasm32-unknown-unknown, wasm-pack 0.14.0, Node 24 and Emscripten 4.0.15. Activate Emscripten with its `emsdk_env.sh` first.

Clone https://github.com/cosinekitty/astronomy and check out `865d3da7d8112bbc7911238052c6af4aaf877181`. Then:

```sh
ASTRONOMY_SOURCE=/absolute/path/to/astronomy sh benchmarks/ephemeris/build.sh
node benchmarks/ephemeris/server.mjs
```

Open http://127.0.0.1:8766 and click Run benchmark. Keep the tab visible, avoid other CPU work, and download the JSON when complete. The local server also saves raw results to `results/throughput.json` and `results/search.json`; subsequent runs overwrite these two files. Generated dependencies and binaries are ignored. The build records individual and total raw, gzip level 9, and Brotli quality 11 sizes plus SHA256 hashes. The common worker/harness is excluded equally. JavaScript is the upstream unminified ESM distribution; compressed transfer sizes are the useful comparison. These are laboratory adapter sizes, not a production bundle prediction.

On this laptop the default MacOSX27 SDK fails linking with an unknown `arm64e.x1` architecture. Setting `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk` fixes builds without changing the system default.

## Method

Four dedicated module workers run sequentially: current VSOP87 Rust/WASM, astro Rust/WASM, Astronomy Engine JavaScript, Astronomy Engine C/WASM. Each timed request returns only elapsed milliseconds and checksum. Rust and C execute their full loops inside WASM with one call per trial. Accuracy requests are separate, untimed batches.

All engines receive the same deterministic sequence spanning JD TT 2415021–2488070 (1900–2100). There are 1,000 warmup evaluations per body path, then one calibration batch per path and nine randomized timed trials. Each path repeats the identical 2,000-date corpus enough times to target at least 60 ms (one to 100 repeats, chosen once before trials). Repeat loops stay inside Rust/C WASM or the JavaScript worker. Evaluation rates use the actual count including repeats; all engines evaluate the same corpus with different multiplicities. Calibration records and repeat counts are retained. The mixed path cycles evenly across the ten supported features, including Sun, Moon, and phase; it is a general workload proxy, not a measured production call distribution. Report median and nearest-rank p95 (with nine trials, p95 is the maximum). Retain raw trials, exact checksums, browser metadata, and visibility changes. Repeated runs reset the seed.

Initialization is measured before warmup: parent worker creation-to-ready and worker module import/initialization. It is a cold worker/module measurement in the current browser cache state, not a guaranteed cold HTTP or browser compilation cache measurement. Throughput does not include worker messaging.

Positions use geometric geocentric mean ecliptic/equinox of date, with TT provided explicitly. The Astronomy Engine adapters subtract simultaneous heliocentric Earth vectors, avoiding light time and aberration, then transform EQJ to true ecliptic of date and subtract nutation in longitude. The C adapter includes the pinned upstream source to access `e_tilt`; JavaScript exports it. That internal coupling is an integration cost. Moon uses each engine's own model (the baseline and astro share the existing astro lunar model).

Accuracy compares circular longitude differences against baseline over 1,001 evenly spaced dates, including endpoints, reporting maximum and RMS by feature. Baseline agreement is not independent truth. Existing JPL DE441 apparent quantity-31 fixtures for Sun, Moon, and phase are copied from `crate/src/tests.rs` and tested separately at the same TT dates. Their errors include intentional geometric/apparent convention differences. These fixtures do not establish planetary accuracy near stations or angular boundaries.

Engine selection must also consider complete representative searches for finalists, result-window agreement, maintenance, and the eventual accuracy contract. A fast microbenchmark alone does not authorize a production replacement.

## Finalist search

Click Run finalist searches after the throughput run. Astronomy Engine JavaScript and C/WASM are the performance finalists; the unchanged production Rust search is the reference. The query is Sun longitude [150°,180°) and Moon longitude [0°,30°), JD TT 2415021–2488070. The adapters implement the relevant monotonic search path with the same steps and one-minute root brackets; they return full windows. This is a complete two-constraint query, not a port or validation of the retrograde search path. C runs the whole search inside WASM. JavaScript runs the whole search inside its worker. Window retrieval is included in the reported time. One warmup and nine randomized timed searches are recorded, with exact per-engine repeatability and window-count/boundary comparisons against production.

## Audit saved results

```sh
node benchmarks/ephemeris/audit.mjs
```

This verifies record completeness, finite and repeatable checksums, unchanged visibility, the existing fixture tolerances, sampled Astronomy Engine disagreement below one arcminute, ordered valid date windows, and identical JavaScript/C finalist windows. These are checks of this experiment, not a global astronomical accuracy guarantee.
