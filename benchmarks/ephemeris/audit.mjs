import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const read=name=>JSON.parse(readFileSync(new URL(`results/${name}.json`,import.meta.url)));
const throughput=read('throughput'),search=read('search');
assert.equal(throughput.calibration.length,44);
assert(throughput.trials.every(t=>Number.isInteger(t.repeats)&&t.repeats>=1&&t.repeats<=100));
assert.equal(throughput.trials.length,4*11*9);
assert.equal(throughput.throughput.length,44);
assert.equal(throughput.accuracy.length,40);
assert.equal(throughput.fixtures.length,12);
for(const fixture of throughput.fixtures)assert(fixture.errorsDegrees.every(e=>Math.abs(e)<(fixture.body===16?0.03:0.02)));
for(const engine of ['javascript','c'])assert(throughput.accuracy.filter(a=>a.engine===engine).every(a=>a.maxDegrees<1/60));
assert.equal(throughput.visibilityChanges,0,'Visibility changed during throughput run');
for(const row of throughput.throughput) {
    assert(row.medianMs>0&&row.p95Ms>=row.medianMs);
    const values=throughput.trials.filter(t=>t.engine===row.engine&&t.body===row.body);
    assert.equal(values.length,9);
    assert(values.every(t=>Number.isFinite(t.checksum)&&t.checksum===values[0].checksum));
}
for(const row of throughput.accuracy) {
    assert(Number.isFinite(row.maxDegrees)&&Number.isFinite(row.rmsDegrees));
    assert(row.maxDegrees>=row.rmsDegrees-1e-12);
    if(row.engine==='baseline')assert.equal(row.maxDegrees,0);
}
assert.equal(search.trials.length,27);
for(const row of search.comparison)assert(row.sameWindowCount&&row.maxBoundaryShiftMinutes<2);
for(const trial of search.trials) {
    assert.equal(trial.windows.length,488);
    for(let i=0;i<trial.windows.length;i+=2) {
        assert(trial.windows[i]<trial.windows[i+1]);
        assert(trial.windows[i]>=2415021&&trial.windows[i+1]<=2488070);
        if(i)assert(trial.windows[i]>=trial.windows[i-1]);
    }
}
const js=search.trials.find(t=>t.engine==='javascript').windows;
const c=search.trials.find(t=>t.engine==='c').windows;
assert.deepEqual(js,c,'JavaScript and C finalist searches differ');
console.log('Benchmark records pass completeness, finite values, repeatability, ordering, and finalist agreement checks.');
