# Fixed complete-search references

This is the bounded step-3 suite: 12 searches, 15 expected windows. These offline test fixtures are not loaded by the application or included in its WASM assets.

## Independent construction

`scripts/fetch_jpl_windows.py` constructs the references using JPL Horizons quantity 31, geocentric target-center apparent longitudes, airless, with TT input and output. It does not call Rust or use production search results. `manifest.json` records the exact queries and SHA-256 hashes for 98 retained responses.

For each fixed constraint, a three-hour JPL grid brackets angular crossings. The reference builder unwraps each angular displacement and checks both arc boundaries, including both boundaries of the narrow Sun arc. It refines candidate roots with at most eight bracket-preserving secant iterations. Every resulting root must also pass a direct JPL sign-change check at one second before and after it. JPL midpoint queries classify the intervals between crossings. Ordinary interval intersection then constructs multi-constraint results. The Mercury station case additionally requires a sampled JPL turning point inside its expected window.

This grid is suitable for these deliberately selected cases, including the broad excursion through the Mercury station arc. It is not a proof that arbitrarily small grazing excursions anywhere in 1900–2100 cannot be missed. Existing Rust tests retain coverage of numerical edge cases and direct-evaluation comparisons. The independent reference suite adds end-to-end astronomical comparisons without attempting exhaustive validation.

Run offline:

```sh
python3 scripts/fetch_jpl_windows.py --verify
cargo test --manifest-path crate/Cargo.toml --release complete_search_windows_match_fixed_jpl_references -- --nocapture
```

The verifier checks hashes, time/frame/query conventions, returned epochs, one-second root brackets, and exact regeneration of `cases.txt`. It requires no network. Run the script without `--verify` only to intentionally collect/refresh reference data. It uses batches of at most 40 dates because a larger request was silently truncated by the API. Successful responses are retained for resumption after transient connection failures.

## Cases and results

Observed differences are internal JD(TT) endpoint differences before display rounding. Budgets were fixed before comparing Rust results; no failing budget was enlarged to make this suite pass.

| Case | Expected windows | Maximum observed difference | Budget |
|---|---:|---:|---:|
| Sun in Aries | 1 | 2.394 s | 30 s |
| Moon wrapped arc, 350°–10° | 1 | 5.952 s | 60 s |
| Moon phase, 0°–30° | 2 | 9.422 s | 60 s |
| Mercury retrograde re-entry, 20°–25° | 3 | 3.275 s | 120 s |
| Mercury station-containing arc, 27°–28° | 2 | 9.339 s | 180 s |
| Sun and Moon in Aries | 1 | 5.335 s | 60 s |
| Both endpoints clipped | 1 | 0 s | Exact |
| Start clipped | 1 | 2.054 s | 30 s; clipped endpoint exact |
| End clipped | 1 | 2.549 s | 30 s; clipped endpoint exact |
| Empty Sun Aries / Mercury Libra combination | 0 | Not applicable | Empty output required |
| Neptune Aries ingress | 1 | 919.622 s | 1,200 s |
| Sun 10°–10.0002°, roughly 17.5-second window | 1 | 2.276 s | 30 s; positive subminute duration required |

The budgets are case-specific regression limits, not global astronomical error bounds. Sun gets a 30-second budget; Moon/phase and their combination get 60 seconds to accommodate the existing lunar model. Mercury has more allowance for slow retrograde motion, especially near a station. Neptune's explicit 20-minute budget preserves the already diagnosed approximately 15.3-minute model discrepancy as a known limitation; it does not redefine the product's desired accuracy or establish that 20 minutes is its maximum error. Public acceptance of model limitations remains a later decision.

The test first requires an exact window count: a timing budget cannot excuse missing or extra windows. It then checks finite, positive, ordered, nonoverlapping, in-range results; matching interiors against the production model; exact clipped endpoints; and the individual reference endpoint budgets. It explicitly requires multiple Mercury re-entry windows, empty output for the empty case, and preservation of the subminute window.

`cases.txt` is a pipe-separated format: name, start JD(TT), end JD(TT), feature IDs, arc starts, arc spans, timing budget in seconds, and flattened reference window endpoints. Lists inside a field are comma-separated. An empty last field means no windows. The fixed case names/count are also checked by Rust to prevent silently losing coverage.
