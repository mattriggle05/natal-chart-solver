# Natal Chart Solver — First Version Plan

Updated 2026-09-29. This plan defines a bounded path to a usable first version. Deferred work is not a prerequisite for starting the UI or completing this milestone. The original broad completion inventory remains in Git history.

## Scope and decisions

- Keep the backend in Rust with the existing VSOP87 and astro dependencies. Ephemeris replacement and C/WASM migration are deferred.
- Search apparent geocentric tropical longitudes, true equinox of date. The apparent-position implementation has been reviewed and tested; its implementation is complete.
- Search within `[1900-01-01 00:00 TT, 2100-01-01 00:00 TT)`. Users select a subrange; the exported backend bounds are authoritative.
- Build the initial form around signs for the currently supported bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune. Moon phase remains supported by the backend; a phase input is optional later.
- Display times to the nearest minute while retaining full internal precision and subminute windows.
- Minute display resolution is implemented. A universal one-minute astronomical accuracy guarantee is not established. The known Neptune discrepancy remains a limitation, not an accepted maximum error or permission to weaken tests silently.
- Keep search implementation in `crate/src/lib.rs` and tests in `crate/src/tests.rs`. Do not restructure them merely to satisfy a checklist.
- Do not update HANDOFF.md as part of routine work.

## Completed foundation

These items describe implemented behavior and existing tests, not exhaustive proofs of accuracy.

- [x] Validated public search API; reject malformed arrays, unsupported feature IDs, invalid angles, and nonfinite or reversed dates.
- [x] Half-open angular constraints and returned time windows; preserve and clip matching range endpoints.
- [x] Wraparound, prograde and retrograde boundary refinement, station splitting, and deterministic window merging.
- [x] Sun-first filtering, per-feature coarse steps, and endpoint reuse.
- [x] Moon longitude and derived Moon phase search.
- [x] Range validation and exported JD(TT) bounds (`5953f84`).
- [x] Explicit TT conversion through UI and worker, strict calendar parsing, historical/provisional civil-time labels (`2812a3f`).
- [x] One-second numerical refinement, minute display, and preservation of subminute windows (`614fd1f`).
- [x] TypeScript tests for conversion, leap seconds, calendar validation, and minute formatting.
- [x] Existing Rust boundary, validation, retrograde, randomized, and direct-evaluation regression tests.
- [x] Isolated browser ephemeris benchmarks retained for future decisions.

## Before UI work — execute in this order

1. [x] **Finish the apparent-position change.** Review light time, aberration, deflection, precession, nutation, and frame handling. Resolve the remaining implementation/frame discrepancy with focused checks. Keep the known VSOP87 model disagreement separate from implementation bugs. Remove experimental production code or finalize it; retain useful diagnostics separately from routine acceptance tests. Completed 2026-09-29: common FK5 input frame, EOP residual explained with a focused reference test, manual surveys marked ignored.
2. [ ] **Make the collected position fixtures into regression tests.** Reuse existing JPL samples across the supported range for every supported body and derived Moon phase. Add explicit, justified angular tolerances and provenance; print-only probes are not accuracy tests. Preserve the Neptune discrepancy as a documented regression case. Do not start an exhaustive century-wide ingress survey.
3. [ ] **Finish a fixed suite of 8–12 complete reference searches.** Inventory and reuse existing coverage before adding cases. Cover ordinary single-body results, multiple constraints, retrograde re-entry, wrapped and narrow arcs, clipped endpoints, and empty results. Include independently sourced complete windows and representative boundary/station cases. Check missing/extra windows separately from endpoint timing differences. Do not count agreement with the same ephemeris as independent astronomical validation.
4. [ ] **Document the measured accuracy and remaining limits.** Update the backend contract and affected design sections with the actual coordinate convention, tested scope, observed timing differences, and distinction between numerical precision and model accuracy. Do not promise one-minute astronomical accuracy or generalize one observed error into a global bound. No further engine-selection research in this milestone.
5. [ ] **Run the final backend acceptance pass and commit the finished changes.** Run the full existing Rust suite, TypeScript time tests, and production WASM/TypeScript/Vite build. Fix failures attributable to the changes. Commit apparent positions, reference tests, and other distinct concerns separately with short messages. Do not include unrelated working-tree changes.

**Stop condition:** Once these five items pass with no unexplained implementation failures, begin the UI. Known, documented model limitations do not trigger an open-ended ephemeris replacement project. If a concrete failure prevents completion, report that specific failure and its smallest proposed resolution instead of expanding the plan. Add further tests only for changed behavior, a reproduced bug, or an explicitly expanded requirement. This is readiness for a usable prototype, not astronomical certification.

## Rudimentary UI — next milestone

- [ ] Replace the hardcoded query with add/remove body-and-sign rows; prevent duplicate and unsupported bodies.
- [ ] Add editable start/end dates, sensible defaults, and validation against the backend's TT bounds, including civil-time conversion at the domain edges.
- [ ] Add accessible labels and field errors; prevent invalid or duplicate submissions.
- [ ] Add typed worker request/result/error messages and explicit initialization, searching, success, empty, and error states.
- [ ] Add cancellation by worker termination/replacement and request identifiers to reject stale responses.
- [ ] Display an ordered result list, count, submitted criteria, minute-level civil times, and subminute-window labels using the existing utilities.
- [ ] Explain the supported date range, approximate astronomical boundaries, and unavailable bodies without exposing implementation details in the normal input flow.
- [ ] Make the form and results usable on desktop and mobile with keyboard navigation and readable contrast. Do not require solar-system visualization redesign.
- [ ] Verify browser success, invalid input, initialization/search failure, empty results, cancellation, and restart without stale output.
- [ ] Check responsiveness on representative short and full-domain searches. Optimize only if an observed usability problem requires it.

## Before first public release — not prerequisites for UI development

- [ ] Review the measured model limitations with Matt and settle the public accuracy wording and whether those limitations are acceptable for release.
- [ ] Verify a known chart search against an independent source; account explicitly for any boundary disagreement.
- [ ] Document supported features, setup, test/build commands, and known limitations in README; pin the supported toolchain.
- [ ] Add CI for the existing correctness tests and production build; verify a clean installation/build.
- [ ] Review dependency security findings and resolve applicable serious issues.
- [ ] Choose the project license and complete essential package metadata.
- [ ] Correct deployment URL/base-path issues and missing asset references; verify the actual deployed search in current Chrome, Firefox, Safari, and Edge.
- [ ] Complete basic responsive/accessibility checks and confirm that normal searches produce no unexpected errors.

## Deferred backlog — requires a concrete need or explicit scope expansion

### Accuracy and supported features

- [ ] Replace VSOP87 with a newer analytical model or locally evaluated ephemeris data, with isolated size/speed/accuracy measurements first.
- [ ] Survey all sign crossings and near-station errors across 1900–2100; investigate stronger astronomical accuracy guarantees.
- [ ] Add Pluto.
- [ ] Decide on nodes, Chiron, Lilith, and other chart points; implement only selected additions.
- [ ] Add Moon phase controls, exact-degree inputs, or an ecliptic dial.
- [ ] Add birth-location, houses, ascendant, or other location-dependent refinement.
- [ ] Expand the supported date range; only then consider pre-Gregorian handling or 1,000-year benchmarks.

### Performance and architecture

- [ ] Revisit C/WASM or translation to C after the Rust product works.
- [ ] Order remaining constraints by measured cost/selectivity.
- [ ] Add determinate progress or streamed results.
- [ ] Add multi-worker partitioning, conservative worker selection, and tested partition merging if measured latency warrants it.
- [ ] Evaluate Brent's method if root refinement is a measured bottleneck.
- [ ] Evaluate an ephemeris cache if position evaluation is a measured bottleneck.
- [ ] Generate synchronized Rust/TypeScript feature definitions if manual maintenance becomes problematic.
- [ ] Reconsider module boundaries only for a concrete maintainability problem.

### Product and presentation

- [ ] Add presets, row reordering, saved preferences, and shareable query URLs.
- [ ] Add result export, detailed placement inspection, and result-driven visualization.
- [ ] Add pagination or virtualization if measured result volume warrants it.
- [ ] Improve solar-system labels, coordinate context, layout, animation accessibility, and initialization behavior if retaining it as a product feature.
- [ ] Add locale-aware formatting and additional timezone presentation.
- [ ] Add polished branding, social previews, and application icons.
- [ ] Add offline/service-worker caching, operational monitoring, and automated dependency updates if needed.
