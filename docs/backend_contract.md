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
