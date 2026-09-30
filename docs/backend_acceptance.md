# Backend acceptance record

Date: 2026-09-30. Scope: the five backend preparation tasks in TODO.md. Implementation under test: `66480d9`, including apparent positions (`0ad2c60`), position regressions (`3489f85`), and complete-window regressions (`a8ac50b`).

## Checks

- Full native Rust release suite: 30 passed, 0 failed, 3 intentionally ignored manual diagnostics/profiling tests; 82.95 seconds. Binary and doc-test targets also passed.
- TypeScript time suite: 5 passed, 0 failed.
- Position fixture verification: 9 response hashes and all 108 positions verified offline.
- Complete-window fixture verification: 98 response hashes and all 12 searches regenerated exactly offline, including direct one-second reference boundary brackets.
- Production WASM build: passed.
- TypeScript compilation: passed.
- Vite production build: passed, 39 modules transformed.

Environment: Rust 1.94.0, wasm-pack 0.14.0, Node 24.19.0. Builds used `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk`. wasm-pack's prebuilt-tool lookup fell back successfully; optional package metadata and an available wasm-pack update produced nonblocking notices.

Commands used (tool executables were resolved from their installed paths):

```sh
cargo test --manifest-path crate/Cargo.toml --release
node --experimental-strip-types --test tests/time.test.ts
python3 scripts/fetch_jpl.py --verify
python3 scripts/fetch_jpl_windows.py --verify
wasm-pack build crate --target web --out-dir ../wasm/pkg --release
node node_modules/typescript/bin/tsc --noEmit
node node_modules/vite/bin/vite.js build
```

The three production-build stages are the stages in `npm run build`. No production source changes were required by this acceptance pass. Generated assets remain untracked build outputs. The WASM asset is 2,077.71 kB raw and 1,410.00 kB gzip; the JPL fixture files are test-only and are not application assets. No new browser latency benchmark was performed.

## Implemented backend

- Browser-local Rust/WASM computation, called through the existing Web Worker; no calculation server or runtime ephemeris download.
- Validated generic angular-constraint search, with multiple intersected constraints, wrapped arcs, retrograde handling, clipped endpoints, and sorted merged windows.
- Half-open angular arcs and time windows; full internal precision and preservation of positive subminute windows.
- Fixed TT domain from 1900-01-01 inclusive to 2100-01-01 exclusive, with exported bounds and rejection rather than silent clipping.
- Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and derived Moon phase.
- Apparent geocentric tropical longitudes using VSOP87E and the existing astro lunar model, consistent input frames, light time, solar deflection, aberration, precession, and analytical nutation.
- One-second root brackets, minute display, strict civil-date parsing, UTC/TT conversion, and historical/provisional time labels.
- Finite regression coverage for malformed input, angular/root edge cases, retrograde stations, randomized direct-evaluation comparisons, independent positions, and independent complete windows.

## Known limits

The fixed window suite found all 15 expected windows without missing or extra results. Its largest non-Neptune endpoint difference is 9.422 seconds; the Neptune ingress example is 919.622 seconds early. These internal TT measurements exclude civil conversion and display rounding. They are observations for selected cases, not global error bounds. The one-minute astronomical accuracy goal remains unproven, and the Neptune case exceeds it.

VSOP87 model error, analytical versus observed Earth-orientation differences, pre-1972 UT estimation, and future leap-second uncertainty remain documented in [backend_contract.md](backend_contract.md). Public-release acceptance of these limitations is still pending. No fixed model correction or replacement engine was introduced.

## UI transition

No additional backend research prerequisite is identified. The UI milestone must implement editable placements and date bounds, typed worker messages, initialization/search/error/empty states, cancellation and stale-response protection, and readable results. The TypeScript enum still contains unsupported entries, including Earth and Pluto; the UI must expose an explicit supported-body list rather than every enum member. Full-domain civil-date controls must respect the backend TT endpoints after conversion.

Browser lifecycle checks and representative responsiveness checks belong to that UI milestone. A successful build does not substitute for them. The current hardcoded search and console-only worker errors are prototype behavior, not a finished interface.

## Deferred

Ephemeris replacement, C/WASM migration, exhaustive ingress/station surveys, a universal minute-accuracy certification, additional bodies/chart points, multiple workers, streaming, filter-order optimization, alternate root solvers, and caches are deferred. CI, clean-machine verification, deployment checks, licensing, applicable dependency-security fixes, public accuracy wording, and cross-browser/accessibility checks remain before public release, not prerequisites for beginning the UI. TODO.md remains the authoritative ordered plan.
