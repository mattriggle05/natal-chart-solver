# Backend calculation contract

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

The fixed JPL regression suite checks 108 body positions and 12 derived Moon phase angles, plus the known Neptune ingress position. Per-feature angular budgets, measured residuals, and offline provenance verification are recorded in [fixtures/jpl/README.md](../fixtures/jpl/README.md). These sampled checks replace the earlier broad Sun/Moon/phase tolerances; they do not certify every epoch or complete search windows. Complete window tests remain the next milestone.
