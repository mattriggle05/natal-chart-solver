# Natal Chart Solver Handoff

## Purpose

Natal Chart Solver is a browser application that reverse-searches dates from partial or complete natal-chart placements. A user supplies angular constraints for astronomical features, and the application returns every date window in which all constraints overlap.

The project is intentionally backend-first. The search engine, supported astronomical features, accuracy, and performance should be settled before the production frontend is designed around them.

## Repository state

- Repository: `https://github.com/mattriggle05/natal-chart-solver.git`
- Branch: `main`
- Remote: `origin/main`
- The original handoff was prepared from a clean tree synchronized with `origin/main`; check Git status and history for subsequent local commits.
- Read `docs/natal_chart_solver_design_doc.md` for architecture and numerical details.
- Read `docs/TODO.md` for the completion checklist.
- Use `git log --oneline` to recover the sequence and wording of completed work.

## Current architecture

- React 19 and TypeScript frontend built by Vite.
- Rust computation crate compiled to WebAssembly with `wasm-pack`.
- A Web Worker owns the WASM search module so searches do not block the UI.
- The application is static and deploys to GitHub Pages; there is no server backend.
- The public Rust entry point is `search` in `crate/src/lib.rs`.
- Search input is represented by three equal-length arrays: feature ID, angular start, and angular span.
- Rust converts numeric feature IDs to the internal `Feature` enum after validating every input.
- Results are a flat array of Julian-date pairs with `[start, end)` semantics.
- Tests are kept in `crate/src/tests.rs`; search implementation remains in `crate/src/lib.rs` by preference.

## Supported search features

| ID | Feature | Search path |
|---:|---|---|
| 0 | Mercury longitude | Retrograde-capable |
| 1 | Venus longitude | Retrograde-capable |
| 3 | Mars longitude | Retrograde-capable |
| 4 | Jupiter longitude | Retrograde-capable |
| 5 | Saturn longitude | Retrograde-capable |
| 6 | Uranus longitude | Retrograde-capable |
| 7 | Neptune longitude | Retrograde-capable |
| 10 | Sun longitude | Monotonic |
| 11 | Moon longitude | Monotonic dedicated dispatch |
| 16 | Moon phase angle | Monotonic dedicated dispatch |

Planetary and solar positions currently use the `vsop87` crate. Lunar longitude uses `astro::lunar::geocent_ecl_pos`. Moon phase is `(Moon longitude - Sun longitude) mod 360`.

## Completed backend work

- Replaced the experimental `search2` interface with one validated public `search` function.
- Removed the superseded daily-sampling search implementation.
- Corrected angular and sign-boundary crossing at 0°/360°.
- Added per-feature conservative coarse steps.
- Added correct start/end handling and deterministic merging of adjacent worker-compatible windows.
- Added instantaneous-velocity station detection and bisection refinement for retrograde planets.
- Added input validation and JavaScript-facing errors.
- Added Sun-first internal ordering and a monotonic fast path.
- Reused endpoint velocities and known endpoint angles to reduce repeated ephemeris calls.
- Added Moon longitude and Moon phase constraints.
- Added targeted boundary, station, retrograde, JPL longitude, lunar-phase, randomized, brute-force comparison, and malformed-input tests.
- Added ignored release profiling tests and randomized-verifier timing/resolution reporting.

## Important correctness decisions

- Angular constraints are generic; the Rust backend does not model zodiac signs. A zodiac sign is represented by a 30° angular interval by the caller.
- Angular intervals are half-open: `[start, start + span)` modulo 360°.
- Returned date windows are also half-open: `[start, end)`.
- Output is sorted and nonoverlapping.
- Worker partitions must be independently searchable and deterministically combinable. A future frontend merger must sort all partition results and merge adjacent or overlapping boundary windows.
- Search boundaries are refined to approximately one minute.
- Astronomical time-scale semantics and the formal supported date range still need to be finalized.

## Performance investigation

Ephemeris evaluation is the dominant cost. The earlier native experiments were kept outside the repository. The browser laboratory is retained under `benchmarks/ephemeris` and is not production code.

### Current VSOP87 versus Astronomy Engine

A native comparison of 100 warmed longitude calculations found the current implementation taking roughly 10–53 ms by body, while Astronomy Engine C took roughly 0.2–0.6 ms. Astronomy Engine was approximately 47–132 times faster for the planets in that experiment.

This is not evidence that C is inherently faster than Rust. Astronomy Engine intentionally uses a compact, truncated VSOP87 model with an approximately one-arcminute accuracy target, while the current Rust dependency evaluates much larger series. The algorithms, coordinate corrections, and accuracy contracts must be aligned before treating the ratio as conclusive.

The official Astronomy Engine JavaScript package was also tested in Node. It was fast enough to remain a serious browser candidate, but those Node timings are not comparable to the current browser WASM timings.

### Current VSOP87 versus `astro`

The existing `astro` crate can calculate all supported planets, not only the Moon. An optimized native experiment over 1,000 dates distributed across 1900–2100 found it approximately 2.0–2.6 times faster than the current `vsop87` path. Maximum disagreement with the current result was about 0.000006°, or approximately 0.02 arcseconds, in that sample.

This is the lowest-complexity alternative because `astro` is already a dependency and compiles with the current Rust/WASM workflow. Its principal risk is maintenance: version 2.0.0 was released in 2016.

### Other candidates

- Astronomy Engine C compiled with Emscripten: mature, compact, fast, and approximately one-arcminute accuracy; requires a different or combined WASM build workflow.
- Astronomy Engine JavaScript: same compact algorithm family with almost no integration work; valuable as a control against the C/WASM version.
- Swiss Ephemeris WASM: established and accurate, but requires an AGPL-compatible application or a purchased professional license.
- XALEN: feature-rich, pure Rust, and WASM-compatible, but too new to treat as mature without independent validation.

## Browser benchmark completed

The isolated laboratory is now in `benchmarks/ephemeris`. Read its `README.md` for reproduction and `RESULTS.md` for the measurements and current decision. Raw browser trials, accuracy checks, search windows, and compressed asset sizes are recorded in its `results/` directory.

Astronomy Engine C/WASM was the fastest overall benchmark candidate: approximately 56 times baseline mixed throughput and 1.5 times Astronomy Engine JavaScript in the calibrated run. The representative 200-year Sun/Moon query took 197.6 ms with production, 35.8 ms with JavaScript, and 12.6 ms with C/WASM. All returned 244 windows; both finalists agreed exactly with each other and differed from baseline by at most 0.88 minutes. These are measurements on one browser, not general latency guarantees.

Maximum sampled Astronomy Engine disagreement was 0.00565406 degrees over 1,001 dates spanning 1900–2100. Existing Sun, Moon, and phase JPL fixtures passed. Planetary truth fixtures near stations remain incomplete. The one-arcminute model target does not establish one-minute date accuracy near a station.

Production search code and frontend code remain unchanged by the benchmark work. On 2026-09-15, Matt decided to keep development, correctness testing, and the first working product in Rust because it is easier to understand and iterate. A C/WASM worker or translation to C is deferred as a future optimization, not the next task.

Rust 1.94.0 and wasm-pack 0.14.0 were installed on this laptop. A default macOS SDK/linker mismatch was resolved by setting `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk` for builds. Emscripten 4.0.15 and Astronomy Engine source used for this run are temporary checkouts at `/tmp/natal-emsdk` and `/tmp/natal-astronomy`; reproduce them from the lab documentation if absent. Node 24 is available through the bundled runtime or activated Emscripten SDK.

## Exact next task

Continue correctness work on the existing Rust backend. Define the longitude and returned-window accuracy targets, TT/UTC handling, and supported date range, then add representative planetary JPL fixtures near angular boundaries and retrograde stations. Verify complete date windows as well as individual longitudes; numerical one-minute refinement is not a guarantee of one-minute astronomical accuracy.

Keep the current Rust implementation and ephemeris dependencies while completing testing and building a working product. Do not start C integration or translation now. Revisit the retained benchmark only as a future optimization after the Rust product works. Present future implementations and proposed short commit messages for approval before committing unless explicitly authorized in advance.

## Original benchmark requirements

Build an isolated browser benchmark laboratory before replacing the production ephemeris engine. Do not integrate a candidate into the search engine first.

The benchmark should use separate Web Workers for:

1. Current `vsop87` Rust/WASM baseline.
2. `astro` Rust/WASM for all bodies.
3. Astronomy Engine JavaScript.
4. Astronomy Engine C/WASM compiled with Emscripten.
5. Optionally Swiss Ephemeris WASM and XALEN Rust/WASM.

Each worker should run a large deterministic loop internally and return only elapsed time plus a checksum. Do not cross the JavaScript/WASM boundary once per longitude calculation. Measure cold initialization separately from warmed throughput. Use identical dates and bodies, warm each implementation, randomize trial order, and report median and tail timings over repeated trials.

Record:

- Per-body longitude evaluations per second.
- Mixed-body throughput representative of real searches.
- Worker and module initialization time.
- Raw, gzip, and Brotli asset sizes.
- Maximum and RMS positional disagreement over 1900–2100.
- Error against the existing authoritative JPL fixtures.
- One complete representative search for only the finalists.

The result should determine whether to keep `vsop87`, switch all positions to `astro`, use Astronomy Engine JavaScript, or justify the added Astronomy Engine C/Emscripten integration.

## Remaining high-priority backend work

- Retain the Rust backend and document its ephemeris accuracy contract.
- Define UTC/UT/TT/JDE handling and the supported date range.
- Finish representative per-body JPL verification near angular boundaries and stations.
- Add Pluto and decide whether nodes, Chiron, Lilith, or other chart points are release requirements.
- Automatically order constraints by measured cost and selectivity.
- Establish latency and memory targets using actual browser measurements.
- Decide on worker count and implement deterministic multi-worker result merging if needed.
- Add cancellation, request identifiers, stale-response protection, progress reporting, and worker integration tests.
- Add CI and clean-machine build verification.

The full remaining product scope is in `docs/TODO.md`. Frontend production work should remain secondary until the supported feature set, accuracy, and performance are stable.

## Development workflow

Expected tools at handoff:

- Rust/Cargo 1.94.0
- `wasm32-unknown-unknown` Rust target
- `wasm-pack` 0.14.0
- Node 24.14.0 was used through nvm because the system Node installation was too old for the selected Vite version.

Install and verify:

```sh
npm ci
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Build the application:

```sh
npm run build
```

Run locally:

```sh
npm run dev -- --host 127.0.0.1
```

Run Rust correctness tests:

```sh
cargo test --manifest-path crate/Cargo.toml --release
```

Run the ignored profiler explicitly:

```sh
cargo test --manifest-path crate/Cargo.toml --release profile_search_execution -- --ignored --nocapture --test-threads=1
```

The randomized correctness test is intentionally expensive and may take several minutes.

## Verification at handoff

The following checks passed on 2026-09-13 before this document was committed:

- `cargo test --manifest-path crate/Cargo.toml --release`: 24 passed, 0 failed, 1 ignored manual profiler.
- `npm run build` with Node 24.14.0: Rust/WASM compilation, TypeScript compilation, and the Vite production build all completed successfully.
- Production WASM output: approximately 1,971.30 kB raw and 1,381.95 kB gzip.
- The only WASM build notices were optional missing Cargo package metadata and availability of a newer `wasm-pack` release.

## Working preferences established during development

- Keep responses and changes professional and direct.
- Backend correctness and performance come before frontend work.
- Finish correctness testing and a working product in Rust first; defer a C/WASM worker or C translation as a future optimization.
- Prefer small helper functions and minimal architectural splits.
- Keep function arguments on one line, even when the line is long.
- Keep the search implementation in `crate/src/lib.rs` and its tests in `crate/src/tests.rs` unless there is a concrete reason to restructure further.
- Use generic angular terminology in the backend; use “sign” rather than “zodiac” when discussing astrology-facing sign boundaries.
- Keep commit messages extremely short, simple, and without dashes.
- For future features, present the proposed commit message with the implementation for approval before committing unless Matt explicitly requests the commit in advance.
- Do not commit statistically insignificant optimization changes without presenting the measurements and deciding whether to discard them.

## New-laptop continuation

1. Share the original Codex task and copy its immutable link.
2. Clone this repository on the new laptop.
3. Install the documented toolchain and run the build and test commands above.
4. Open the cloned repository as a local Codex project.
5. Start a new task using the prompt below and include the shared-task link.

```text
Read docs/HANDOFF.md, docs/natal_chart_solver_design_doc.md, and docs/TODO.md completely. Inspect the current source and recent Git history. Continue from the exact next task in HANDOFF.md. The previous task is available at: <shared task URL>. Keep the backend in Rust and continue correctness work toward a working product. C/WASM is deferred as a future optimization. Follow the working preferences recorded in HANDOFF.md.
```
