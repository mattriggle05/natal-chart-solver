// Canonical search time is Julian Date on the uniform Terrestrial Time scale.
export type JulianDateTt = number & { readonly __timeScale: 'TT' };
const DAY_MS = 86_400_000;
const UNIX_JD = 2_440_587.5;
const MJD_ZERO = 2_400_000.5;
// IERS Leap_Second.dat, Bulletin C 72 (July 2026). Effective MJD and TAI-UTC.
const LEAPS = [41317,41499,41683,42048,42413,42778,43144,43509,43874,44239,44786,45151,45516,46247,47161,47892,48257,48804,49169,49534,50083,50630,51179,53736,54832,56109,57204,57754];
const CONFIRMED_UNTIL_MS = Date.UTC(2027, 0, 1);
export interface CivilTime {
    unixMs: number;
    scale: 'UT estimate' | 'UTC' | 'UTC provisional';
    leapSecond: boolean;
}

function requireFinite(value: number): void {
    if (!Number.isFinite(value)) throw new RangeError('Time must be finite');
}

// Espenak/Meeus historical Delta T polynomials; used only before 1972.
function historicalDeltaT(unixMs: number): number {
    const date = new Date(unixMs);
    const year = date.getUTCFullYear();
    const y = year + (unixMs - Date.UTC(year, 0, 1)) / (Date.UTC(year + 1, 0, 1) - Date.UTC(year, 0, 1));
    if (y < 1899 || y >= 1972) throw new RangeError('Historical conversion is limited to the initial search era');
    if (y < 1920) { const t = y - 1900; return -2.79 + 1.494119*t - 0.0598939*t*t + 0.0061966*t**3 - 0.000197*t**4; }
    if (y < 1941) { const t = y - 1920; return 21.20 + 0.84493*t - 0.076100*t*t + 0.0020936*t**3; }
    if (y < 1961) { const t = y - 1950; return 29.07 + 0.407*t - t*t/233 + t**3/2547; }
    const t = y - 1975;
    return 45.45 + 1.067*t - t*t/260 - t**3/718;
}

/** Input is UTC from 1972, estimated UT before then. Future UTC holds the last known leap offset. */
export function civilUnixMsToJdTt(unixMs: number): JulianDateTt {
    requireFinite(unixMs);
    if (!Number.isFinite(new Date(unixMs).getTime())) throw new RangeError('Invalid civil time');
    const jd = unixMs / DAY_MS + UNIX_JD;
    let offset = 0;
    if (jd < MJD_ZERO + LEAPS[0]) offset = historicalDeltaT(unixMs);
    else {
        let index = LEAPS.length - 1;
        while (jd < MJD_ZERO + LEAPS[index]) --index;
        offset = 32.184 + 10 + index;
    }
    return (jd + offset / 86400) as JulianDateTt;
}

/** Parse a Gregorian date-only input as civil midnight without browser-local time-zone inference. */
export function civilDateToJdTt(value: string): JulianDateTt {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) throw new RangeError('Use YYYY-MM-DD');
    const ms = Date.parse(value + 'T00:00:00Z');
    if (!Number.isFinite(ms) || new Date(ms).toISOString().slice(0, 10) !== value) throw new RangeError('Invalid calendar date');
    return civilUnixMsToJdTt(ms);
}

export function jdTtToCivil(jdTt: JulianDateTt): CivilTime {
    requireFinite(jdTt);
    if (!Number.isFinite(new Date((jdTt - UNIX_JD) * DAY_MS).getTime())) throw new RangeError('Invalid Julian date');
    for (let index = LEAPS.length - 1; index >= 0; --index) {
        const start = MJD_ZERO + LEAPS[index];
        const offset = 32.184 + 10 + index;
        if (jdTt >= start + offset / 86400) {
            const unixMs = (jdTt - UNIX_JD) * DAY_MS - offset * 1000;
            return { unixMs, scale: unixMs >= CONFIRMED_UNTIL_MS ? 'UTC provisional' : 'UTC', leapSecond: false };
        }
        // JavaScript cannot encode :60. Carry this fact separately; minute display rounds to the next midnight.
        if (index > 0 && jdTt >= start + (offset - 1) / 86400) {
            return { unixMs: (start - UNIX_JD) * DAY_MS, scale: 'UTC', leapSecond: true };
        }
    }
    let unixMs = (jdTt - UNIX_JD) * DAY_MS;
    // At the 1972 changeover the historical UT approximation need not match UTC exactly.
    unixMs = Math.min(unixMs, Date.UTC(1972, 0, 1) - 1);
    for (let i = 0; i < 5; ++i) unixMs = (jdTt - UNIX_JD) * DAY_MS - historicalDeltaT(Math.min(unixMs, Date.UTC(1972, 0, 1) - 1)) * 1000;
    return { unixMs, scale: 'UT estimate', leapSecond: false };
}
