# Local preview search timings

Measured October 6–7, 2026 against commit `25fdf4a`, using the local Vite preview form at `http://127.0.0.1:5173/natal-chart-solver/preview/` in the Codex in-app browser and the release WASM built for deployment. These are a small set of interactive observations, not a latency guarantee or a controlled cross-version benchmark. Power state, thermal conditions, and other host activity were not controlled.

Temporary local instrumentation measured `performance.now()` around the synchronous WASM search inside its worker and from the hook's search invocation through receipt and string formatting of results. Total time excludes React rendering/paint. Each request created a fresh worker; browser asset caches were not cleared. All measurement changes were removed afterward. No production search algorithm or deployed assets were changed.

## Results

All large queries use the full exported TT domain, 1900-01-01 inclusive to 2100-01-01 exclusive. Unlisted bodies are unconstrained. Counts below come from the raw result length, not the temporary timing line inserted into the output list.

| Constraints | Compute ms | Request through formatting ms | Intervals |
| --- | ---: | ---: | ---: |
| Sun Virgo, Moon Aries, first run | 11095.3 | 11121.0 | 244 |
| Sun Virgo, Moon Aries, repeat | 10988.1 | 11007.8 | 244 |
| Mercury Aries | 104119.3 | 104140.1 | 240 |
| Sun Aries, Mercury Aries | 16347.4 | 16365.4 | 221 |
| Sun Aries | 5750.1 | 5769.5 | 200 |
| Nine-body combination below | 13010.1 | 13029.0 | 0 |
| Sun Virgo, 2005-01-01 to 2006-01-01, after cancellation | 34.4 | 51.4 | 1 |

Nine-body query: Sun Aries, Moon Aries, Mercury Aries, Venus Taurus, Mars Cancer, Jupiter Taurus, Saturn Pisces, Uranus Taurus, Neptune Pisces. This was an arbitrary combination, not an independent known-chart acceptance case. An empty intersection allows later filters to do no work.

Mercury's completed observation was restarted after an interrupted session; the earlier incomplete run was excluded. A Sun-only attempt whose page reset before timing could be retrieved was also excluded. The recorded Sun-only row is the subsequent completed measurement in a dedicated tab.

## Comparison and diagnosis

The archived browser benchmark in `benchmarks/ephemeris/RESULTS.md` recorded a median 197.6 ms for Sun Virgo plus Moon Aries over a nearly identical 200-year domain (its endpoints are shifted by half a day). Today's roughly 11-second computation is about 56 times that historical figure. Both return 244 intervals. The old run used geometric positions, one-minute root refinement, and a different browser session. This ratio is not a controlled attribution of the slowdown to a single change.

Current overhead outside WASM was approximately 17–26 ms in these observations. The dominant measured cost is search computation, not worker loading or result formatting; paint cost was not measured.

Source inspection identifies likely causes to profile next:

- Apparent longitude now evaluates Earth at reception and at two nearby times for observer velocity, evaluates the target iteratively for light time, and evaluates the Sun for corrections. The older Sun path used a direct Earth longitude, and the older Moon path used only the lunar series.
- Mercury uses 3.5-day coarse steps. Retrograde detection adds two apparent-longitude evaluations for each finite-difference velocity, plus station/boundary refinement.
- One-second bisection adds approximately six iterations versus one-minute refinement for otherwise equal brackets. This does not by itself explain a 56-fold difference.
- Sun is filtered first. Adding Sun Aries reduced Mercury Aries from 104.1 to 16.35 seconds by restricting its candidate intervals. More constraints do not necessarily mean a slower search.

These source observations do not quantify individual correction costs. No correction, precision target, or model was changed during this test.

## Responsiveness and next action

Cancellation while Mercury was computing restored the form immediately in the browser interaction. A subsequent one-year Sun search returned the expected interval successfully. This checks functional responsiveness, not frame-rate or input-latency percentiles.

Recommendation: retain Rust and apparent positions; do one bounded profiling pass on the Sun/Moon and Mercury cases before changing engines. Measure repeated ephemeris evaluation and per-filter cost, then target the largest demonstrated redundant work. Re-run these same queries and the relevant existing correctness regressions after any optimization. C/WASM migration remains deferred.
