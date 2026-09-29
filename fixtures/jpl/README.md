# JPL apparent reference investigation

Offline Horizons observer-table quantity 31 fixtures, Earth center `500@399`, TT input and output, no atmospheric refraction. `manifest.json` records requests and hashes of original responses. Regenerate with `python3 scripts/fetch_jpl.py`; this requires network access. Dates include both domain endpoints and representative intermediate epochs. These samples do not establish whole-domain or near-station accuracy.

The uncommitted apparent-position implementation uses VSOP87E barycentric coordinates, iterated light time, solar deflection, observer-velocity aberration, precession, FK5 correction, and nutation. It retains the existing astro lunar series. At the 108 sampled body/epoch combinations, maximum absolute longitude errors in arcseconds were:

| Mercury | Venus | Mars | Jupiter | Saturn | Uranus | Neptune | Sun | Moon |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.104 | 0.097 | 0.092 | 0.481 | 0.401 | 1.092 | 2.539 | 0.098 | 5.410 |

`python3 scripts/probe_neptune.py` independently queries and refines Neptune's 2025 Aries ingress. The raw responses are retained as `neptune_ingress_*.json`; the query/iteration record is `neptune_ingress.json`. Quantity 31 has finite output precision; the final root is approximate to a fraction of a second, not exact to every printed digit.

- JPL ingress: approximately JD(TT) 2460764.999763406.
- Current Rust apparent implementation: JD(TT) 2460764.989118159.
- Difference: approximately 919.75 seconds (15.33 minutes), Rust earlier.

This rejects a universal one-minute boundary accuracy claim for the current implementation. It is one complete crossing comparison, not the planned complete multi-constraint window suite. The decision between stricter ephemeris accuracy and minute display with disclosed model uncertainty is pending. Neither the wider fixture validation task nor the complete search-window validation task is complete.

Follow-up diagnosis: see [ephemeris_diagnosis.md](../../../docs/ephemeris_diagnosis.md). Both Rust packages reproduce the original Neptune coefficients; controlled JPL substitutions identify Neptune position data as the dominant source of error. Smaller frame-convention differences remain unresolved.
