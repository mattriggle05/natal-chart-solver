# Backend calculation contract

Status as of 2026-09-30: the apparent-position implementation and fixed reference suites are complete. The final combined acceptance pass is still pending. This contract describes a prototype; public acceptance of its astronomical limitations remains a separate release decision.

## Search domain

The initial search domain is `[1900-01-01 00:00 TT, 2100-01-01 00:00 TT)`, expressed as `[2415020.5, 2488069.5)` in JD(TT). The end instant may be supplied as an exclusive search endpoint. This is a fixed 200-year domain, not a moving interval relative to the browser clock, and does not include the remainder of calendar year 2100.

The user may choose any nonempty subrange. Automatic frontend defaults must also fit within this domain. The public Rust `search` function rejects requests outside it rather than silently clipping them. The exported `search_date_range` function supplies the two bounds to callers so the frontend need not duplicate them. Internal ephemeris evaluations may probe just outside a requested interval for numerical derivatives.

These limits define the scope being validated, not a completed astronomical accuracy certification. No restriction on search duration within this domain is imposed yet. Historical and future civil-time conversion must be handled separately from the TT search domain.

The production backend remains Rust. The archived ephemeris benchmark is a historical experiment; C/WASM integration is deferred until the Rust product is working.

## Time scale and calendar

All Rust ephemeris and search arguments and results use JD(TT), including fractional days. TypeScript uses a branded `JulianDateTt` and explicitly named worker fields. Civil date inputs use Gregorian `YYYY-MM-DD` at midnight; browser time-zone inference is not used. The heliocentric display also receives TT.

From 1972 onward, civil conversion uses TT = UTC + (TAI−UTC) + 32.184 seconds with the [IERS Bulletin C 72 leap-second table](https://hpiers.obspm.fr/iers/bul/bulc/Leap_Second.dat). Dates from 2027 onward hold the last known offset and are labeled `UTC provisional`; future leap seconds cannot yet be known. Before 1972, conversion uses the [Espenak/Meeus Delta T polynomials](https://eclipse.gsfc.nasa.gov/SEhelp/deltatpoly2004.html) and is labeled `UT estimate`, not historical UTC. These civil-time uncertainties are separate from ephemeris and numerical error.

JavaScript timestamps cannot represent `23:59:60`. Inverse conversion flags a leap-second instant and maps its display timestamp to the following midnight; the original TT result remains unchanged. Search bounds apply after conversion: a civil midnight is not generally a TT midnight, so full-domain controls should use the exported TT bounds directly.

Run `npm run test:time` with Node 24 to verify conversion anchors, calendar validation, historical and future labeling, and a leap-second transition. Conversion round trips verify implementation consistency, not astronomical accuracy.

## Numerical and display precision

Angular crossings and station brackets refine to one second, retaining full floating-point JD(TT) endpoints. Bisection returns the bracket midpoint, giving at most half a second of bracketing error for a correctly bracketed root of the implemented model. This is not a bound on ephemeris error or station-detection error.

Display rounds each civil endpoint to its nearest minute. Rounding can add up to 30 seconds of error; it must never affect interval intersection, merging, or subsequent searches. Positive windows shorter than one minute remain in results and receive a `less than one minute` label, including when both displayed endpoints round to the same minute.

A strict 60-second astronomical boundary target would leave at most 29.5 seconds for ephemeris and time-conversion error after numerical refinement and display rounding. That target has not been certified. Slow motion near stations can turn very small longitude errors into large time errors; no universal time guarantee follows from a longitude tolerance alone.

## Apparent coordinates

The Rust pipeline evaluates apparent geocentric tropical ecliptic longitude using VSOP87E barycentric positions and the existing astro lunar series. VSOP vectors are rotated from the dynamical J2000 ecliptic to FK5 before combination; the lunar mean-of-date position is precessed into that same inertial frame. Reception and emission positions therefore share one frame.

Corrections include iterated light travel time, finite-distance solar gravitational deflection (limited inside the solar disk), and relativistic observer-velocity aberration. The result is precessed to date and receives astro's analytical IAU1980 nutation. Moon phase is the difference between the apparent Moon and Sun longitudes. The heliocentric visualization retains its separate VSOP87D display coordinates.

This is an analytical true-equinox-of-date convention, without measured Earth-orientation corrections. Horizons applies observed EOP corrections in the modern era. The retained 2025 Neptune diagnostic isolates that difference: about +0.1095 arcsecond before applying the independently published IERS dPsi adjustment, less than 0.002 arcsecond afterward when using JPL positions. EOP is used only in the test; the application downloads no ephemeris or EOP data. This single correction-pipeline check does not remove VSOP87's planetary model error or certify the full domain.

Ephemeris arguments use TT; the millisecond-scale TT/TDB difference is neglected by these analytical models. The small solar-potential term in relativistic aberration is also omitted. Solar deflection uses the Sun at reception; light paths inside the Sun are regularized rather than physically modeled. These approximations are distinct from the measured Neptune model discrepancy. The correction tests and numerical refinement do not establish one-minute astronomical accuracy.


## Position regression coverage

The fixed JPL regression suite checks 108 body positions and 12 derived Moon phase angles, plus the known Neptune ingress position. Per-feature angular budgets, measured residuals, and offline provenance verification are recorded in [fixtures/jpl/README.md](../fixtures/jpl/README.md). These sampled checks replace the earlier broad Sun/Moon/phase tolerances; they do not certify every epoch or complete search windows. The fixed complete-window suite is documented in [fixtures/jpl/windows/README.md](../fixtures/jpl/windows/README.md): 12 JPL-derived searches, exact window-count and clipping checks, and separate case-specific timing budgets. It retains the known Neptune discrepancy and does not establish a universal one-minute accuracy guarantee.


## Measured accuracy and interpretation

| Layer | Established behavior or measurement | What it does not establish |
|---|---|---|
| Numerical refinement | One-second brackets; midpoint error at most half a second for a correctly bracketed model root | Accuracy of the astronomical model or completeness of station detection |
| Display | Nearest civil minute; up to 30 seconds of rounding | A one-minute astronomical guarantee |
| Position comparisons | 108 body samples at 12 epochs spanning the domain; 12 phase samples derived from JPL | Maximum error between samples or at every boundary/station |
| Complete search comparisons | 12 fixed searches in 2024–2025; all 15 expected windows returned, with no missing or extra windows | Completeness of every possible query across 1900–2100 |
| Non-Neptune window endpoints | Largest absolute difference in this suite: 9.422 seconds against JPL, before display rounding | A global ten-second bound or a bound for all other planets |
| Neptune window endpoint | 919.622 seconds early, about 15.33 minutes, at the tested 2025 Aries ingress | A constant correction or maximum possible Neptune error |
| Correction-pipeline isolation | JPL positions plus a test-only observed nutation adjustment leave about 0.00056 arcsecond residual at one epoch | Accuracy of production VSOP87 positions or the absence of all other frame/model differences |

The position samples' largest residuals include 2.539385 arcseconds for Neptune, 5.320064 arcseconds for the Moon, and 5.275545 arcseconds for derived Moon phase. The full per-feature measurements and regression limits live in the position fixture README. The complete-window suite covers Sun, Moon, phase, Mercury, and Neptune, including one Sun/Moon combination; it does not yet establish complete-window accuracy for Venus, Mars, Jupiter, Saturn, or Uranus.

Regression limits are explicit test budgets chosen to detect changes to the diagnosed implementation. They are not observational uncertainties or global error bounds. In particular, the Neptune case's 1,200-second allowance records a known limitation; passing that test is not acceptance of twenty-minute accuracy as a product requirement. Do not enlarge a failing limit without diagnosing the change.

During approximately the interval separating our predicted sign entry from an independent chart's sign entry, the two models can assign different signs. This can shift an overlap or make a short matching window appear or disappear. The fixed suite found no missing/extra windows, but does not rule them out for other queries. Near a station, angular motion approaches zero, so even a small longitude error can imply a large boundary-time error. A fixed minute adjustment or a whole-domain guarantee inferred from these samples is not justified.

Historical UT estimates and unknown future leap seconds add civil-time uncertainty separately from model error. The reference-window comparisons use TT directly; their measured timing differences do not include civil-time conversion or minute rounding. The user-facing wording should state that boundaries are approximate and displayed to the nearest minute, rather than describing them as accurate to one minute.

## Validation boundary and next milestone

The bounded position and complete-window suites are sufficient inputs to the planned final acceptance pass; no exhaustive ingress survey or new ephemeris investigation is required before the UI. Step 5 runs the existing full Rust suite, TypeScript time tests, offline fixture verification, and production WASM/TypeScript/Vite build together. New work is justified by an actual failure or an explicitly expanded requirement, not by the possibility of adding more samples indefinitely.

After that pass, begin the rudimentary UI described in [TODO.md](TODO.md). Keep VSOP87, Rust, and the documented limitations. Ephemeris replacement, stronger global accuracy guarantees, and broader performance optimization remain deferred. This is a development stopping rule, not approval of the unresolved public-release accuracy promise.
