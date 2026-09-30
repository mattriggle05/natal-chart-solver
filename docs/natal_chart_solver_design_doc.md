# Natal Chart Solver — Design & Context Document

The current milestone order is defined in [TODO.md](TODO.md). Coordinate conventions, measured accuracy, and limitations are defined in [backend_contract.md](backend_contract.md); historical measurements below are not current performance guarantees.

## Project Overview

A browser-based astronomical alignment search engine that finds historical and future
date ranges where specified planetary bodies occupy specified zodiac signs simultaneously.
The intended use case is natal chart analysis — a user enters their birth chart planetary
positions and the app finds all dates when that configuration occurred or will occur.

**Repository:** `natal-chart-solver`
**Deployed to:** GitHub Pages via `gh-pages` npm package
**Homepage:** `https://mattriggl05.github.io/natal-chart-solver`

---

## Architecture

### Stack
- **Frontend:** React 19 + TypeScript, built with Vite 7
- **Computation core:** Rust compiled to WebAssembly via `wasm-pack`
- **Deployment:** Static files on GitHub Pages (no server-side compute)

### Key Architectural Decision: Client-Side WASM
All computation runs entirely in the browser. There is no backend server. This was a
deliberate decision to eliminate network latency from the computation path and avoid
server infrastructure costs. The "server" is a static CDN (GitHub Pages).

### Directory Structure
```
/
├── src/                          # React + TypeScript
│   ├── components/               # UI components (SearchBox, SolarSystem, etc.)
│   ├── hooks/                    # Custom React hooks (useDateSearch)
│   ├── workers/                  # Web Worker files
│   ├── types/                    # Shared TypeScript types
│   └── utils/                    # Utility functions (date conversion etc.)
├── crate/                        # Rust source
│   └── src/
│       └── lib.rs                # All Rust code currently in one file
├── wasm/                         # wasm-pack OUTPUT — never edit manually
│   └── pkg/                      # Generated JS glue + .wasm binary
├── public/                       # Static assets
├── vite.config.ts
└── package.json
```

### Web Worker Architecture
Computation runs in a Web Worker (separate OS thread) to keep the React UI responsive.
The worker owns the WASM module lifecycle. Communication uses `postMessage`.

```
React Component
  → calls search(params) from custom hook
      → hook postMessages to Worker
          → Worker calls Rust WASM find function
          → Rust returns the completed result; Worker sends it via postMessage
      → hook updates React state
  → component re-renders with results
```

**SharedArrayBuffer is intentionally NOT used.** GitHub Pages cannot serve the required
COOP/COEP headers to enable it. The performance loss is minimal for this use case —
the only thing SharedArrayBuffer would have enabled is a shared abort flag, which is
replaced by `worker.terminate()` + respawn on cancellation.

---

## Build System

### Scripts (package.json)
```json
"build:wasm": "wasm-pack build crate --target web --out-dir ../wasm/pkg --release",
"build": "npm run build:wasm && tsc && vite build",
"dev": "vite",
"predeploy": "npm run build",
"deploy": "gh-pages -d build"
```

Note: `--out-dir ../wasm/pkg` uses a relative path from the `crate/` directory.
On Windows this is `..\\wasm\\pkg` in JSON strings but forward slashes work cross-platform.

### Vite Configuration
```typescript
resolve: { alias: { '@wasm': path.resolve(__dirname, 'wasm/pkg') } }
worker: { format: 'es' }
optimizeDeps: { exclude: ['natal-solver'] }
```

The `@wasm` alias lets TypeScript import from the generated WASM package cleanly.
`optimizeDeps.exclude` prevents Vite from pre-bundling the WASM glue (it handles
its own initialization).

### Cargo.toml Profile
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

Full optimization for release builds. `wasm-pack build --dev` for fast iteration
during development (~3-8 second rebuilds vs 30+ for release).

---

## Astronomical Model

The production backend remains Rust with the existing VSOP87 and astro models. Complete correctness testing and a working product in Rust first; a C/WASM worker or C translation is deferred as a future optimization. The isolated measurements remain in `../benchmarks/ephemeris/RESULTS.md`. The apparent implementation, position regression suite, and fixed reference-window suite are implemented. The final acceptance pass passed on 2026-09-30; the next milestone is the rudimentary UI.

### Coordinate System and Apparent Positions

Search uses apparent geocentric tropical ecliptic longitude, referred to the analytical true equinox of date. Inputs and results use JD(TT). The implemented conventions and limitations are specified in [backend_contract.md](backend_contract.md).

VSOP87E provides barycentric rectangular J2000 positions. A fixed rotation puts those vectors into FK5 before combining them with the astro lunar model, whose mean-of-date offset is precessed into the same frame. Light-time iteration uses reception-time Earth and emission-time target positions. Solar deflection and observer-velocity aberration precede precession to date and analytical IAU1980 nutation.

The application does not load measured Earth-orientation corrections. Horizons applies those corrections in the modern era; this convention difference is separated from ephemeris error in a focused JPL/IERS regression test. The known Neptune model discrepancy remains. Numerical refinement and minute display do not imply one-minute astronomical accuracy.

Moon longitude retains `astro::lunar::geocent_ecl_pos`, with its principal ELP-2000/82 terms. Moon phase is `(apparent Moon longitude - apparent Sun longitude) mod 360`. Sun and Moon searches retain dedicated monotonic dispatch; planetary searches retain retrograde handling.

The heliocentric visualization still uses VSOP87D of-date longitudes. Its difference from J2000 positions is an expected reference-frame difference, not a known 0.36-degree crate error.

### Planet/Feature ID Scheme
```
0  = Mercury
1  = Venus
2  = Earth (no geocentric longitude — observer)
3  = Mars
4  = Jupiter
5  = Saturn
6  = Uranus
7  = Neptune
10 = Sun
11 = Moon
16 = Moon phase angle
```
IDs 8 and 9 are intentionally unused (reserved). Pluto is not currently implemented.

### Angular Constraints
The backend does not represent zodiac signs. It searches half-open angular arcs defined
by a start in `[0°, 360°)` and a span in `(0°, 360°]`. The frontend converts a zodiac
sign index to `start_degrees = sign_index * 30` and `span_degrees = 30`.

Using a start and span instead of start and end removes ambiguity for arcs that cross
0°/360°. Angle membership is `(angle - start_degrees).rem_euclid(360) < span_degrees`.

### Julian Date Conversion

All ephemeris and search values are JD(TT). The shared `src/utils/time.ts` converts explicit Gregorian civil dates and browser timestamps to TT, and converts results back with historical/provisional time labels. See [backend_contract.md](backend_contract.md) for the leap-second and pre-1972 policy. The roughly 69-second modern UTC/TT difference must not be ignored for minute-level results.

---

## Rust Functions

### `search` (validated public search function, wasm_bindgen exported)
```rust
pub fn search(start_julian_date: f64, end_julian_date: f64, feature_ids: &[u8],
              angle_starts: &[f64], angle_spans: &[f64]) -> Result<Vec<f64>, JsValue>
```

**Parameters:**
- `start_julian_date`, `end_julian_date`: Julian date range to search
- `feature_ids`: Angular feature IDs; the search currently evaluates the Sun first when present
- `angle_starts`: Corresponding arc start angles in `[0°, 360°)`
- `angle_spans`: Corresponding arc spans in `(0°, 360°]`

**Returns:** A flat-packed `Vec<f64>` of `[window_start_jd, window_end_jd, ...]`
pairs, or a JavaScript error for invalid input.

The public function validates every input and then delegates to
`search_refined_windows`. This leaves one stable browser contract while allowing the
internal search algorithm to be replaced later.

### `search_refined_windows` (active internal algorithm)

**Algorithm:**
1. Starts with one window: the full search range
2. For each angular constraint in internal evaluation order:
   - For each current candidate window:
     - Coarse sweep using a conservative per-body step
     - At each step: compute the feature angle
     - Detect retrograde stations via instantaneous velocity sign changes
     - On detected station: bisect to find precise station time
     - Split the step into monotonic segments around the station
     - Detect every arc boundary crossed, including both boundaries of a narrow arc within one coarse step
     - Bisect each crossed boundary and accumulate matching sub-windows
   - Replace window list with new sub-windows
3. Return final intersected windows

**Known issues / incomplete areas:**
- No streaming/callback — returns all results at once after full computation

**Current conservative coarse steps (per planet):**
```
Sun:     28 days
Moon:    10 days
Moon phase angle: 10 days
Mercury: 3.5 days
Venus:   12 days
Mars:    18 days
Jupiter: 60 days
Saturn:  67 days
Uranus:  75 days
Neptune: 78 days
```

---

### `angle_at`
```rust
pub fn angle_at(julian_date: f64, feature: Feature) -> f64
```
Returns the selected feature's angle in degrees `[0°, 360°)` at a given Julian date.
The current features are geocentric body longitudes and the derived Moon phase angle.
The `Feature` enum prevents an unsupported numeric ID from reaching astronomical
evaluation after input validation.

The apparent-position pipeline evaluates reception/emission positions and Earth velocity in one inertial frame. These corrections require additional ephemeris evaluations compared with the original geometric implementation.

---

### `longitude_from_observer`
```rust
pub fn longitude_from_observer(observer_coords: RectangularCoordinates,
                                feature_coords: RectangularCoordinates) -> f64
```
Converts heliocentric rectangular coordinates to geocentric ecliptic longitude via
vector subtraction and atan2. Named to clearly express "longitude of feature as seen
from observer." The z dimension is intentionally ignored — ecliptic latitude is not
needed for longitude-based angular constraints.

---

### `instantaneous_velocity`
```rust
pub fn instantaneous_velocity(julian_date: f64, feature: Feature) -> f64
```
Computes geocentric angular velocity in degrees/day using the centered finite difference:
```
v(t) = (longitude(t + H) - longitude(t - H)) / (2H)
```
The implemented `H` is `6e-6` days. Centering avoids the first-order truncation error of a one-sided difference, but this step is not a proved optimum or a universal derivative error bound. Each velocity evaluation calls the full apparent-position pipeline twice. Station and window regression tests check the behavior at selected cases.

---

### `bisection_derivative_find_zero`
```rust
pub fn bisection_derivative_find_zero(start_julian_date: f64,
                                       end_julian_date: f64,
                                       feature: Feature) -> f64
```
Finds the Julian date of a retrograde station (velocity zero) within a given interval
using bisection. Caller must guarantee opposite velocity signs at endpoints.

**Termination conditions:**
- Exact numerical zero velocity (including at a bracket endpoint)
- Interval width < 1 second (1/86400 days); return the bracket midpoint

**Bisection direction:** Uses `f64_same_sign` against reference velocity at `left` to
determine which half contains the zero. Correctly handles both prograde→retrograde and
retrograde→prograde stations.

---

### `bisection_value_find`
```rust
pub fn bisection_value_find(start_julian_date: f64, end_julian_date: f64,
                             target_value: f64, feature: Feature) -> f64
```
Finds the Julian date when a feature angle equals `target_value` within a monotonic
interval. Used to refine angular-constraint boundary crossings.

**Termination conditions:**
- Exact numerical equality with the target
- Interval width < 1 second; return the bracket midpoint

**Target values** are the constraint's start angle and its normalized end angle.

---

### `f64_same_sign`
```rust
pub fn f64_same_sign(a: f64, b: f64) -> bool
```
Bit-manipulation sign comparison. Returns `false` if either value is ±0.0 (zero has
no meaningful sign in our velocity context — landing exactly on a station). Uses XOR
on the sign bit of the IEEE 754 representation. Avoids floating point comparison
overhead.

---

### `system_model_at_date` (wasm_bindgen exported)
```rust
pub fn system_model_at_date(julian_date: f64) -> Vec<f64>
```
Returns heliocentric ecliptic longitudes for all 8 planets (Mercury through Neptune)
for the solar system display visualization. Uses `vsop87d` (spherical, ecliptic of date).
Frame comparisons must use the same ecliptic and equinox.

---

## TypeScript / React

### Custom Hook: `useDateSearch`
Owns the Web Worker lifecycle. Spawns worker on mount, terminates on unmount.
Exposes `{ search, results }` to components.

`search(params)` posts a message to the worker with:
```typescript
interface SearchParams {
    startJdTt: JulianDateTt;
    endJdTt: JulianDateTt;
    featureIds: Feature[];
    angleStarts: number[];
    angleSpans: number[];
}
```

The hook, worker, and Rust search implementation preserve these three equal-length
arrays. The worker converts them into one `Uint8Array` and two `Float64Array` values
for the WASM boundary. Matching Rust and TypeScript `Feature` enums give the shared
integer feature IDs readable names.

`results` is a `Float64Array` — flat-packed `[start_jd, end_jd, start_jd, end_jd, ...]`.

### Worker: `alignment.worker.ts`
Initializes WASM once on spawn (cached — subsequent calls are no-ops).
Calls the validated public `search` function with typed arrays constructed from params.
Error handling via try/catch with `postMessage({ type: 'ERROR' })`.

### Component: `SearchBox`
Currently hardcoded test search (Sun in Virgo, 2005-2006). Houses the
`formatResults` / `jdToDate` display logic.

### Component: `SolarSystem`
Visual solar system display. Uses `system_model_at_date` for heliocentric positions.
Accepts a `date: Date` prop and re-runs on date change.

### Date Utilities (`src/utils/time.ts`)

Strict Gregorian date parsing, civil-to-TT conversion, and labeled TT-to-civil conversion are shared and tested with `npm run test:time`.

---

## Key Mathematical Decisions

### Why Bisection over Brent's Method
Brent's method was discussed and the algorithm was designed, but plain bisection was
implemented first for simplicity. Brent's would converge in ~5-8 iterations vs ~14
for bisection on our interval sizes — a meaningful but not critical improvement.
Implementation should be straightforward using the argmin crate source as reference.

### Retrograde Station Detection — The Core Problem
The fundamental challenge is detecting a small angular excursion caused by retrograde
motion between coarse samples.

**Implemented solution:**
1. Use `instantaneous_velocity()` at each coarse step — not average velocity
2. When velocity sign changes between steps, bisect to find the exact station time
3. Split at the detected station and refine each resulting segment. Per-body coarse steps aim to avoid multiple stations within one segment; selected regression cases support their use, but are not an exhaustive proof for every possible query.

### The Aliasing Problem
Using `curr_lon - prev_lon` as a velocity proxy can give the wrong sign when a station
occurs near the end of a step (the planet slows, turns, but hasn't traveled back far
enough to make the net displacement negative). The implementation avoids this by using
`instantaneous_velocity()` at both ends of each coarse segment.

### Monotonic Interval Assumption
Between any two consecutive retrograde stations, a planet's geocentric longitude is
strictly monotonic. The algorithm treats segments between detected stations as monotonic, detects either or both constraint boundaries within each segment, and refines each crossing independently. Completeness depends on detecting the relevant stations; the finite reference suite does not prove that prerequisite for every epoch.

### Safe Step Sizes
The step must keep angular displacement below 180° so direction can be unwrapped
unambiguously. For retrograde features it must also be short enough that two stations
cannot occur inside one sampled segment. Constraint width does not limit the step because
both boundaries can be detected between samples.

| Planet | Current step |
|--------|--------------|
| Sun    | 28d          |
| Moon   | 10d          |
| Moon phase angle | 10d |
| Mercury| 3.5d         |
| Venus  | 12d          |
| Mars   | 18d          |
| Jupiter| 60d          |
| Saturn | 67d          |
| Uranus | 75d          |
| Neptune| 78d          |

Historical profiling of the earlier geometric implementation over 2000–2010 measured a standalone Moon-longitude sign
search at approximately 12 ms and a standalone Moon-phase sign search at approximately
360 ms. The latter includes the existing VSOP87 Sun calculation. In a ten-feature
query with the Sun evaluated first, the Moon-longitude and Moon-phase stages took
approximately 1 ms and 12 ms respectively because they operated on already narrowed
windows. Adding the lunar model increased the optimized WASM from 1,965.54 kB to
1,971.30 kB, an increase of 5.76 kB (0.29%).

---

## Current Validation and Remaining Work

The fixed position suite checks 108 body positions and 12 independently derived phase angles across representative 1900–2100 epochs. The complete-window suite checks 12 searches in 2024–2025 and returns all 15 independently constructed JPL windows. Its maximum non-Neptune endpoint difference is 9.422 seconds; the Neptune example is 919.622 seconds early. These are internal TT results before display rounding, not global accuracy bounds. Full measurements and case-specific limits are linked from the backend contract.

Backend preparation passed its final combined acceptance checks; see [backend_acceptance.md](backend_acceptance.md). Next, build the editable search form, worker lifecycle/error handling, and readable results. Engine replacement, streaming, multi-worker execution, caching, and additional bodies are deferred in [TODO.md](TODO.md); they do not block the UI milestone.

---

## Known Bugs

The historical ~0.36° display discrepancy was an of-date versus J2000 frame difference. Current accuracy limitations are recorded in the backend contract.

---

## Dependencies

### Rust (Cargo.toml)
```toml
wasm-bindgen = "0.2"
vsop87 = "2.1"
astro = "2.0"
```

### TypeScript (package.json)
```json
"clsx": "^2.1.1"
"react": "^19.2.4"
"react-dom": "^19.2.4"
```

---

## Project Summary

A browser-based reverse search for dates matching selected planetary signs, using Rust compiled to WebAssembly in a Web Worker. The initial domain is 1900–2100 with minute display and documented astronomical limitations. Search correctness has finite regression coverage; the editable first-version UI and public-release acceptance work remain pending. No universal latency or one-minute astronomical accuracy claim is established.
