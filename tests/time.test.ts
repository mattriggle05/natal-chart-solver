import test from 'node:test';
import assert from 'node:assert/strict';
import { civilDateToJdTt, civilUnixMsToJdTt, jdTtToCivil } from '../src/utils/time.ts';
import type { JulianDateTt } from '../src/utils/time.ts';

test('J2000 TT maps to 2000-01-01 11:58:55.816 UTC', () => {
    const utc = Date.parse('2000-01-01T11:58:55.816Z');
    assert(Math.abs(civilUnixMsToJdTt(utc) - 2451545) * 86400 < 0.0001);
    assert(Math.abs(jdTtToCivil(2451545 as JulianDateTt).unixMs - utc) < 0.1);
});
test('calendar parsing is strict and UTC based', () => {
    assert.equal(civilDateToJdTt('2000-02-29'), civilUnixMsToJdTt(Date.UTC(2000, 1, 29)));
    for (const invalid of ['1900-02-29', '2000-13-01', '01/01/2000', '2000-01-01T00:00:00']) assert.throws(() => civilDateToJdTt(invalid));
    assert.throws(() => civilUnixMsToJdTt(NaN));
    assert.throws(() => jdTtToCivil(Infinity as JulianDateTt));
});
test('historical and provisional civil times are labeled and round trip', () => {
    for (const year of [1900,1920,1941,1961,1971,1972,2000,2026,2050,2099]) {
        const ms = Date.UTC(year, 5, 15, 12, 34, 56);
        const result = jdTtToCivil(civilUnixMsToJdTt(ms));
        assert(Math.abs(result.unixMs - ms) < 0.1, `${year}: ${result.unixMs-ms}`);
        assert.equal(result.scale, year < 1972 ? 'UT estimate' : year >= 2027 ? 'UTC provisional' : 'UTC');
    }
});
test('a positive leap second adds one SI second and remains distinguishable', () => {
    const before = civilUnixMsToJdTt(Date.parse('2016-12-31T23:59:59Z'));
    const after = civilUnixMsToJdTt(Date.parse('2017-01-01T00:00:00Z'));
    assert(Math.abs((after-before)*86400 - 2) < 0.0001);
    assert.equal(jdTtToCivil((before + 1.5/86400) as JulianDateTt).leapSecond, true);
    assert.equal(jdTtToCivil(after).leapSecond, false);
});
