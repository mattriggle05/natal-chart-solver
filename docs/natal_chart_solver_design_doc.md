# Natal Chart Solver — Design & Context Document

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
          → Rust streams results back via postMessage
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

### Coordinate System
All calculations use **geocentric ecliptic longitude** — the apparent position of a
planet as seen from Earth, measured in degrees along the ecliptic plane (0-360°).
This matches the tropical zodiac used in astrology.

### VSOP87 Variants Used
The `vsop87` Rust crate provides several variants. We use:

| Module | Coordinates | Frame | Used For |
|--------|-------------|-------|----------|
| `vsop87c` | Heliocentric rectangular | Ecliptic of date | Geocentric calculations (search) |
| `vsop87b` | Heliocentric spherical | J2000 | Sun longitude (earth + 180°) |
| `vsop87d` | Heliocentric spherical | Ecliptic of date | Solar system display model |

**Why ecliptic of date (vsop87c/d) over J2000 (vsop87a/b)?**
The tropical zodiac is defined relative to the vernal equinox of the current date, not
J2000. Users entering natal chart positions from modern astrology apps expect "ecliptic
of date" coordinates. Using J2000 would introduce a ~0.37° error by 2026 (growing at
~50.3 arcseconds/year due to precession). vsop87c was verified against JPL Horizons
and matches to within 0.001° for geocentric longitude.

**Known limitation:** vsop87d heliocentric longitudes are ~0.36° off from JPL for
display purposes. This is believed to be a bug or mislabeling in the Rust crate's
equinox-of-date transformation. Since heliocentric display is cosmetic only, this
is acceptable. Geocentric calculations (the search) are accurate.

### Geocentric Longitude Calculation
For planets (VSOP87C provides rectangular heliocentric coordinates):
```
dx = planet.x - earth.x
dy = planet.y - earth.y
longitude = atan2(dy, dx) converted to degrees, rem_euclid(360)
```

For the Sun:
```
longitude = (vsop87b::earth(jd).longitude().to_degrees() + 180°).rem_euclid(360°)
```

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
```
IDs 8, 9 are intentionally unused (reserved). The Moon and Pluto are not currently
implemented (see Future Work).

### Zodiac Sign Mapping
Signs are indexed 0-11 (Aries=0 through Pisces=11). A planet at longitude L is in
sign `(L / 30.0) as u8`. This is computed as `L * (1.0/30.0)` using a compile-time
constant reciprocal to avoid division.

### Julian Date Conversion
TypeScript:
```typescript
const jd = date.getTime() / 86400000.0 + 2440587.5;
```
The library uses JDE (Terrestrial Time) internally. The difference between UTC and TT
is currently ~69 seconds — negligible for our use case (windows measured in days).

---

## Rust Functions

### `search2` (primary search function, wasm_bindgen exported)
```rust
pub fn search2(start_julian_date: f64, end_julian_date: f64,
               feature_ids: &[u8], feature_signs: &[u8]) -> Vec<f64>
```

**Parameters:**
- `start_julian_date`, `end_julian_date`: Julian date range to search
- `feature_ids`: Planet IDs in the order they should be evaluated (best filter first)
- `feature_signs`: Corresponding zodiac sign index (0-11) for each planet

**Returns:** Flat-packed `Vec<f64>` of `[window_start_jd, window_end_jd, ...]` pairs

**Algorithm:**
1. Starts with one window: the full search range
2. For each planet in order:
   - For each current candidate window:
     - Coarse sweep using a conservative per-body step
     - At each step: compute current longitude, check sign membership
     - Detect retrograde stations via instantaneous velocity sign changes
     - On detected station: bisect to find precise station time
     - Split the step into monotonic segments around the station
     - On sign boundary crossing: bisect to find precise crossing time
     - Accumulate sub-windows where this planet is in the correct sign
   - Replace window list with new sub-windows
3. Return final intersected windows

**Known issues / incomplete areas:**
- Search input validation and structured errors are not implemented
- Exact boundary inclusion semantics still need to be formalized across the public API
- No streaming/callback — returns all results at once after full computation

**Current conservative coarse steps (per planet):**
```
Sun:     14 days
Mercury: 3.5 days
Venus:   12 days
Mars:    18 days
Jupiter: 60 days
Saturn:  67 days
Uranus:  75 days
Neptune: 78 days
```

---

### `geocentric_longitude`
```rust
pub fn geocentric_longitude(julian_date: f64, feature_id: u8) -> f64
```
Returns geocentric ecliptic longitude in degrees [0, 360) for a given body at a given
Julian date. Returns `f64::NAN` for unknown feature IDs (NAN propagates visibly through
downstream calculations rather than silently corrupting results like -1.0 would).

Earth (`vsop87c::earth`) is computed once and reused for all planet calculations in a
single call to avoid redundant VSOP87 evaluations.

---

### `longitude_from_observer`
```rust
pub fn longitude_from_observer(observer_coords: RectangularCoordinates,
                                feature_coords: RectangularCoordinates) -> f64
```
Converts heliocentric rectangular coordinates to geocentric ecliptic longitude via
vector subtraction and atan2. Named to clearly express "longitude of feature as seen
from observer." The z dimension is intentionally ignored — ecliptic latitude is not
needed for zodiac sign membership which is purely a function of ecliptic longitude.

---

### `instantaneous_velocity`
```rust
pub fn instantaneous_velocity(julian_date: f64, feature_id: u8) -> f64
```
Computes geocentric angular velocity in degrees/day using the centered finite difference:
```
v(t) = (longitude(t + H) - longitude(t - H)) / (2H)
```
Where `H = 6e-6` days (cube root of f64::EPSILON — optimal h that balances truncation
error O(h²) against floating point cancellation error O(ε/h)).

**Why centered difference over one-sided:**
The centered difference is ~400x more accurate near velocity zero (at retrograde stations).
Near a station the true velocity is tiny — a one-sided error of 1.5e-8 could flip the
sign where the centered error of 3.6e-11 would not. Sign correctness at stations is
critical for the algorithm.

Cost: 2 VSOP87 calls per evaluation (4 total including earth computation).

---

### `bisection_derivative_find_zero`
```rust
pub fn bisection_derivative_find_zero(start_julian_date: f64,
                                       end_julian_date: f64,
                                       feature_id: u8) -> f64
```
Finds the Julian date of a retrograde station (velocity zero) within a given interval
using bisection. Caller must guarantee opposite velocity signs at endpoints.

**Termination conditions:**
- `|velocity| <= 6e-12` degrees/day (proportional to H² — the error floor of `instantaneous_velocity`)
- Interval width < 1 minute (1/1440 days) — floating point refinement limit

**Bisection direction:** Uses `f64_same_sign` against reference velocity at `left` to
determine which half contains the zero. Correctly handles both prograde→retrograde and
retrograde→prograde stations.

---

### `bisection_value_find`
```rust
pub fn bisection_value_find(start_julian_date: f64, end_julian_date: f64,
                             target_value: f64, feature_id: u8) -> f64
```
Finds the Julian date when a planet's geocentric longitude equals `target_value` within
a given interval. Used to find precise sign boundary crossings.

**Termination conditions:**
- `|longitude - target| < 1/3600°` (1 arcsecond — the accuracy floor of VSOP87C)
- Interval width < 1 minute

**Target values** are sign boundaries: `sign_index * 30.0` (entry) or
`(sign_index + 1) * 30.0` (exit).

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
Known ~0.36° inaccuracy vs JPL for display purposes only — acceptable.

---

### `search` (legacy, superseded by `search2`)
The original brute-force search — steps 1 day at a time and checks every day for
sign membership. Kept for reference but should be removed. No bisection, no window
refinement, returns individual dates not ranges.

---

## TypeScript / React

### Custom Hook: `useDateSearch`
Owns the Web Worker lifecycle. Spawns worker on mount, terminates on unmount.
Exposes `{ search, results }` to components.

`search(params)` posts a message to the worker with:
```typescript
interface SearchParams {
    startJd:    number;
    endJd:      number;
    featureIds: number[];
    featureSigns: number[];
}
```

`results` is a `Float64Array` — flat-packed `[start_jd, end_jd, start_jd, end_jd, ...]`.

### Worker: `alignment.worker.ts`
Initializes WASM once on spawn (cached — subsequent calls are no-ops).
Calls `search2` with typed arrays constructed from params.
Error handling via try/catch with `postMessage({ type: 'ERROR' })`.

### Component: `SearchBox`
Currently hardcoded test search (Sun in Libra, 2005-2006). Houses the
`formatResults` / `jdToDate` display logic.

### Component: `SolarSystem`
Visual solar system display. Uses `system_model_at_date` for heliocentric positions.
Accepts a `date: Date` prop and re-runs on date change.

### Date Utilities (`src/utils/date.ts` — partially implemented)
`jdToDate(jd: number): string` — Julian Date to MM/DD/YYYY (Meeus Ch.7 algorithm).
Gregorian-to-JD conversion exists in discussion but may not be in utils yet.

---

## Key Mathematical Decisions

### Why Bisection over Brent's Method
Brent's method was discussed and the algorithm was designed, but plain bisection was
implemented first for simplicity. Brent's would converge in ~5-8 iterations vs ~14
for bisection on our interval sizes — a meaningful but not critical improvement.
Implementation should be straightforward using the argmin crate source as reference.

### Retrograde Station Detection — The Core Problem
The fundamental challenge: no finite step size can guarantee finding an arbitrarily
small angular excursion caused by retrograde motion crossing a sign boundary briefly.

**Implemented solution:**
1. Use `instantaneous_velocity()` at each coarse step — not average velocity
2. When velocity sign changes between steps, bisect to find the exact station time
3. This is guaranteed correct because planetary retrogrades have a physical minimum
   duration governed by orbital mechanics — the minimum retrograde duration sets the
   safe step size, not an arbitrary choice

### The Aliasing Problem
Using `curr_lon - prev_lon` as a velocity proxy can give the wrong sign when a station
occurs near the end of a step (the planet slows, turns, but hasn't traveled back far
enough to make the net displacement negative). The implementation avoids this by using
`instantaneous_velocity()` at both ends of each coarse segment.

### Monotonic Interval Guarantee
Between any two consecutive retrograde stations, a planet's geocentric longitude is
strictly monotonic. This guarantees at most one sign boundary crossing per interval,
making bisection provably correct (by the Intermediate Value Theorem). The algorithm
partitions time into monotonic intervals using station times as breakpoints.

### Safe Step Sizes
The step must be short enough that one monotonic segment cannot pass completely through
a 30° sign without either endpoint landing inside it. It must also be short enough that
two stations cannot occur inside one sampled segment. The current implementation uses
additional safety margin after the randomized verifier demonstrated that the earlier
43-day Mars estimate could skip a complete Aries interval.

| Planet | Current step |
|--------|--------------|
| Sun    | 14d          |
| Mercury| 3.5d         |
| Venus  | 12d          |
| Mars   | 18d          |
| Jupiter| 60d          |
| Saturn | 67d          |
| Uranus | 75d          |
| Neptune| 78d          |

---

## Future Work / Incomplete Items

### High Priority

**1. Streaming results back to UI**
Currently `search2` returns only after full computation. Pass a `js_sys::Function`
callback into the Rust function and call it with each window as it's found. The UI
can then populate progressively rather than waiting for completion.

**2. Sun as first filter**
The Sun never retrogrades and is cheapest to calculate. It should always be the first
planet evaluated regardless of user input order, collapsing the search space by ~92%
before any other planet is checked.

### Medium Priority

**6. Multi-worker parallelism**
Split the date range across N workers (N = navigator.hardwareConcurrency).
Each worker searches a sub-range and posts results back. An orchestrator merges
and sorts. SharedArrayBuffer not needed — independent workers with postMessage
coordination is sufficient.

**7. Dynamic worker count based on device capability**
Use `navigator.hardwareConcurrency` and `navigator.deviceMemory` to choose worker
count. Run a micro-benchmark on first load and cache result in localStorage.

**8. Moon support**
ELP2000 series for the Moon — no maintained Rust crate exists. Implement the
truncated series from Meeus Ch. 47 (~60 terms) manually in Rust.

**9. Pluto support**
Small dedicated series from Meeus Ch. 37 (~40 terms). Manual implementation.

**10. Brent's method**
Replace plain bisection with Brent's method for faster convergence.
Reference: argmin crate `BrentRoot` implementation.

**11. Cancellation**
When user changes search parameters mid-computation, terminate the worker and spawn
a fresh one. Keep a compiled `WebAssembly.Module` object to pass to the new worker
to avoid recompilation cost.

### Lower Priority

**12. Precomputed ephemeris tile cache**
For the common case (inner solar system, popular date ranges), precompute planet
longitudes every 0.5 days as a compact binary (Float32, delta-encoded). Deliver as
a static asset (~3.5MB compressed). WASM reads from table + linear interpolation,
falling back to full VSOP87 only for refinement. 5-10x speedup potential.

**13. Service Worker caching**
Cache WASM binary and any ephemeris tiles via Cache API. Instant repeat loads,
offline support.

**14. User input UI**
Currently the search is hardcoded in `SearchBox`. Need a proper UI for:
- Selecting planets
- Selecting zodiac signs per planet
- Date range selection
- Results display with JD→calendar conversion

**15. Ecliptic dial input**
Draggable SVG dial for each planet showing ecliptic position. Users drag to their
natal chart position rather than selecting a sign from a dropdown.

---

## Known Bugs

1. **`system_model_at_date` heliocentric longitudes ~0.36° off JPL** — believed to be
   a vsop87d crate issue. Display only, acceptable.

---

## Dependencies

### Rust (Cargo.toml)
```toml
wasm-bindgen = "0.2"
vsop87 = "2.1"
console_error_panic_hook = "0.1"  # Rust panic messages in browser console
js-sys = "0.3"                    # For callback support (streaming — not yet implemented)
web-sys = { version = "0.3", features = ["console"] }  # console.log from Rust
```

### TypeScript (package.json)
```json
"astronomy-engine": "^2.1.19"  # Installed but superseded by Rust WASM — can be removed
"clsx": "^2.1.1"
"react": "^19.2.4"
"react-dom": "^19.2.4"
```

---

## Resume Description

> **Natal Chart Solver** — Built a browser-based astronomical alignment search engine
> that calculates historical dates matching a given planetary configuration across a
> 100-year span using VSOP87 ephemeris data compiled to WebAssembly via Rust.

> Architected a multi-threaded computation pipeline using Web Workers and Rust→WASM
> to offload intensive planetary calculations off the main thread, keeping the
> React/TypeScript UI fully responsive during searches.

> Deployed as a fully serverless application — all computation runs client-side,
> eliminating backend infrastructure costs while achieving sub-300ms search times
> across 12 planetary bodies.
