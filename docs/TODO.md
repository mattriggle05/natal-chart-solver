# Natal Chart Solver — Completion Checklist

This checklist defines the work required for Natal Chart Solver to be considered complete. Items are divided into computation/backend work and frontend/product work. A release is complete when every applicable checkbox is checked and the final acceptance criteria pass.

## Backend / Computation

### Search correctness

- [ ] Define and document exact search interval semantics, including whether start and end boundaries are inclusive or exclusive.
- [x] Validate that `start_julian_date` is finite and earlier than `end_julian_date`.
- [x] Validate that `feature_ids` and `feature_signs` have equal, nonzero lengths.
- [x] Validate every feature ID and zodiac-sign ID before starting a search.
- [x] Return structured errors to TypeScript instead of `NaN`, panics, or silently empty results.
- [x] Correctly preserve a matching window that begins at the start of the requested search range.
- [x] Correctly append a matching window that remains open at the end of the requested search range.
- [ ] Handle exact sign-boundary timestamps consistently.
- [x] Normalize angular differences across the 0°/360° boundary.
- [x] Correctly detect Pisces-to-Aries and Aries-to-Pisces transitions in both prograde and retrograde motion.
- [x] Replace average-displacement station detection with instantaneous angular velocity.
- [x] Ensure station bracketing works when a station lies exactly on a coarse-sample timestamp.
- [x] Split search ranges into provably monotonic intervals around retrograde stations.
- [x] Make longitude boundary refinement work for both increasing and decreasing longitude.
- [ ] Verify that multiple sign entries during one retrograde cycle produce separate, ordered windows.
- [ ] Merge adjacent or numerically overlapping result windows where appropriate.
- [x] Guarantee sorted, nonoverlapping results for every valid search.
- [ ] Decide how UTC, UT, TT, and JDE differences are handled and document the supported time accuracy.
- [ ] Define the supported historical and future date range based on VSOP87 accuracy.

### Astronomical coverage and accuracy

- [ ] Establish an accuracy target for longitudes, sign boundaries, and returned date windows.
- [ ] Validate geocentric longitude calculations against JPL Horizons or another authoritative ephemeris across representative dates.
- [ ] Verify all supported planets near sign boundaries and retrograde stations.
- [ ] Investigate and resolve or formally accept the `vsop87d` solar-system-display offset.
- [ ] Implement Moon longitude with a documented astronomical model and accuracy range.
- [ ] Implement Pluto longitude with a documented astronomical model and accuracy range.
- [ ] Decide whether Chiron, the North Node, Lilith, and other chart points are release requirements.
- [ ] Implement every additional release-required chart point and document whether it is true, mean, or otherwise derived.
- [ ] Keep the Rust feature-ID definitions and TypeScript `Feature` enum synchronized from one authoritative source.
- [ ] Remove unsupported feature identifiers from the production UI until their computation is implemented.

### Search performance and execution

- [x] Replace the fixed one-day coarse step with verified per-body safe step sizes.
- [ ] Automatically order filters by expected selectivity and computation cost.
- [ ] Use the Sun as the first filter when it is included and verify that reordering cannot change results.
- [x] Remove per-step Rust console logging from production builds.
- [ ] Benchmark single-body and multi-body searches over representative 1-, 10-, 100-, and 1,000-year ranges.
- [ ] Define acceptable search latency and memory limits for supported devices.
- [ ] Add progressive result delivery or progress reporting for long searches.
- [ ] Implement search cancellation by terminating and replacing the active worker.
- [ ] Ignore stale worker responses using search/request identifiers.
- [ ] Decide whether multi-worker range partitioning is required to meet the performance target.
- [ ] If required, implement multi-worker search partitioning, result ordering, deduplication, and boundary merging.
- [ ] Select worker count conservatively using device capabilities and measured performance.
- [ ] Evaluate Brent's method or another root finder after correctness is established.
- [ ] Evaluate a precomputed ephemeris cache only if profiling shows VSOP87 evaluation remains a bottleneck.

### Rust/WASM API and maintainability

- [x] Replace `search2` with a stable, clearly named public search API.
- [x] Rename the superseded daily-sampling search and remove it from the public WASM API.
- [ ] Break `crate/src/lib.rs` into focused modules for coordinates, bodies, search, roots, dates, and WASM bindings.
- [ ] Add typed request, result, progress, cancellation, and error contracts across the worker boundary.
- [ ] Ensure WASM initialization failures are surfaced to the UI.
- [ ] Add Rust documentation for public functions and non-obvious numerical assumptions.
- [ ] Add package description, repository, and license metadata to `Cargo.toml`.
- [ ] Decide whether panic-hook and `js-sys` dependencies are needed, then align `Cargo.toml` with the design documentation.
- [ ] Remove the unused `astronomy-engine` dependency unless it becomes part of verification or production behavior.
- [ ] Pin and document supported Node, npm, Rust, wasm-pack, and wasm target versions.
- [ ] Add a version-manager file such as `.nvmrc` so Vite always runs on a supported Node version.

### Automated verification

- [ ] Add Rust unit tests for angle normalization and sign mapping.
- [ ] Add Rust unit tests for prograde and retrograde root refinement.
- [ ] Add tests for stations, exact boundaries, range endpoints, and 0°/360° wraparound.
- [x] Add tests for malformed arrays, invalid IDs, invalid signs, reversed ranges, and non-finite values.
- [ ] Add golden-data tests using authoritative ephemeris values.
- [x] Add property tests asserting sorted, nonoverlapping windows whose sampled interiors satisfy all requested signs.
- [x] Compare optimized search results against a small-step brute-force reference implementation.
- [ ] Add TypeScript tests for Julian-date conversion and result formatting.
- [ ] Add worker integration tests covering success, error, cancellation, and stale responses.
- [ ] Add end-to-end tests for representative user searches.
- [ ] Run the full test suite and production build in continuous integration.

### Security, build, and release infrastructure

- [ ] Review and remediate npm audit findings without breaking the locked build.
- [ ] Add automated dependency-update and security scanning.
- [ ] Ensure production source maps and development-server access follow the intended security posture.
- [ ] Verify reproducible installation with `npm ci` on a clean machine.
- [ ] Verify reproducible Rust/WASM compilation on a clean machine.
- [ ] Add CI checks for formatting, TypeScript, Rust, tests, and the production build.
- [ ] Document the complete local development, test, build, preview, and deployment workflow.
- [ ] Verify that generated WASM artifacts are always produced during build and do not need to be committed.
- [ ] Decide whether a service worker and offline asset caching are release requirements.
- [ ] If required, implement versioned caching for the app shell and WASM binary.

## Frontend / Product

### Product definition and information architecture

- [ ] Finalize the primary use case: reverse-searching dates from complete or partial natal-chart placements.
- [ ] Define the minimum supported set of celestial bodies for the first complete release.
- [ ] Decide whether users enter signs only, exact degrees, or both.
- [ ] Decide whether birth time and birth location refinement are part of the release.
- [ ] Define how approximate, ambiguous, or multiple date results are explained.
- [ ] Replace the “Coming soon...” placeholder with a clear product name, purpose, and concise instructions.
- [ ] Explain the difference between astronomical computation and astrological interpretation.
- [ ] Publish supported date ranges, accuracy limitations, and unsupported chart features.

### Search form

- [ ] Replace the hardcoded `SearchBox` query with editable search controls.
- [ ] Add start- and end-date inputs with sensible defaults and validation.
- [ ] Add rows for selecting a celestial body and its zodiac sign.
- [ ] Allow users to add, remove, and reorder placement rows.
- [ ] Prevent duplicate celestial-body selections unless there is a defined reason to allow them.
- [ ] Prevent unsupported bodies from being selected.
- [ ] Add accessible labels, descriptions, validation messages, and keyboard interaction.
- [ ] Disable submission until the query is valid.
- [ ] Provide useful presets or examples for first-time users.
- [ ] Decide whether an ecliptic dial is valuable; implement it only if it materially improves exact-position input.
- [ ] Keep search state in the URL so searches can be bookmarked and shared.
- [ ] Preserve appropriate search preferences locally between sessions.

### Search lifecycle and feedback

- [ ] Show an initialization state while the WASM module loads.
- [ ] Show an explicit searching state and elapsed time.
- [ ] Display determinate progress when the computation layer can provide it.
- [ ] Add a cancel-search control.
- [ ] Cancel or supersede an active search when parameters change and the user starts again.
- [ ] Show actionable errors for invalid input, worker failure, WASM failure, and unsupported searches.
- [ ] Provide a clear empty-results state.
- [ ] Prevent duplicate submissions and stale results.
- [ ] Keep the interface responsive throughout the largest supported search.

### Results experience

- [ ] Replace comma-separated result text with a structured result list or table.
- [ ] Display result windows using unambiguous calendar formatting and timezone conventions.
- [ ] Preserve fractional Julian dates when time-level precision is relevant.
- [ ] Show result count and search criteria with the results.
- [ ] Sort results chronologically and clearly distinguish exact timestamps from ranges.
- [ ] Allow users to inspect the calculated placements at a result date.
- [ ] Allow a result to update the solar-system visualization.
- [ ] Add pagination, virtualization, or incremental rendering for large result sets.
- [ ] Add copy, download, or export functionality for results if required by target users.
- [ ] Provide a shareable link that reconstructs the query and selected result.

### Solar-system visualization

- [ ] Decide whether the heliocentric solar-system model is part of the core workflow or a supporting visualization.
- [ ] Label every planet and orbit clearly.
- [ ] Add zodiac/ecliptic context if the visualization is meant to explain search results.
- [ ] Distinguish the heliocentric display from the geocentric zodiac calculations.
- [ ] Handle invalid dates and WASM calculation errors.
- [ ] Avoid reinitializing WASM unnecessarily on every date update.
- [ ] Verify transitions and layout across supported browsers.
- [ ] Make the visualization usable on small screens without clipping or forced page-level overflow.
- [ ] Add reduced-motion behavior for users who request it.

### Design, responsiveness, and accessibility

- [ ] Create a cohesive visual design for the search, progress, results, and visualization views.
- [ ] Replace viewport-dependent fixed sizing where it causes overflow or unusable layouts.
- [ ] Restore normal page scrolling where content exceeds the viewport.
- [ ] Support mobile, tablet, laptop, and wide desktop layouts.
- [ ] Meet WCAG 2.2 AA color-contrast requirements.
- [ ] Ensure complete keyboard navigation and visible focus states.
- [ ] Use semantic headings, landmarks, forms, buttons, tables, and status regions.
- [ ] Announce progress, completion, errors, and result counts to assistive technologies.
- [ ] Verify zoom to 200% and text reflow without loss of functionality.
- [ ] Test with reduced motion, high contrast, and common screen readers.

### Dates, formatting, and localization

- [ ] Move date conversion and formatting into tested shared utilities.
- [ ] Replace the current date-only Julian conversion when time precision is required.
- [ ] Define behavior for dates before the Gregorian calendar transition.
- [ ] Use an unambiguous default date format and consider locale-aware presentation.
- [ ] Clarify whether displayed times are UTC, local time, TT, or another astronomical time scale.
- [ ] Handle browser parsing consistently instead of relying on ambiguous date strings.

### Application metadata and deployment

- [ ] Correct and standardize the GitHub Pages URL and username spelling across README, package metadata, and documentation.
- [ ] Replace or remove missing `logo192.png` and `logo512.png` manifest references.
- [ ] Create final favicon, application icons, title, description, and social preview metadata.
- [ ] Verify the Vite base path and all asset URLs on GitHub Pages.
- [ ] Add a user-facing not-found or routing fallback if client-side routes are introduced.
- [ ] Verify production behavior using `npm run preview` before every release.
- [ ] Deploy to GitHub Pages through a repeatable CI workflow rather than a workstation-only manual step.
- [ ] Add basic privacy-respecting error and performance monitoring if operational requirements justify it.
- [ ] Verify the production application in current Chrome, Firefox, Safari, and Edge.

### Documentation and final acceptance

- [ ] Rewrite the README with product status, screenshots, supported features, setup, testing, architecture, and deployment instructions.
- [ ] Update the design document so filenames, dependencies, implemented behavior, and performance claims match reality.
- [ ] Separate implemented features from proposed future work in all documentation.
- [ ] Document astronomical data sources, algorithms, accuracy, and limitations.
- [ ] Document known limitations that remain intentionally accepted for release.
- [ ] Add contributor guidance and coding/testing conventions.
- [ ] Add an appropriate license.
- [ ] Complete a clean-machine installation and build using only repository documentation.
- [ ] Complete an end-to-end acceptance test for a known natal chart with independently verified expected date windows.
- [ ] Confirm all automated tests pass in CI.
- [ ] Confirm the production build has no unexpected console errors or warnings.
- [ ] Confirm there are no unresolved high-severity security findings applicable to production.
- [ ] Confirm the deployed application is responsive, accessible, and usable without developer assistance.
- [ ] Confirm every release-required body, input mode, search range, and result behavior is documented and tested.
- [ ] Mark the application complete only after every applicable item above is checked or explicitly moved to a documented post-release roadmap.
