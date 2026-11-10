# Search layer comparison — October 10, 2026

**Current benchmark policy (November 10, 2026):** all search performance cases use the full 1900–2100 domain. The runnable Mercury case is now `mercury200`. The ten-year figures below and saved results are historical measurements, not current full-range performance evidence.

Purpose: isolate native calculation cost, direct WASM execution, and frontend overhead. No optimization or astronomical behavior change was made. Source under test: `f62d79a` (same tree as prior `25fdf4a`). Rust 1.94.0; Node 24.19.0; Codex in-app browser; existing release WASM SHA256 is in results.json. Benchmarks ran sequentially, not alongside one another; host power/thermal state and unrelated activity were not controlled.

## Results (seconds)

| Query | Native release Rust, two runs | Direct Node WASM, two runs | Direct browser WASM, two runs | Frontend compute / total, one run |
|---|---|---|---|---|
| Sun Aries, 1900–2100 | 2.2206–2.2402 | 5.7740–5.8113 | 5.9193–5.9582 | 6.0518 / 6.0725 |
| Sun Virgo + Moon Aries, 1900–2100 | 4.0692–4.1225 | 10.6575–11.1276 | 10.9511–11.0194 | 11.2424 / 11.2603 |
| Mercury Aries, 2000–2010 | 1.4842–1.4844 | 3.9159–3.9339 | 4.3089–4.3911 | 3.7333 / 3.7507 |

The full-domain endpoints are exported TT bounds 2415020.5–2488069.5. Mercury endpoints are civil-date conversions to TT: 2451544.5007428704–2455197.5007660184. Window counts are 200, 244, and 11 respectively. Output sums match across current layers to the displayed floating-point precision. Counts/sums are sanity checks, not a full endpoint-equivalence or independent accuracy test.

The frontend timings include temporary worker and hook instrumentation. Compute brackets the search call; total includes worker creation, loading, messaging and result-string formatting, but excludes React rendering and paint. Overhead is 17.4–20.7 ms. Direct WASM timings exclude module initialization and call the public wasm-bindgen search wrapper without React, the production hook, or date conversion. Each direct environment runs both repetitions in the same instance. Frontend searches create fresh workers. No large second-run improvement was observed. This does not establish behavior across all browsers or devices.

## Old source control

The library source from `f5f65a8` was compiled natively in an isolated /tmp project using the current Cargo manifest, lockfile, compiler and release settings, with the same input harness. This compares old calculation code, not an exact reconstruction of the September toolchain/binary.

| Query | Old native release, seconds | Current native / old, approximate |
|---|---|---|
| Sun Aries | 0.1623–0.1642 | 13.7x |
| Sun Virgo + Moon Aries | 0.1730–0.1733 | 23.7x |
| Mercury Aries, 10 years | 0.4162–0.4163 | 3.6x |

The old implementation returns the same counts for these cases but different endpoint sums, expected given geometric coordinates and older refinement. It is not an accuracy-preserving replacement. Native results establish that the slowdown exists independently of WASM. Current WASM execution adds roughly 2.5–3x native time in this sample. The roughly 56x historical browser comparison combines implementation changes and runtime/session differences; it is not a measured compiler defect.

Source inspection: current apparent positions repeatedly calculate Earth, target and Sun vectors; lunar target iterations also calculate Earth. Mercury's station detection uses finite-difference velocities requiring extra apparent positions. One-second refinement adds root iterations. These are concrete work increases, but this experiment does not apportion time among individual corrections, trigonometry, or search stages. A focused internal profile is the next step before optimizing or changing languages/models.

## Reproduction and retained data

- `raw.mjs` defines matched queries and direct WASM calls, shared by `node.mjs` and `worker.mjs`.
- Run `node benchmarks/layers/node.mjs` for raw Node WASM results.
- Start the existing Vite dev server; open `/natal-chart-solver/benchmarks/layers/` and press Run matched queries for browser results.
- `native.rs` is the manual native harness. Temporarily copy it to `crate/src/bin/layer_timing.rs`, run `cargo run --manifest-path crate/Cargo.toml --release --bin layer_timing`, then remove the temporary copy. On this machine builds require SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk.
- Raw captured values are in `results.json`, `native.txt`, and `old-native.txt`. Browser values were transcribed from visible harness output, rounded to 0.1 ms. Native and Node values came from command output.
- Temporary production frontend timing instrumentation was removed. The benchmark is not included in Vite's production HTML entry points. No deployment or commit was performed.

## Errors and verification limits

The first browser navigation failed because the local server was stopped; restarting it resolved the failure before browser timing. Initial Mercury native/Node inputs used TT midnight instead of converted civil midnight; those preliminary Mercury results were discarded and rerun with corrected dates. Current browser and frontend measurements used the corrected values. Two direct repetitions and one frontend observation per case are a small diagnostic sample, not a statistical latency certification. The old browser/WASM build was not rerun. Rendering, individual math-library costs and per-correction costs were not profiled. No correctness protection beyond the stated count/sum sanity checks is claimed; production algorithms were not changed.
