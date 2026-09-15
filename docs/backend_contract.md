# Backend calculation contract

## Search domain

The initial search domain is `[1900-01-01 00:00 TT, 2100-01-01 00:00 TT)`, expressed as `[2415020.5, 2488069.5)` in JD(TT). The end instant may be supplied as an exclusive search endpoint. This is a fixed 200-year domain, not a moving interval relative to the browser clock, and does not include the remainder of calendar year 2100.

The user may choose any nonempty subrange. Automatic frontend defaults must also fit within this domain. The public Rust `search` function rejects requests outside it rather than silently clipping them. The exported `search_date_range` function supplies the two bounds to callers so the frontend need not duplicate them. Internal ephemeris evaluations may probe just outside a requested interval for numerical derivatives.

These limits define the scope being validated, not a completed astronomical accuracy certification. No restriction on search duration within this domain is imposed yet. Historical and future civil-time conversion must be handled separately from the TT search domain.

The production backend remains Rust. The archived ephemeris benchmark is a historical experiment; C/WASM integration is deferred until the Rust product is working.
